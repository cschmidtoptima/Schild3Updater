use regex::Regex;
use semver::Version;

pub mod config;
pub mod install;
pub mod self_update;
pub mod service;
pub mod version_info;

pub fn compatible_server(notes: &str) -> Option<Version> {
    let pattern = Regex::new(
        r"(?im)^\s*[-*]?\s*Lauffähig mit (?:(?:Version )?(?:dem )?SVWS-Server\s*(?:Version\s*)?v?(\d+\.\d+\.\d+)|der Version (\d+\.\d+\.\d+) des SVWS-Servers)\.?\s*$",
    ).unwrap();
    let versions: Vec<_> = pattern
        .captures_iter(notes)
        .filter_map(|capture| {
            Version::parse(capture.get(1).or_else(|| capture.get(2))?.as_str()).ok()
        })
        .collect();
    match versions.as_slice() {
        [version] => Some(version.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_compatibility_only() {
        for text in [
            "Lauffähig mit dem SVWS-Server 1.4.1.",
            "- Lauffähig mit Version SVWS-Server 1.2.2",
            "Lauffähig mit SVWS-Server Version 1.1.1",
            "Lauffähig mit SVWS-Server v1.1.1",
            "Lauffähig mit der Version 1.5.1 des SVWS-Servers.",
        ] {
            assert!(compatible_server(text).is_some(), "{text}");
        }
        assert!(compatible_server("Anpassung an SVWS-Server 1.4.1").is_none());
        assert!(compatible_server("Lauffähig mit SVWS-Server 1.1.1 oder neuer").is_none());
        assert!(compatible_server(
            "Lauffähig mit SVWS-Server 1.1.1\nLauffähig mit SVWS-Server 1.2.2"
        )
        .is_none());
    }
}
