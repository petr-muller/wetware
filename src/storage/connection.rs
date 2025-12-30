/// Database connection management
use crate::errors::ThoughtError;
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// Get the default database path
pub fn default_db_path() -> PathBuf {
    PathBuf::from("wetware.db")
}

/// Get a database connection
///
/// Creates the database file if it doesn't exist.
pub fn get_connection(db_path: Option<&Path>) -> Result<Connection, ThoughtError> {
    let default_path = default_db_path();
    let path = db_path.unwrap_or(&default_path);
    let conn = Connection::open(path)?;

    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    Ok(conn)
}

/// Get an in-memory database connection (for testing)
pub fn get_memory_connection() -> Result<Connection, ThoughtError> {
    let conn = Connection::open_in_memory()?;
    conn.execute("PRAGMA foreign_keys = ON", [])?;
    Ok(conn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_memory_connection() {
        let conn = get_memory_connection().unwrap();
        // Verify foreign keys are enabled
        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }

    #[test]
    fn test_get_connection_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        assert!(!db_path.exists());

        let _conn = get_connection(Some(&db_path)).unwrap();

        assert!(db_path.exists());
    }

    #[test]
    fn test_get_connection_foreign_keys_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = get_connection(Some(&db_path)).unwrap();

        let fk_enabled: i32 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(fk_enabled, 1);
    }
}
