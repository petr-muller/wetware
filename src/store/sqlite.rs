use rusqlite::{Connection, params, params_from_iter};
use crate::model::entities::Entity;
use crate::model::thoughts::{Fragment, RawThought, Thought};

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
    pub fn get_thoughts(&self, entity: Option<String>) -> Result<Vec<RawThought>> {
        let mut stmt_lines = vec!["SELECT thought, datetime FROM thoughts"];
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
            Ok(RawThought::from_store(
                row.get(0)?,
                row.get(1)?,
            ))
        })?;

        let mut thoughts = vec![];

        for thought in rows {
            thoughts.push(thought?);
        };

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
            if let Fragment::EntityRef{entity, ..} = fragment {
                self.conn.execute(
                    "INSERT INTO entities (name) VALUES (?1) ON CONFLICT(name) DO NOTHING",
                    params![entity],
                )?;
                let mut stmt = self.conn.prepare("SELECT id FROM entities WHERE name=?1")?;
                let mut rows = stmt.query_map(params![entity], |row| row.get::<usize, usize>(0))?;
                let entity_id = rows.next().unwrap()?;
                self.conn.execute(
                    "INSERT INTO thoughts_entities (thought_id, entity_id) VALUES (?1, ?2)",
                    params![thought_id, entity_id],
                )?;
            }
        }

        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use std::error::Error;
    use rusqlite::types::FromSqlError;
    use super::*;
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
