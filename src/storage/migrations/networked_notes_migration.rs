/// Database migration for networked notes feature
/// Creates tables: thoughts, entities, thought_entities
use rusqlite::{Connection, Result};

pub fn migrate(conn: &Connection) -> Result<()> {
    // Create thoughts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thoughts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // Create index on created_at for chronological ordering
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thoughts_created_at ON thoughts(created_at)",
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
        "CREATE TABLE IF NOT EXISTS thought_entities (
            thought_id INTEGER NOT NULL,
            entity_id INTEGER NOT NULL,
            PRIMARY KEY (thought_id, entity_id),
            FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
            FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Create indexes for entity lookups
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thought_entities_entity ON thought_entities(entity_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_thought_entities_thought ON thought_entities(thought_id)",
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

        assert!(tables.contains(&"thoughts".to_string()));
        assert!(tables.contains(&"entities".to_string()));
        assert!(tables.contains(&"thought_entities".to_string()));
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

        assert!(indexes.contains(&"idx_thoughts_created_at".to_string()));
        assert!(indexes.contains(&"idx_thought_entities_entity".to_string()));
        assert!(indexes.contains(&"idx_thought_entities_thought".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        // Run migration twice - should not error
        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }
}
