use anyhow::{bail, Context, Result};
use fs2::FileExt;
use semver::Version;
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const MAX_EXPANDED: u64 = 2 * 1024 * 1024 * 1024;

fn safe_path(name: &str) -> Result<PathBuf> {
    let normalized = name.replace('\\', "/");
    let path = PathBuf::from(&normalized);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || normalized.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            part.is_empty()
                || part.ends_with(['.', ' '])
                || part
                    .chars()
                    .any(|character| character < ' ' || "<>:\"|?*".contains(character))
                || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
                || part.to_ascii_lowercase().starts_with(".schild3-updater")
        })
    {
        bail!("Unsicherer Archivpfad: {name}");
    }
    Ok(path)
}

pub fn extract(archive: &Path, destination: &Path) -> Result<()> {
    let mut zip = zip::ZipArchive::new(File::open(archive)?)?;
    if zip.len() > 20000 {
        bail!("Zu viele Dateien im Updatepaket.");
    }
    let mut seen = HashSet::new();
    let mut total = 0u64;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index)?;
        let name = entry.name().trim_end_matches('/');
        let relative = safe_path(name)?;
        if !seen.insert(relative.to_string_lossy().to_lowercase()) {
            bail!("Doppelter Archivpfad: {name}");
        }
        if entry.is_symlink() {
            bail!("Symbolische Links sind im Update nicht erlaubt.");
        }
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(target)?;
            continue;
        }
        total = total
            .checked_add(entry.size())
            .context("Archivgröße ungültig.")?;
        if total > MAX_EXPANDED {
            bail!("Entpacktes Update ist zu groß.");
        }
        fs::create_dir_all(target.parent().context("Ungültiger Archivpfad.")?)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        let copied = std::io::copy(&mut (&mut entry).take(MAX_EXPANDED + 1), &mut output)?;
        if copied != entry.size() {
            bail!("Dateigröße im Archiv stimmt nicht.");
        }
        output.sync_all()?;
    }
    Ok(())
}

fn ordinary(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            bail!("Verknüpfung/Junction nicht erlaubt: {}", path.display());
        }
    }
    if metadata.file_type().is_symlink() {
        bail!("Verknüpfung nicht erlaubt: {}", path.display());
    }
    Ok(())
}

fn collect(root: &Path, relative: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root.join(relative))? {
        let entry = entry?;
        ordinary(&entry.path())?;
        let path = relative.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            collect(root, &path, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(path);
        } else {
            bail!("Nicht unterstützter Dateityp im Update.");
        }
    }
    Ok(())
}

fn atomic_copy(source: &Path, destination: &Path) -> Result<()> {
    let parent = destination.parent().context("Zielpfad ungültig.")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    std::io::copy(&mut File::open(source)?, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    temporary.persist(destination).with_context(|| {
        format!(
            "Datei gesperrt oder nicht beschreibbar: {}",
            destination.display()
        )
    })?;
    Ok(())
}

pub fn ensure_no_interrupted_update(target: &Path) -> Result<()> {
    let backups = target.join(".schild3-updater-backups");
    if !backups.exists() {
        return Ok(());
    }
    ordinary(&backups)?;
    for entry in fs::read_dir(backups)? {
        let entry = entry?;
        ordinary(&entry.path())?;
        let backup = entry.path();
        if backup.is_dir()
            && backup.join("manifest.json").exists()
            && !backup.join("completed").exists()
            && !backup.join("rolled-back").exists()
        {
            bail!("Ein unterbrochenes Update wurde gefunden. Vor einem weiteren Update die Sicherung manuell prüfen/wiederherstellen: {}", backup.display());
        }
    }
    Ok(())
}

pub fn apply(
    staging: &Path,
    target: &Path,
    installed: &Version,
    expected: &Version,
    progress: &impl Fn(String),
) -> Result<PathBuf> {
    ordinary(target)?;
    let lock_path = target.join(".schild3-updater.lock");
    if lock_path.exists() {
        ordinary(&lock_path)?;
    }
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)?;
    lock.try_lock_exclusive()
        .context("Ein anderer Updater verwendet diesen Clientordner.")?;
    ensure_no_interrupted_update(target)?;
    if crate::version_info::client_version(target)? != *installed {
        bail!("Clientversion hat sich geändert. Bitte erneut prüfen.");
    }
    let result = apply_files(staging, target, &|_| Ok(()), &|| {
        if crate::version_info::client_version(target)? != *expected {
            bail!("Installierte Version stimmt nach dem Update nicht.");
        }
        Ok(())
    })?;
    progress(format!("Sicherung: {}", result.display()));
    Ok(result)
}

