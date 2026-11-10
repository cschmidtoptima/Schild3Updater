use crate::{compatible_server, config::Settings, install, version_info};
use anyhow::{bail, Context, Result};
use reqwest::{blocking::Client, Url};
use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::Path,
    time::Duration,
};

const REPOSITORY: &str = "https://api.github.com/repos/SVWS-NRW/Schild-NRW-3/releases";
const MAX_DOWNLOAD: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub digest: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Release {
    pub tag_name: String,
    #[serde(default)]
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

#[derive(Clone)]
pub struct UpdatePlan {
    pub settings: Settings,
    pub installed: Version,
    pub server: Version,
    pub target: Version,
    pub asset: Asset,
}

pub fn http_client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!("SchILD3Updater/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(300))
        .https_only(true)
        .build()?)
}

pub fn server_version(client: &Client, address: &str) -> Result<Version> {
    let mut url = Url::parse(address.trim())
        .context("Bitte eine vollstaendige HTTPS-Serveradresse eingeben.")?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        bail!("Die Serveradresse muss HTTPS verwenden und darf keine Zugangsdaten enthalten.");
    }
    url.set_query(None);
    url.set_fragment(None);
    let base = format!("{}/", url.as_str().trim_end_matches('/'));
    let endpoint = Url::parse(&base)?.join("status/version")?;
    let response = client
        .get(endpoint)
        .timeout(Duration::from_secs(30))
        .send()?
        .error_for_status()?;
    let mut text = String::new();
    response.take(4097).read_to_string(&mut text)?;
    if text.len() > 4096 {
        bail!("Unerwartet grosse Antwort des Versionsendpunkts.");
    }
    let text = serde_json::from_str::<String>(&text).unwrap_or_else(|_| text.trim().to_owned());
    Version::parse(text.trim().trim_start_matches('v'))
        .context("Der Server liefert keine eindeutige Versionsnummer.")
}

pub fn select_release(releases: Vec<Release>, server: &Version) -> Result<(Version, Asset)> {
    let mut candidates = Vec::new();
    for release in releases {
        if release.draft
            || release.prerelease
            || compatible_server(release.body.as_deref().unwrap_or("")).as_ref() != Some(server)
        {
            continue;
        }
        let Ok(version) = Version::parse(release.tag_name.trim_start_matches('v')) else {
            continue;
        };
        if !version.pre.is_empty() {
            continue;
        }
        let expected = format!("Setup_SchILD3_v{version}.zip");
        if let Some(asset) = release
            .assets
            .into_iter()
            .find(|asset| asset.name == expected)
        {
            candidates.push((version, asset));
        }
    }
    candidates.sort_by(|left, right| right.0.cmp(&left.0));
    candidates.into_iter().next().context("Kein ausdruecklich fuer diese Serverversion freigegebenes Client-ZIP gefunden. Es werden keine Dateien veraendert.")
}

pub fn check(settings: &Settings, progress: &impl Fn(String)) -> Result<UpdatePlan> {
    let directory = Path::new(settings.client_dir.trim());
    if !directory.is_absolute() || !directory.is_dir() {
        bail!("Bitte einen vorhandenen absoluten Clientpfad oder UNC-Pfad eingeben.");
    }
    install::ensure_no_interrupted_update(directory)?;
    let installed = version_info::client_version(directory)?;
    let client = http_client()?;
    progress("Serverversion wird gelesen ...".into());
    let server = server_version(&client, &settings.server_url)?;
    progress(format!(
        "SVWS {server}: kompatible GitHub-Releases werden gesucht ..."
    ));
    let mut releases = Vec::new();
    for page in 1..=10 {
        let batch: Vec<Release> = client
            .get(REPOSITORY)
            .query(&[("per_page", 100), ("page", page)])
            .send()?
            .error_for_status()?
            .json()
            .context("GitHub-Releases konnten nicht gelesen werden.")?;
        let finished = batch.len() < 100;
        releases.extend(batch);
        if finished {
            break;
        }
    }
    let (target, asset) = select_release(releases, &server)?;
    if installed > target {
        bail!("Installierter Client {installed} ist neuer als die freigegebene Version {target} fuer Server {server}. Automatisches Downgrade ist gesperrt.");
    }
    Ok(UpdatePlan {
        settings: settings.clone(),
        installed,
        server,
        target,
        asset,
    })
}

pub fn download(
    client: &Client,
    asset: &Asset,
    progress: &impl Fn(String),
) -> Result<tempfile::NamedTempFile> {
    let url = Url::parse(&asset.browser_download_url)?;
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !url
            .path()
            .starts_with("/SVWS-NRW/Schild-NRW-3/releases/download/")
    {
        bail!("Downloadadresse liegt nicht im offiziellen Release-Repository.");
    }
    let expected = asset.digest.as_deref().and_then(|value| value.strip_prefix("sha256:"))
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .context("GitHub liefert fuer dieses Paket keine gueltige SHA-256-Pruefsumme. Update abgebrochen.")?;
    let mut response = client.get(url).send()?.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|size| size > MAX_DOWNLOAD)
    {
        bail!("Updatepaket ist zu gross.");
    }
    let mut temporary = tempfile::NamedTempFile::new()?;
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 65536];
    let mut total = 0u64;
    let mut last_mb = 0;
    loop {
        let count = response.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        if total > MAX_DOWNLOAD {
            bail!("Updatepaket ueberschreitet das Downloadlimit.");
        }
        temporary.write_all(&buffer[..count])?;
        hash.update(&buffer[..count]);
        let mb = total / (1024 * 1024);
        if mb >= last_mb + 4 {
            progress(format!("Download: {mb} MiB ..."));
            last_mb = mb;
        }
    }
    if format!("{:x}", hash.finalize()) != expected.to_ascii_lowercase() {
        bail!("SHA-256-Pruefsumme stimmt nicht. Update wurde verworfen.");
    }
    temporary.flush()?;
    Ok(temporary)
}

