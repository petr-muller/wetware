/// Configuration file management
use crate::errors::ThoughtError;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Wetware configuration
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Configuration format version for future migrations
    pub version: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { version: 1 }
    }
}

const CONFIG_FILENAME: &str = "config.toml";

/// Load configuration from the data directory.
///
/// Returns default config if the file doesn't exist.
/// Returns an error if the file exists but is malformed.
pub fn load_config(data_dir: &Path) -> Result<Config, ThoughtError> {
    let config_path = data_dir.join(CONFIG_FILENAME);

    if !config_path.exists() {
        return Ok(Config::default());
    }

    let contents = std::fs::read_to_string(&config_path)?;
    toml::from_str(&contents).map_err(|e| ThoughtError::InvalidInput(format!("Malformed config file: {e}")))
}

/// Save configuration to the data directory.
pub fn save_config(data_dir: &Path, config: &Config) -> Result<(), ThoughtError> {
    let config_path = data_dir.join(CONFIG_FILENAME);
    let contents =
        toml::to_string_pretty(config).map_err(|e| ThoughtError::InvalidInput(format!("Failed to serialize config: {e}")))?;
    std::fs::write(&config_path, contents)?;
    Ok(())
}

/// Ensure a config file exists in the data directory, creating a default one if missing.
pub fn ensure_config(data_dir: &Path) -> Result<Config, ThoughtError> {
    let config = load_config(data_dir)?;
    let config_path = data_dir.join(CONFIG_FILENAME);
    if !config_path.exists() {
        save_config(data_dir, &config)?;
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.version, 1);
    }

    #[test]
    fn test_load_config_missing_file_returns_default() {
        let temp = TempDir::new().unwrap();
        let config = load_config(temp.path()).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp = TempDir::new().unwrap();
        let config = Config { version: 2 };
        save_config(temp.path(), &config).unwrap();
        let loaded = load_config(temp.path()).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn test_load_malformed_config_errors() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("config.toml"), "not valid toml [[[").unwrap();
        let result = load_config(temp.path());
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Malformed config file"));
    }

    #[test]
    fn test_ensure_config_creates_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        assert!(!config_path.exists());
        let config = ensure_config(temp.path()).unwrap();
        assert_eq!(config, Config::default());
        assert!(config_path.exists());
    }

    #[test]
    fn test_ensure_config_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let custom = Config { version: 5 };
        save_config(temp.path(), &custom).unwrap();
        let config = ensure_config(temp.path()).unwrap();
        assert_eq!(config, custom);
    }

    #[test]
    fn test_config_roundtrip_toml() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }
}
