/// Database migration for networked notes feature
/// Creates tables: notes, entities, note_entities
use rusqlite::{Connection, Result};

pub fn migrate(conn: &Connection) -> Result<()> {
    // Create notes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create index on created_at for chronological ordering
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at)",
        [],
    )?;

    // Create entities table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
            canonical_name TEXT NOT NULL
        )",
        [],
    )?;

    // Create junction table for many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS note_entities (
            note_id INTEGER NOT NULL,
            entity_id INTEGER NOT NULL,
            PRIMARY KEY (note_id, entity_id),
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
            FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes for entity lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_note_entities_entity ON note_entities(entity_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_note_entities_note ON note_entities(note_id)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_creates_tables() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Verify tables exist
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
    fn test_migration_creates_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        // Verify indexes exist
        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(indexes.contains(&"idx_notes_created_at".to_string()));
        assert!(indexes.contains(&"idx_note_entities_entity".to_string()));
        assert!(indexes.contains(&"idx_note_entities_note".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migration twice - should not error
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }
}
