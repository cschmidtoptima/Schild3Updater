use crate::{service, version_info};
use anyhow::{bail, ensure, Context, Result};
use semver::Version;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

pub const RELEASES_PAGE: &str = "https://github.com/cschmidtoptima/Schild3Updater/releases";
const API: &str = "https://api.github.com/repos/cschmidtoptima/Schild3Updater/releases/latest";
const DOWNLOAD_PREFIX: &str = "https://github.com/cschmidtoptima/Schild3Updater/releases/download/";
const ASSET_NAME: &str = "SchILD3Updater.exe";
const MAX_DOWNLOAD: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Update {
    pub version: Version,
    pub asset: service::Asset,
}

pub fn select_release(release: service::Release, current: &Version) -> Result<Option<Update>> {
    if release.draft || release.prerelease {
        return Ok(None);
    }
    let version = Version::parse(
        release
            .tag_name
            .strip_prefix('v')
            .unwrap_or(&release.tag_name),
    )
    .context("Der Updater-Release hat keinen gültigen SemVer-Versionstag.")?;
    if !version.pre.is_empty() || version <= *current {
        return Ok(None);
    }
    let asset = release
        .assets
        .into_iter()
        .find(|asset| asset.name == ASSET_NAME)
        .context("Der neue Updater-Release enthält keine SchILD3Updater.exe.")?;
    expected_digest(&asset)?;
    ensure!(
        asset.browser_download_url == format!("{DOWNLOAD_PREFIX}{}/{ASSET_NAME}", release.tag_name),
        "Die Updater-Datei stammt nicht aus dem erwarteten GitHub-Release."
    );
    Ok(Some(Update { version, asset }))
}

pub fn check() -> Result<Option<Update>> {
    let response = service::http_client()?
        .get(API)
        .timeout(Duration::from_secs(20))
        .send()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        bail!("Noch kein öffentlich erreichbarer Updater-Release vorhanden (GitHub 404). Private Repositorys werden nicht unterstützt.");
    }
    let response = response.error_for_status()?;
    let mut bytes = Vec::new();
    response.take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 1024 * 1024,
        "GitHub-Releaseantwort ist zu groß."
    );
    select_release(
        serde_json::from_slice(&bytes)?,
        &Version::parse(env!("CARGO_PKG_VERSION"))?,
    )
}

fn expected_digest(asset: &service::Asset) -> Result<String> {
    let digest = asset
        .digest
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .context("GitHub liefert keine SHA-256-Prüfsumme für die Updater-EXE.")?;
    ensure!(
        digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "Ungültige SHA-256-Prüfsumme für die Updater-EXE."
    );
    Ok(digest.to_ascii_lowercase())
}

pub fn download(update: &Update, progress: &impl Fn(String)) -> Result<tempfile::NamedTempFile> {
    let url = reqwest::Url::parse(&update.asset.browser_download_url)?;
    ensure!(
        url.scheme() == "https"
            && url.host_str() == Some("github.com")
            && url.username().is_empty()
            && url.password().is_none()
            && url
                .path()
                .starts_with("/cschmidtoptima/Schild3Updater/releases/download/")
            && url.path().ends_with("/SchILD3Updater.exe")
            && url.query().is_none()
            && url.fragment().is_none(),
        "Unerlaubte Downloadadresse für die Updater-EXE."
    );
    let expected = expected_digest(&update.asset)?;
    progress(format!(
        "Updater {} wird von GitHub heruntergeladen ...",
        update.version
    ));
    let mut response = service::http_client()?
        .get(url)
        .send()?
        .error_for_status()?;
    ensure!(
        response.content_length().unwrap_or(0) <= MAX_DOWNLOAD,
        "Updater-Download ist zu groß."
    );
    let mut file = tempfile::Builder::new()
        .prefix("schild3-updater-")
        .suffix(".exe")
        .tempfile()?;
    let mut hash = Sha256::new();
    let mut total = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = response.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        ensure!(total <= MAX_DOWNLOAD, "Updater-Download ist zu groß.");
        hash.update(&buffer[..count]);
        file.write_all(&buffer[..count])?;
    }
    file.as_file().sync_all()?;
    ensure!(
        format!("{:x}", hash.finalize()) == expected,
        "SHA-256-Prüfung der Updater-EXE fehlgeschlagen."
    );
    ensure!(
        version_info::read_version(file.path())? == update.version,
        "Die Dateiversion der Updater-EXE stimmt nicht mit dem Release überein."
    );
    Ok(file)
}

