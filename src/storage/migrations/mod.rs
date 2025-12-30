/// Database migrations module
pub mod networked_notes_migration;

use crate::errors::NoteError;
use rusqlite::Connection;

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<(), NoteError> {
    // Run migration 001: networked notes
    networked_notes_migration::migrate(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_run_migrations() {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();

        // Verify all tables were created
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"notes".to_string()));
        assert!(tables.contains(&"entities".to_string()));
        assert!(tables.contains(&"note_entities".to_string()));
    }

    #[test]
    fn test_run_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice - should not error
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
    }
}
