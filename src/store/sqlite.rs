use crate::model::entities::Entity;
use crate::model::thoughts::{Fragment, RawThought, Thought};
use chrono::NaiveDate;
use indexmap::IndexMap;
use rusqlite::{params, params_from_iter, Connection};

pub struct Store {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct SqliteStoreError {
    pub message: String,
}

impl std::fmt::Display for SqliteStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SQLite store error: {}", self.message)
    }
}

impl std::error::Error for SqliteStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl From<rusqlite::Error> for SqliteStoreError {
    fn from(rusqlite_err: rusqlite::Error) -> Self {
        SqliteStoreError {
            message: rusqlite_err.to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, SqliteStoreError>;

pub fn open(db: String) -> Result<Store> {
    let conn = Connection::open(db)?;
    Ok(Store { conn })
}

impl Store {
    pub fn get_entities(&self) -> Result<Vec<Entity>> {
        let stmt = "SELECT name FROM entities ORDER BY name";
        let mut stmt = self.conn.prepare(stmt)?;
        let rows = stmt.query_map(params![], |row| {
            Ok(Entity {
                raw: row.get(0)?,
            })
        })?;

        let mut entities = vec![];
        for entity in rows {
            entities.push(entity?);
        };

        Ok(entities)
    }
    pub fn get_thought(&self, thought_id: u32) -> Result<RawThought> {
        let mut stmt = self.conn.prepare(
            "SELECT thought, datetime FROM thoughts WHERE id = ?1"
        )?;
        let rows = stmt.query_map(params![thought_id], |row| {
            let raw: String = row.get(0)?;
            let added: NaiveDate = row.get(1)?;
            Ok(RawThought::from_store(raw, added))
        })?;

        let mut thoughts = vec![];
        for row in rows {
            thoughts.push(row?);
        }

        match thoughts.len() {
            0 => { Err(SqliteStoreError { message: format!("No thought with id {thought_id}") }) }
            1 => Ok(thoughts[0].clone()),
            _ => Err(SqliteStoreError { message: format!("BUG: Multiple thoughts with id {thought_id}") }),
        }
    }

    pub fn get_thoughts(&self, entity: Option<String>) -> Result<IndexMap<u32, RawThought>> {
        let mut stmt_lines = vec!["SELECT thoughts.id, thought, datetime FROM thoughts"];
        let mut params = vec![];

        if let Some(entity) = entity {
            stmt_lines.append(&mut vec![
                "JOIN thoughts_entities ON thoughts.id = thoughts_entities.thought_id",
                "JOIN entities ON thoughts_entities.entity_id = entities.id",
                "WHERE entities.name = ?1"]);
            params.push(entity)
        }

        stmt_lines.push("ORDER BY datetime");

        let mut stmt = self.conn.prepare(stmt_lines.join("\n").as_str())?;

        let rows = stmt.query_map(params_from_iter(params), |row| {
            let id: u32 = row.get(0)?;
            let raw: String = row.get(1)?;
            let added: NaiveDate = row.get(2)?;

            Ok((id, RawThought::from_store(raw, added)))
        })?;

        let mut thoughts = IndexMap::new();
        for item in rows {
            let (id, thought) = item?;
            thoughts.insert(id, thought);
        }

        Ok(thoughts)
    }

    fn make_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS thoughts (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought TEXT NOT NULL,
                    datetime INTEGER NOT NULL
                    )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEST NOT NULL UNIQUE
                    )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS thoughts_entities (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought_id INTEGER,
                    entity_id INTEGER,
                    FOREIGN KEY(thought_id) REFERENCES thoughts(id),
                    FOREIGN KEY(entity_id) REFERENCES entities(id),
                    UNIQUE(thought_id, entity_id)
                    )",
            (),
        )?;

        Ok(())
    }

    pub fn add_thought(&self, thought: Thought) -> Result<()> {
        self.make_tables()?;

        self.conn.execute(
            "INSERT INTO thoughts (thought, datetime) VALUES (?1, ?2)",
            params![thought.raw, thought.added],
        )?;

        let thought_id = self.conn.last_insert_rowid();

        for fragment in thought.fragments {
            if let Fragment::EntityRef { entity, .. } = fragment {
                self.conn.execute(
                    "INSERT INTO entities (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
                    params![entity],
                )?;
                let mut stmt = self.conn.prepare("SELECT id FROM entities WHERE name=?1")?;
                let mut rows = stmt.query_map(params![entity], |row| row.get::<usize, usize>(0))?;
                let entity_id = rows.next().unwrap()?;
                self.conn.execute(
                    "INSERT INTO thoughts_entities (thought_id, entity_id) VALUES (?1, ?2) ON CONFLICT(thought_id, entity_id) DO NOTHING",
                    params![thought_id, entity_id],
                )?;
            }
        }

        Ok(())
    }
    pub fn edit_thought(&self, thought_id: u32, thought: Thought) -> Result<()> {
        self.conn.execute(
            "UPDATE thoughts SET datetime = ?1 WHERE id = ?2",
            params!(thought.added, thought_id),
        )?;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::FromSqlError;
    use std::error::Error;
    #[test]
    fn sqlite_store_error_display() {
        let err = SqliteStoreError { message: String::from("this is an error") };
        assert_eq!(err.to_string(), "SQLite store error: this is an error")
    }

    #[test]
    fn sqlite_store_error_error() {
        let err = SqliteStoreError { message: String::from("this is an error") };
        let source = err.source();
        assert!(source.is_none())
    }

    #[test]
    fn sqlite_store_error_from_rusqlite() {
        let rusqlite_err = rusqlite::Error::from(FromSqlError::InvalidType);
        let rusqlite_err_message = rusqlite_err.to_string();
        let err = SqliteStoreError::from(rusqlite_err);
        assert_eq!(err.message, rusqlite_err_message)
    }
}