pub fn install(update: &Update, progress: &impl Fn(String)) -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let file = download(update, progress)?;
    progress("Updater-EXE wird ersetzt ...".into());
    self_replace::self_replace(file.path()).context(
        "Updater konnte nicht ersetzt werden. Schreibrechte und weitere laufende Instanzen prüfen.",
    )?;
    Ok(executable)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> service::Release {
        service::Release {
            tag_name: tag.into(),
            body: None,
            draft: false,
            prerelease: false,
            assets: vec![service::Asset {
                name: ASSET_NAME.into(),
                browser_download_url: format!("{DOWNLOAD_PREFIX}{tag}/{ASSET_NAME}"),
                digest: Some(format!("sha256:{}", "a".repeat(64))),
            }],
        }
    }

    #[test]
    fn only_newer_stable_versions_are_offered() {
        let current = Version::new(0, 2, 0);
        assert_eq!(
            select_release(release("v0.3.0"), &current)
                .unwrap()
                .unwrap()
                .version,
            Version::new(0, 3, 0)
        );
        for tag in ["v0.2.0", "v0.1.0", "v0.3.0-beta.1"] {
            assert!(select_release(release(tag), &current).unwrap().is_none());
        }
        let mut candidate = release("v0.3.0");
        candidate.draft = true;
        assert!(select_release(candidate.clone(), &current)
            .unwrap()
            .is_none());
        candidate.draft = false;
        candidate.prerelease = true;
        assert!(select_release(candidate, &current).unwrap().is_none());
    }

    #[test]
    fn wrong_assets_sources_and_missing_digests_are_rejected() {
        let current = Version::new(0, 2, 0);
        let mut candidate = release("v0.3.0");
        candidate.assets[0].digest = None;
        assert!(select_release(candidate, &current).is_err());
        let mut candidate = release("v0.3.0");
        candidate.assets[0].browser_download_url = "https://example.org/SchILD3Updater.exe".into();
        assert!(select_release(candidate, &current).is_err());
        let mut candidate = release("v0.3.0");
        candidate.assets[0].name = "other.exe".into();
        assert!(select_release(candidate, &current).is_err());
    }

    #[test]
    fn checksum_must_be_a_complete_sha256() {
        let mut asset = release("v0.3.0").assets.remove(0);
        for value in ["sha256:123", "sha512:abcd", "sha256:zzzz"] {
            asset.digest = Some(value.into());
            assert!(expected_digest(&asset).is_err());
        }
    }

    #[test]
    fn unsafe_download_urls_are_rejected_before_network_access() {
        for url in [
            "http://github.com/cschmidtoptima/Schild3Updater/releases/download/v0.3.0/SchILD3Updater.exe",
            "https://github.com.evil.example/cschmidtoptima/Schild3Updater/releases/download/v0.3.0/SchILD3Updater.exe",
            "https://github.com/other/project/releases/download/v0.3.0/SchILD3Updater.exe",
            "https://user:password@github.com/cschmidtoptima/Schild3Updater/releases/download/v0.3.0/SchILD3Updater.exe",
            "https://github.com/cschmidtoptima/Schild3Updater/releases/download/v0.3.0/SchILD3Updater.exe?unexpected=1",
        ] {
            let mut asset = release("v0.3.0").assets.remove(0);
            asset.browser_download_url = url.into();
            let result = download(&Update { version: Version::new(0, 3, 0), asset }, &|_| panic!("Download darf nicht beginnen"));
            assert!(result.is_err(), "{url}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn windows_replacement_preserves_neighbouring_files() {
        use std::{
            fs,
            io::{Seek, SeekFrom},
            os::windows::process::CommandExt,
            process::Command,
        };
        const CHILD_DIRECTORY: &str = "SCHILD3_ISOLATED_REPLACEMENT_TEST";
        const MARKER: &[u8] = b"schild3-self-replacement-test-marker";
        if let Some(directory) = std::env::var_os(CHILD_DIRECTORY) {
            let payload = PathBuf::from(directory).join("payload.exe");
            self_replace::self_replace(payload).unwrap();
            return;
        }
        let directory = tempfile::tempdir().unwrap();
        let executable = directory.path().join("UpdaterTest.exe");
        let payload = directory.path().join("payload.exe");
        fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
        fs::copy(&executable, &payload).unwrap();
        fs::OpenOptions::new()
            .append(true)
            .open(&payload)
            .unwrap()
            .write_all(MARKER)
            .unwrap();
        fs::write(directory.path().join("settings.json"), b"preserve settings").unwrap();
        fs::create_dir(directory.path().join("client")).unwrap();
        fs::write(
            directory.path().join("client/local.txt"),
            b"preserve client",
        )
        .unwrap();
        let output = Command::new(&executable)
            .args([
                "--exact",
                "self_update::tests::windows_replacement_preserves_neighbouring_files",
                "--nocapture",
            ])
            .env(CHILD_DIRECTORY, directory.path())
            .creation_flags(0x08000000)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::metadata(&executable).unwrap().len(),
            fs::metadata(&payload).unwrap().len()
        );
        let mut replaced = fs::File::open(&executable).unwrap();
        replaced
            .seek(SeekFrom::End(-(MARKER.len() as i64)))
            .unwrap();
        let mut marker = vec![0; MARKER.len()];
        replaced.read_exact(&mut marker).unwrap();
        assert_eq!(marker, MARKER);
        assert_eq!(
            fs::read(directory.path().join("settings.json")).unwrap(),
            b"preserve settings"
        );
        assert_eq!(
            fs::read(directory.path().join("client/local.txt")).unwrap(),
            b"preserve client"
        );
    }
}
