/// Database migration for directed entity relations (parent/child)
/// Creates table: entity_relations
use rusqlite::{Connection, Result};

pub fn migrate(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entity_relations (
            child_id INTEGER NOT NULL,
            parent_id INTEGER NOT NULL,
            PRIMARY KEY (child_id, parent_id),
            FOREIGN KEY (child_id) REFERENCES entities(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES entities(id) ON DELETE CASCADE,
            CHECK (child_id != parent_id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_entity_relations_parent ON entity_relations(parent_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_entity_relations_child ON entity_relations(child_id)",
        [],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_creates_table() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(tables.contains(&"entity_relations".to_string()));
    }

    #[test]
    fn test_migration_creates_indexes() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();

        let indexes: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert!(indexes.contains(&"idx_entity_relations_parent".to_string()));
        assert!(indexes.contains(&"idx_entity_relations_child".to_string()));
    }

    #[test]
    fn test_migration_idempotent() {
        let conn = Connection::open_in_memory().unwrap();

        migrate(&conn).unwrap();
        migrate(&conn).unwrap();
    }
}
