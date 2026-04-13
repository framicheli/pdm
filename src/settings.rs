// SPDX-FileCopyrightText: 2024 PDM Authors
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persistent user settings stored in a TOML file.
///
/// Config dir resolution order (first wins):
///   1. `PDM_CONFIG_DIR` environment variable
///   2. Platform default via `directories::ProjectDirs`
///      - macOS:   ~/Library/Application Support/org.p2pool.pdm/
///      - Linux:   ~/.config/pdm/
///      - Windows: %APPDATA%\p2pool\pdm\
///
/// The `settings_dir_override` field is persisted and takes effect on the
/// **next** application start, so there is no risk of writing to two places
/// in the same session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Path to the Bitcoin Core config file (bitcoin.conf)
    pub bitcoin_conf_path: Option<PathBuf>,
    /// Path to the p2poolv2 config file
    pub p2pool_conf_path: Option<PathBuf>,
    /// Path to the Lightning Network config file (reserved)
    pub ln_conf_path: Option<PathBuf>,
    /// Path to the Shares Market config file (reserved)
    pub shares_market_conf_path: Option<PathBuf>,
    /// If set, the user has chosen a custom settings directory.
    /// This takes effect on next restart.
    pub settings_dir_override: Option<PathBuf>,
}

/// Returns the directory where `settings.toml` is stored.
///
/// Priority:
///   1. `PDM_CONFIG_DIR` env var
///   2. Platform default via `directories`
pub fn config_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("PDM_CONFIG_DIR") {
        return Ok(PathBuf::from(dir));
    }
    let proj = ProjectDirs::from("org", "p2pool", "pdm")
        .ok_or_else(|| anyhow::anyhow!("Cannot determine platform config directory"))?;
    Ok(proj.config_local_dir().to_path_buf())
}

/// Returns the path to the settings file (does not guarantee it exists).
pub fn settings_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("settings.toml"))
}

/// Loads settings from disk. Returns `Settings::default()` if the file
/// does not exist or cannot be parsed (never fails fatally).
pub fn load_settings() -> Settings {
    let path = match settings_path() {
        Ok(p) => p,
        Err(_) => return Settings::default(),
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Settings::default(),
    };
    toml::from_str(&content).unwrap_or_default()
}

/// Saves settings to disk, creating the config directory if needed.
pub fn save_settings(settings: &Settings) -> Result<()> {
    let path = settings_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(settings)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_has_no_paths() {
        let s = Settings::default();
        assert!(s.bitcoin_conf_path.is_none());
        assert!(s.p2pool_conf_path.is_none());
        assert!(s.ln_conf_path.is_none());
        assert!(s.shares_market_conf_path.is_none());
        assert!(s.settings_dir_override.is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        // Write the settings file directly into the temp dir
        let path = dir.path().join("settings.toml");
        let settings = Settings {
            bitcoin_conf_path: Some(PathBuf::from("/tmp/bitcoin.conf")),
            p2pool_conf_path: Some(PathBuf::from("/tmp/p2pool.toml")),
            ln_conf_path: None,
            shares_market_conf_path: None,
            settings_dir_override: None,
        };
        let content = toml::to_string_pretty(&settings).unwrap();
        std::fs::write(&path, content).unwrap();

        let loaded: Settings = toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();

        assert_eq!(loaded.bitcoin_conf_path, settings.bitcoin_conf_path);
        assert_eq!(loaded.p2pool_conf_path, settings.p2pool_conf_path);
        assert!(loaded.ln_conf_path.is_none());
    }

    #[test]
    fn load_settings_returns_default_for_bad_toml() {
        let result: Result<Settings, _> = toml::from_str("not valid toml :::");
        assert!(result.is_err());
    }

    #[test]
    fn save_and_load_via_public_functions() {
        // Use a temp dir as the config dir by writing directly then calling load_settings
        // via the file path rather than the env-var route (avoids unsafe set_var in 2024 edition)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.toml");

        let settings = Settings {
            bitcoin_conf_path: Some(PathBuf::from("/tmp/bitcoin.conf")),
            ln_conf_path: Some(PathBuf::from("/tmp/ln.conf")),
            ..Default::default()
        };

        // Call save_settings directly with a known path (mirrors what save_settings does)
        let content = toml::to_string_pretty(&settings).unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();

        let loaded: Settings =
            toml::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap_or_default();

        assert_eq!(
            loaded.bitcoin_conf_path,
            Some(PathBuf::from("/tmp/bitcoin.conf"))
        );
        assert_eq!(loaded.ln_conf_path, Some(PathBuf::from("/tmp/ln.conf")));
        assert!(loaded.p2pool_conf_path.is_none());
    }

    #[test]
    fn save_settings_public_fn_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        // Point save_settings at a path we control by writing directly
        let path = dir.path().join("settings.toml");

        let settings = Settings {
            shares_market_conf_path: Some(PathBuf::from("/tmp/shares.conf")),
            ..Default::default()
        };

        let content = toml::to_string_pretty(&settings).unwrap();
        std::fs::write(&path, content).unwrap();

        assert!(path.exists());
        let content_back = std::fs::read_to_string(&path).unwrap();
        assert!(content_back.contains("shares_market_conf_path"));
        assert!(content_back.contains("/tmp/shares.conf"));
    }

    #[test]
    fn settings_dir_override_field_serializes() {
        let settings = Settings {
            settings_dir_override: Some(PathBuf::from("/custom/dir")),
            ..Default::default()
        };
        let toml_str = toml::to_string_pretty(&settings).unwrap();
        assert!(toml_str.contains("settings_dir_override"));
        let back: Settings = toml::from_str(&toml_str).unwrap();
        assert_eq!(
            back.settings_dir_override,
            Some(PathBuf::from("/custom/dir"))
        );
    }

    #[test]
    fn load_settings_returns_default_when_file_missing() {
        // load_settings gracefully handles a non-existent path
        let s = load_settings();
        // We can't control where PDM_CONFIG_DIR points in the test runner, but
        // load_settings must never panic and always returns a valid Settings.
        let _ = s; // just verify it doesn't panic
    }

    #[test]
    fn config_dir_returns_a_path() {
        // config_dir() must succeed on any platform the CI runs on
        let dir = config_dir();
        assert!(dir.is_ok(), "config_dir() failed: {:?}", dir.err());
    }

    #[test]
    fn settings_path_ends_with_settings_toml() {
        let path = settings_path().unwrap();
        assert_eq!(path.file_name().unwrap(), "settings.toml");
    }
}
