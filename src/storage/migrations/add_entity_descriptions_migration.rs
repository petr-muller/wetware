// Migration: Add description column to entities table
// This migration adds support for multi-paragraph descriptions on entities

use crate::errors::ThoughtError;
use rusqlite::Connection;

/// Add description TEXT column to entities table (idempotent)
pub fn migrate_add_entity_descriptions(conn: &Connection) -> Result<(), ThoughtError> {
    // Check if column already exists
    let column_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('entities') WHERE name='description'",
        [],
        |row| row.get(0),
    )?;

    // Only add column if it doesn't exist
    if column_exists == 0 {
        conn.execute_batch("ALTER TABLE entities ADD COLUMN description TEXT;")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_migration_adds_description_column() {
        let temp_db = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_db.path()).unwrap();

        // Create initial entities table without description
        conn.execute_batch(
            "CREATE TABLE entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                canonical_name TEXT NOT NULL
            );",
        )
        .unwrap();

        // Run migration
        migrate_add_entity_descriptions(&conn).unwrap();

        // Verify column exists
        let column_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('entities') WHERE name='description'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(column_exists);
    }
}
