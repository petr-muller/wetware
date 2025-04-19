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
    
    fn get_thought(&self, id: usize) -> Result<Option<Thought>> {
        let conn = self.get_connection()?;
        
        let mut stmt = conn.prepare("SELECT content FROM thoughts WHERE id = ?")?;
        let mut rows = stmt.query(params![id as i64])?;
        
        if let Some(row) = rows.next()? {
            let content = row.get::<_, String>(0)?;
            Ok(Some(Thought::new(id, content)))
        } else {
            Ok(None)
        }
    }
}