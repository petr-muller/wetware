use rusqlite::{Connection, params};
use anyhow::{Result, Context};
use std::path::Path;

use crate::model::Thought;
use super::Storage;

/// SQLite-based storage implementation
pub struct SqliteStorage {
    /// Path to the SQLite database file
    db_path: String,
}

impl SqliteStorage {
    /// Create a new SQLite storage at the given path
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            db_path: path.as_ref().to_string_lossy().to_string(),
        }
    }
    
    /// Get a database connection
    fn get_connection(&self) -> Result<Connection> {
        Connection::open(&self.db_path)
            .with_context(|| format!("Failed to open database at {}", self.db_path))
    }
}

impl Storage for SqliteStorage {
    fn init(&self) -> Result<()> {
        let conn = self.get_connection()?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS thoughts (
                id INTEGER PRIMARY KEY,
                content TEXT NOT NULL
            )",
            [],
        )?;
        
        Ok(())
    }
    
    fn save_thought(&self, content: &str) -> Result<Thought> {
        let conn = self.get_connection()?;
        
        conn.execute(
            "INSERT INTO thoughts (content) VALUES (?)",
            params![content],
        )?;
        
        let id = conn.last_insert_rowid() as usize;
        Ok(Thought::new(id, content.to_string()))
    }
    
    fn get_thoughts(&self) -> Result<Vec<Thought>> {
        let conn = self.get_connection()?;
        
        let mut stmt = conn.prepare("SELECT id, content FROM thoughts ORDER BY id")?;
        let thought_iter = stmt.query_map([], |row| {
            let id = row.get::<_, i64>(0)? as usize;
            let content = row.get::<_, String>(1)?;
            Ok(Thought::new(id, content))
        })?;
        
        let mut thoughts = Vec::new();
        for thought in thought_iter {
            thoughts.push(thought?);
        }
        
        Ok(thoughts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_temp_db() -> (SqliteStorage, NamedTempFile) {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = SqliteStorage::new(temp_file.path());
        storage.init().unwrap();
        (storage, temp_file)
    }

    #[test]
    fn test_init_creates_tables() {
        let (storage, _temp_file) = create_temp_db();
        
        // Verify the table was created
        let conn = storage.get_connection().unwrap();
        let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='thoughts'").unwrap();
        let rows = stmt.query_map([], |row| row.get::<_, String>(0)).unwrap();
        
        let table_names: Vec<String> = rows.map(|r| r.unwrap()).collect();
        assert_eq!(table_names, vec!["thoughts"]);
    }

    #[test]
    fn test_save_thought() {
        let (storage, _temp_file) = create_temp_db();
        
        let thought = storage.save_thought("Test thought").unwrap();
        
        assert_eq!(thought.id(), 1);
        assert_eq!(thought.content(), "Test thought");
        
        // Verify it was saved in the database
        let conn = storage.get_connection().unwrap();
        let mut stmt = conn.prepare("SELECT id, content FROM thoughts WHERE id = ?").unwrap();
        let rows = stmt.query_map([1], |row| {
            Ok((row.get::<_, i64>(0).unwrap(), row.get::<_, String>(1).unwrap()))
        }).unwrap();
        
        let results: Vec<(i64, String)> = rows.map(|r| r.unwrap()).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], (1, "Test thought".to_string()));
    }

    #[test]
    fn test_get_thoughts_empty() {
        let (storage, _temp_file) = create_temp_db();
        
        let thoughts = storage.get_thoughts().unwrap();
        
        assert!(thoughts.is_empty());
    }

    #[test]
    fn test_get_thoughts_multiple() {
        let (storage, _temp_file) = create_temp_db();
        
        storage.save_thought("First thought").unwrap();
        storage.save_thought("Second thought").unwrap();
        storage.save_thought("Third thought").unwrap();
        
        let thoughts = storage.get_thoughts().unwrap();
        
        assert_eq!(thoughts.len(), 3);
        assert_eq!(thoughts[0].id(), 1);
        assert_eq!(thoughts[0].content(), "First thought");
        assert_eq!(thoughts[1].id(), 2);
        assert_eq!(thoughts[1].content(), "Second thought");
        assert_eq!(thoughts[2].id(), 3);
        assert_eq!(thoughts[2].content(), "Third thought");
    }
}