fn apply_files(
    staging: &Path,
    target: &Path,
    before_copy: &impl Fn(usize) -> Result<()>,
    verify: &impl Fn() -> Result<()>,
) -> Result<PathBuf> {
    let mut files = Vec::new();
    collect(staging, Path::new(""), &mut files)?;
    files.sort();
    if files.is_empty() {
        bail!("Leeres Updatepaket.");
    }
    for relative in &files {
        safe_path(&relative.to_string_lossy())?;
        let mut current = target.to_path_buf();
        for part in relative.components() {
            current.push(part);
            match fs::symlink_metadata(&current) {
                Ok(_) => ordinary(&current)?,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
                Err(error) => return Err(error.into()),
            }
        }
        let destination = target.join(relative);
        if destination.exists() {
            let mut options = OpenOptions::new();
            options.read(true).write(true);
            #[cfg(windows)]
            {
                use std::os::windows::fs::OpenOptionsExt;
                options.share_mode(0);
            }
            options.open(&destination).with_context(|| {
                format!(
                    "Bitte SchILD3 schließen. Datei gesperrt oder schreibgeschützt: {}",
                    destination.display()
                )
            })?;
        }
    }
    let backup_root = target.join(".schild3-updater-backups");
    if backup_root.exists() {
        ordinary(&backup_root)?;
    }
    fs::create_dir_all(&backup_root)?;
    let backup = tempfile::Builder::new()
        .prefix("update-")
        .tempdir_in(backup_root)?
        .keep();
    let originals = backup.join("originals");
    let mut existing = HashSet::new();
    for relative in &files {
        let source = target.join(relative);
        if source.exists() {
            let saved = originals.join(relative);
            fs::create_dir_all(saved.parent().context("Sicherungspfad ungültig.")?)?;
            atomic_copy(&source, &saved)?;
            existing.insert(relative.clone());
        }
    }
    let manifest = serde_json::json!({"files": files, "previously_present": existing});
    let mut journal = File::create(backup.join("manifest.json"))?;
    journal.write_all(serde_json::to_string_pretty(&manifest)?.as_bytes())?;
    journal.sync_all()?;
    let mut changed = Vec::new();
    let mut created_dirs = Vec::new();
    let result = (|| -> Result<()> {
        for (index, relative) in files.iter().enumerate() {
            before_copy(index)?;
            let destination = target.join(relative);
            let mut missing = Vec::new();
            let mut parent = destination.parent().context("Zielpfad ungültig.")?;
            while parent != target && !parent.exists() {
                missing.push(parent.to_path_buf());
                parent = parent
                    .parent()
                    .context("Zielpfad liegt außerhalb des Clientordners.")?;
            }
            for directory in missing.into_iter().rev() {
                fs::create_dir(&directory)?;
                created_dirs.push(directory);
            }
            atomic_copy(&staging.join(relative), &destination)?;
            changed.push(relative.clone());
        }
        verify()?;
        let mut completed = File::create(backup.join("completed"))?;
        completed.write_all(b"ok")?;
        completed.sync_all()?;
        Ok(())
    })();
    if let Err(error) = result {
        let mut failures = Vec::new();
        for relative in changed.iter().rev() {
            let destination = target.join(relative);
            let restore = if existing.contains(relative) {
                atomic_copy(&originals.join(relative), &destination)
            } else {
                fs::remove_file(&destination).map_err(anyhow::Error::from)
            };
            if let Err(problem) = restore {
                failures.push(format!("{}: {problem:#}", relative.display()));
            }
        }
        for directory in created_dirs.iter().rev() {
            let _ = fs::remove_dir(directory);
        }
        if failures.is_empty() {
            let _ = fs::remove_file(backup.join("completed"));
            let mut marker = File::create(backup.join("rolled-back")).with_context(|| format!("Rücksetzung erfolgt, aber Status konnte nicht gespeichert werden. Sicherung: {}", backup.display()))?;
            marker.write_all(b"ok")?;
            marker.sync_all()?;
            bail!("Update fehlgeschlagen; bisherige Dateien wiederhergestellt. Sicherung: {}. Ursache: {error:#}", backup.display());
        }
        bail!("Update fehlgeschlagen: {error:#}. Rücksetzung unvollständig: {}. Sicherung für manuelle Wiederherstellung: {}", failures.join("; "), backup.display());
    }
    Ok(backup)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unsafe_paths() {
        for path in [
            "../escape",
            "sub/../escape",
            "C:/escape",
            "//host/share/file",
            "a:stream",
            "CON.txt",
            "sub/NUL",
            "sub/file.",
            "LPT1",
            ".schild3-updater.lock",
            "a//b",
        ] {
            assert!(safe_path(path).is_err(), "{path}");
        }
        assert_eq!(
            safe_path("Vorlagen\\Report.rtm").unwrap(),
            PathBuf::from("Vorlagen/Report.rtm")
        );
    }

    #[test]
    fn preserves_non_package_files_and_backs_up_overwrites() {
        let target = tempfile::tempdir().unwrap();
        let stage = tempfile::tempdir().unwrap();
        fs::create_dir(target.path().join("local")).unwrap();
        fs::write(target.path().join("local/settings.ini"), "private").unwrap();
        fs::write(target.path().join("client.exe"), "old").unwrap();
        fs::write(stage.path().join("client.exe"), "new").unwrap();
        fs::create_dir(stage.path().join("data")).unwrap();
        fs::write(stage.path().join("data/new.dll"), "new dll").unwrap();
        let backup = apply_files(stage.path(), target.path(), &|_| Ok(()), &|| Ok(())).unwrap();
        assert_eq!(
            fs::read(target.path().join("local/settings.ini")).unwrap(),
            b"private"
        );
        assert_eq!(fs::read(target.path().join("client.exe")).unwrap(), b"new");
        assert_eq!(
            fs::read(backup.join("originals/client.exe")).unwrap(),
            b"old"
        );
        assert_eq!(
            fs::read(target.path().join("data/new.dll")).unwrap(),
            b"new dll"
        );
    }

    #[test]
    fn rolls_back_partial_update() {
        let target = tempfile::tempdir().unwrap();
        let stage = tempfile::tempdir().unwrap();
        fs::write(target.path().join("a.exe"), "old").unwrap();
        fs::write(stage.path().join("a.exe"), "new").unwrap();
        fs::create_dir(stage.path().join("b")).unwrap();
        fs::write(stage.path().join("b/new.dll"), "new dll").unwrap();
        fs::write(stage.path().join("c.dll"), "later").unwrap();
        let result = apply_files(
            stage.path(),
            target.path(),
            &|index| {
                if index == 2 {
                    bail!("simulated write error");
                }
                Ok(())
            },
            &|| Ok(()),
        );
        assert!(result.is_err());
        assert_eq!(fs::read(target.path().join("a.exe")).unwrap(), b"old");
        assert!(!target.path().join("b").exists());
        assert!(!target.path().join("c.dll").exists());
        ensure_no_interrupted_update(target.path()).unwrap();
    }

    #[test]
    fn refuses_unfinished_transaction() {
        let target = tempfile::tempdir().unwrap();
        let backup = target.path().join(".schild3-updater-backups/update-test");
        fs::create_dir_all(&backup).unwrap();
        fs::write(backup.join("manifest.json"), "{}").unwrap();
        assert!(ensure_no_interrupted_update(target.path()).is_err());
        fs::write(backup.join("completed"), "ok").unwrap();
        ensure_no_interrupted_update(target.path()).unwrap();
    }

    #[test]
    fn verification_failure_restores_old_files() {
        let target = tempfile::tempdir().unwrap();
        let stage = tempfile::tempdir().unwrap();
        fs::write(target.path().join("client.exe"), "old").unwrap();
        fs::write(stage.path().join("client.exe"), "new").unwrap();
        assert!(
            apply_files(stage.path(), target.path(), &|_| Ok(()), &|| anyhow::bail!(
                "bad version"
            ))
            .is_err()
        );
        assert_eq!(fs::read(target.path().join("client.exe")).unwrap(), b"old");
    }

    #[test]
    fn extracts_zip_and_rejects_traversal() {
        use zip::write::SimpleFileOptions;
        for name in ["sub/file.txt", "../escape.txt"] {
            let archive = tempfile::NamedTempFile::new().unwrap();
            let mut writer = zip::ZipWriter::new(archive.reopen().unwrap());
            writer
                .start_file(name, SimpleFileOptions::default())
                .unwrap();
            writer.write_all(b"content").unwrap();
            writer.finish().unwrap();
            let destination = tempfile::tempdir().unwrap();
            let result = extract(archive.path(), destination.path());
            assert_eq!(result.is_ok(), name == "sub/file.txt");
        }
    }

    #[cfg(windows)]
    #[test]
    fn locked_destination_keeps_original() {
        use std::os::windows::fs::OpenOptionsExt;
        let target = tempfile::tempdir().unwrap();
        let stage = tempfile::tempdir().unwrap();
        let file = target.path().join("client.exe");
        fs::write(&file, "old").unwrap();
        fs::write(stage.path().join("client.exe"), "new").unwrap();
        let guard = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&file)
            .unwrap();
        assert!(apply_files(stage.path(), target.path(), &|_| Ok(()), &|| Ok(())).is_err());
        assert_eq!(fs::read(file).unwrap(), b"old");
        drop(guard);
    }
}
