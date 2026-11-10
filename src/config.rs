use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub server_url: String,
    pub client_dir: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_url: String::new(),
            client_dir: String::new(),
        }
    }
}

pub fn config_path() -> Result<PathBuf> {
    let executable =
        std::env::current_exe().context("Updater-Pfad konnte nicht ermittelt werden.")?;
    Ok(executable.with_file_name("settings.json"))
}

pub fn load() -> Result<Settings> {
    let path = config_path()?;
    match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).context("Einstellungen sind ungültig."),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(error) => Err(error).context("Einstellungen konnten nicht gelesen werden."),
    }
}

pub fn save(settings: &Settings) -> Result<()> {
    let path = config_path()?;
    let parent = path.parent().context("Konfigurationspfad ist ungültig.")?;
    std::fs::create_dir_all(parent)?;
    let temporary = tempfile::NamedTempFile::new_in(parent)?;
    serde_json::to_writer_pretty(temporary.as_file(), settings)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .context("Einstellungen konnten nicht gespeichert werden.")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_next_to_executable() {
        let executable = std::env::current_exe().unwrap();
        let expected = executable.parent().unwrap().join("settings.json");
        assert_eq!(config_path().unwrap(), expected);
    }
}