pub fn update(plan: &UpdatePlan, progress: &impl Fn(String)) -> Result<std::path::PathBuf> {
    let directory = Path::new(plan.settings.client_dir.trim());
    let client = http_client()?;
    if server_version(&client, &plan.settings.server_url)? != plan.server
        || version_info::client_version(directory)? != plan.installed
    {
        bail!("Server- oder Clientversion hat sich geaendert. Bitte erneut pruefen.");
    }
    if plan.installed >= plan.target {
        bail!("Kein Update erforderlich.");
    }
    progress(format!("{} wird heruntergeladen ...", plan.asset.name));
    let archive = download(&client, &plan.asset, progress)?;
    let staging = tempfile::tempdir()?;
    install::extract(archive.path(), staging.path())?;
    if version_info::client_version(staging.path())? != plan.target {
        bail!("Die EXE-Version im Paket entspricht nicht dem Release.");
    }
    #[derive(Deserialize)]
    struct ServerManifest {
        server_version: String,
    }
    let manifest: ServerManifest =
        serde_json::from_slice(&std::fs::read(staging.path().join("server.json"))?)?;
    if Version::parse(&manifest.server_version)? != plan.server {
        bail!("server.json im Paket passt nicht zum Server. Keine Installation.");
    }
    if server_version(&client, &plan.settings.server_url)? != plan.server {
        bail!("Serverversion hat sich waehrend des Downloads geaendert. Bitte erneut pruefen.");
    }
    progress("Paket verifiziert. Dateien werden gesichert und aktualisiert ...".into());
    install::apply(
        staging.path(),
        directory,
        &plan.installed,
        &plan.target,
        progress,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, server: &str) -> Release {
        Release {
            tag_name: format!("v{version}"),
            body: Some(format!("Lauffähig mit dem SVWS-Server {server}.")),
            draft: false,
            prerelease: false,
            assets: vec![Asset {
                name: format!("Setup_SchILD3_v{version}.zip"),
                browser_download_url: String::new(),
                digest: None,
            }],
        }
    }

    #[test]
    fn selects_highest_explicit_exact_match() {
        let releases = vec![
            release("3.5.0", "1.4.1"),
            release("3.6.0", "1.5.1"),
            release("3.5.1", "1.4.1"),
        ];
        let (version, _) = select_release(releases, &Version::new(1, 4, 1)).unwrap();
        assert_eq!(version, Version::new(3, 5, 1));
        assert!(select_release(vec![release("3.6.0", "1.5.1")], &Version::new(1, 5, 2)).is_err());
    }

    #[test]
    fn excludes_preview_and_missing_packages() {
        let mut preview = release("3.6.0", "1.5.1");
        preview.prerelease = true;
        assert!(select_release(vec![preview], &Version::new(1, 5, 1)).is_err());
        let mut empty = release("3.6.0", "1.5.1");
        empty.assets.clear();
        assert!(select_release(vec![empty], &Version::new(1, 5, 1)).is_err());
    }

    #[test]
    fn rejects_insecure_or_credential_urls_before_network() {
        let client = http_client().unwrap();
        for address in [
            "http://example.org",
            "https://user:secret@example.org",
            "no-url",
        ] {
            assert!(server_version(&client, address).is_err());
        }
    }

    #[test]
    #[ignore = "Expliziter Netzwerktest mit offiziellem ZIP und temporaerem Clientordner"]
    fn official_release_end_to_end() {
        let archive =
            std::env::var_os("SCHILD3_TEST_ARCHIVE").expect("SCHILD3_TEST_ARCHIVE setzen");
        let address = std::env::var("SCHILD3_TEST_SERVER").expect("SCHILD3_TEST_SERVER setzen");
        let extracted = tempfile::tempdir().unwrap();
        install::extract(Path::new(&archive), extracted.path()).unwrap();
        let client = tempfile::tempdir().unwrap();
        let original = std::fs::read(extracted.path().join(version_info::CLIENT_EXE)).unwrap();
        std::fs::write(client.path().join(version_info::CLIENT_EXE), &original).unwrap();
        std::fs::create_dir(client.path().join("EigeneDaten")).unwrap();
        std::fs::write(client.path().join("EigeneDaten/lokal.ini"), b"unveraendert").unwrap();
        let settings = Settings {
            server_url: address,
            client_dir: client.path().to_string_lossy().into_owned(),
        };
        let progress = |message| println!("{message}");
        let plan = check(&settings, &progress).unwrap();
        assert!(
            plan.installed < plan.target,
            "Fuer diesen Test ein aelteres offizielles ZIP verwenden"
        );
        let backup = update(&plan, &progress).unwrap();
        assert_eq!(
            version_info::client_version(client.path()).unwrap(),
            plan.target
        );
        assert_eq!(
            std::fs::read(client.path().join("EigeneDaten/lokal.ini")).unwrap(),
            b"unveraendert"
        );
        assert_eq!(
            std::fs::read(backup.join("originals").join(version_info::CLIENT_EXE)).unwrap(),
            original
        );
        install::ensure_no_interrupted_update(client.path()).unwrap();
    }
}
