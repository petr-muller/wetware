/// Data directory resolution and initialization
use crate::errors::ThoughtError;
use std::path::{Path, PathBuf};

/// Resolve the wetware data directory.
///
/// If `override_path` is provided, uses that directly.
/// In release builds, falls back to `dirs::data_dir()/wetware`.
/// In debug builds, panics if no override is provided — this prevents
/// development and testing from ever touching the user's production data.
pub fn resolve_data_dir(override_path: Option<&Path>) -> Result<PathBuf, ThoughtError> {
    if let Some(path) = override_path {
        return Ok(path.to_path_buf());
    }

    #[cfg(debug_assertions)]
    {
        panic!(
            "debug builds require an explicit data directory override \
             (set WETWARE_DATA_DIR). \
             This prevents accidentally touching production data."
        );
    }

    #[cfg(not(debug_assertions))]
    {
        dirs::data_dir()
            .map(|d| d.join("wetware"))
            .ok_or_else(|| {
                ThoughtError::InvalidInput(
                    "Could not determine data directory: $HOME or platform equivalent is not set"
                        .to_string(),
                )
            })
    }
}

/// Ensure the data directory exists, creating it if necessary.
pub fn ensure_data_dir(path: &Path) -> Result<(), ThoughtError> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Get the default database path within a data directory.
pub fn default_db_path_in(data_dir: &Path) -> PathBuf {
    data_dir.join("default.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_data_dir_with_override() {
        let temp = TempDir::new().unwrap();
        let result = resolve_data_dir(Some(temp.path())).unwrap();
        assert_eq!(result, temp.path());
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "debug builds require an explicit data directory override")]
    fn test_resolve_data_dir_panics_in_debug_without_override() {
        let _ = resolve_data_dir(None);
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn test_resolve_data_dir_without_override_does_not_panic_in_release() {
        let result = std::panic::catch_unwind(|| resolve_data_dir(None));
        assert!(result.is_ok());
    }

    #[test]
    fn test_ensure_data_dir_creates_directory() {
        let temp = TempDir::new().unwrap();
        let nested = temp.path().join("a").join("b").join("c");
        assert!(!nested.exists());
        ensure_data_dir(&nested).unwrap();
        assert!(nested.exists());
        assert!(nested.is_dir());
    }

    #[test]
    fn test_ensure_data_dir_existing_directory() {
        let temp = TempDir::new().unwrap();
        ensure_data_dir(temp.path()).unwrap();
        assert!(temp.path().exists());
    }

    #[test]
    fn test_default_db_path_in() {
        let dir = Path::new("/some/data/dir");
        assert_eq!(default_db_path_in(dir), PathBuf::from("/some/data/dir/default.db"));
    }
}
