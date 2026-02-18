/// Database migrations module
pub mod add_entity_descriptions_migration;
pub mod networked_notes_migration;

use crate::errors::ThoughtError;
use rusqlite::Connection;

/// Run all database migrations
pub fn run_migrations(conn: &Connection) -> Result<(), ThoughtError> {
    // Run migration 001: networked notes
    networked_notes_migration::migrate(conn)?;

    // Run migration 002: entity descriptions
    add_entity_descriptions_migration::migrate_add_entity_descriptions(conn)?;

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

        assert!(tables.contains(&"thoughts".to_string()));
        assert!(tables.contains(&"entities".to_string()));
        assert!(tables.contains(&"thought_entities".to_string()));

        // Verify description column exists in entities table
        let description_col_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('entities') WHERE name='description'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(
            description_col_exists,
            "description column should exist in entities table"
        );
    }

    #[test]
    fn test_run_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migrations twice - should not error
        run_migrations(&conn).unwrap();
        run_migrations(&conn).unwrap();
    }
}
