use crate::model::entities;
use crate::model::fragments::Fragment;
use crate::model::thoughts::{AddedThought, EditedThought, RawThought, Thought};
use chrono::NaiveDate;
use indexmap::IndexMap;
use rusqlite::{params, params_from_iter, Connection};
use rusqlite_migration::{Migrations, M};

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

impl From<crate::model::thoughts::Error> for SqliteStoreError {
    fn from(thought_err: crate::model::thoughts::Error) -> Self {
        SqliteStoreError {
            message: thought_err.to_string(),
        }
    }
}

type Result<T> = std::result::Result<T, SqliteStoreError>;

pub fn open(db: &String) -> Result<Store> {
    let migrations = Migrations::new(vec![
        M::up(
            "CREATE TABLE IF NOT EXISTS thoughts (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought     TEXT NOT NULL,
                    datetime    INTEGER NOT NULL
                   );
                   CREATE TABLE IF NOT EXISTS entities (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    name        TEXT NOT NULL UNIQUE
                   );
                   CREATE TABLE IF NOT EXISTS thoughts_entities (
                    id          INTEGER PRIMARY KEY AUTOINCREMENT,
                    thought_id  INTEGER,
                    entity_id   INTEGER,
                    FOREIGN KEY(thought_id) REFERENCES thoughts(id),
                    FOREIGN KEY(entity_id)  REFERENCES entities(id),
                    UNIQUE(thought_id, entity_id)
                   );",
        ),
        M::up(
            r#"ALTER TABLE entities
                   ADD description TEXT NOT NULL DEFAULT "";
                   CREATE TABLE IF NOT EXISTS entity_description_entities (
                     id              INTEGER PRIMARY KEY AUTOINCREMENT,
                     entity_id       INTEGER,
                     entity_ref_id   INTEGER,
                     FOREIGN KEY(entity_id)      REFERENCES entities(id),
                     FOREIGN KEY(entity_ref_id)  REFERENCES entities(id),
                     UNIQUE(entity_id, entity_ref_id)
                 );"#,
        ),
    ]);

    let mut conn = Connection::open(db)?;

    migrations.to_latest(&mut conn).unwrap();

    Ok(Store { conn })
}

impl Store {
    pub fn get_entities(&self) -> Result<Vec<entities::RawEntity>> {
        let stmt = "SELECT name, description FROM entities ORDER BY name";
        let mut stmt = self.conn.prepare(stmt)?;
        let rows = stmt.query_map(params![], |row| {
            Ok(entities::RawEntity {
                name: row.get(0)?,
                description: row.get(1)?,
            })
        })?;

        let mut entities = vec![];
        for entity in rows {
            entities.push(entity?);
        }

        Ok(entities)
    }

    pub fn get_entity(&self, entity_name: &String) -> Result<entities::RawEntity> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, description FROM entities WHERE name = ?1")?;
        let rows = stmt.query_map(params![entity_name], |row| {
            let name: String = row.get(0)?;
            let description: String = row.get(1)?;
            Ok(entities::RawEntity { name, description })
        })?;

        let mut entities = vec![];
        for row in rows {
            entities.push(row?);
        }

        match entities.len() {
            0 => Err(SqliteStoreError {
                message: format!("No such entity: {entity_name}"),
            }),
            1 => Ok(entities[0].clone()),
            _ => Err(SqliteStoreError {
                message: format!("BUG: Multiple entities named {entity_name}"),
            }),
        }
    }

    pub fn edit_entity(&self, entity: entities::Entity) -> Result<()> {
        self.conn.execute(
            "UPDATE entities SET description = ?1 WHERE name = ?2",
            params![entity.description.raw, entity.name],
        )?;

        let mut stmt = self.conn.prepare("SELECT id FROM entities WHERE name=?1")?;
        let mut rows = stmt.query_map(params![entity.name], |row| row.get::<usize, u32>(0))?;
        let entity_id = rows.next().unwrap()?;

        self.conn.execute(
            "DELETE FROM entity_description_entities WHERE entity_id = ?1",
            params![entity_id],
        )?;

        for fragment in entity.description.fragments {
            if let Fragment::EntityRef { entity, .. } = fragment {
                self.link_entity_from_description(entity_id, entity)?;
            }
        }

        Ok(())
    }

    pub fn get_thought(&self, thought_id: u32) -> Result<RawThought> {
        let mut stmt = self
            .conn
            .prepare("SELECT thought, datetime FROM thoughts WHERE id = ?1")?;
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
            0 => Err(SqliteStoreError {
                message: format!("No thought with id {thought_id}"),
            }),
            1 => Ok(thoughts[0].clone()),
            _ => Err(SqliteStoreError {
                message: format!("BUG: Multiple thoughts with id {thought_id}"),
            }),
        }
    }

    pub fn get_thoughts(&self, entity: Option<String>) -> Result<IndexMap<u32, RawThought>> {
        let mut stmt_lines = vec!["SELECT thoughts.id, thought, datetime FROM thoughts"];
        let mut params = vec![];

        if let Some(entity) = entity {
            stmt_lines.append(&mut vec![
                "JOIN thoughts_entities ON thoughts.id = thoughts_entities.thought_id",
                "JOIN entities ON thoughts_entities.entity_id = entities.id",
                "WHERE entities.name = ?1",
            ]);
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

    pub fn add_thought(&self, thought: Thought) -> Result<AddedThought> {
        self.conn.execute(
            "INSERT INTO thoughts (thought, datetime) VALUES (?1, ?2)",
            params![thought.text.raw, thought.added],
        )?;

        let thought_id = self.conn.last_insert_rowid() as u32;

        let mut added = AddedThought {
            id: thought_id,
            thought: thought.clone(),
            new_entities: vec![],
        };

        for fragment in thought.text.fragments {
            if let Fragment::EntityRef { entity, .. } = fragment {
                if self.link_thought_to_entity(thought_id, entity.clone())? {
                    added.new_entities.push(entity);
                }
            }
        }

        Ok(added)
    }

    pub fn edit_thought(&self, thought_id: u32, thought: Thought) -> Result<EditedThought> {
        // Get the old thought before updating
        let old_thought = self.get_thought(thought_id)?.as_thought()?;
        
        self.conn.execute(
            "UPDATE thoughts SET thought = ?1, datetime = ?2 WHERE id = ?3",
            params!(thought.text.raw, thought.added, thought_id),
        )?;

        self.conn.execute(
            "DELETE FROM thoughts_entities WHERE thought_id = ?1",
            params![thought_id],
        )?;

        let mut edited = EditedThought {
            id: thought_id,
            old_thought,
            thought: thought.clone(),
            new_entities: vec![],
        };

        for fragment in thought.text.fragments {
            if let Fragment::EntityRef { entity, .. } = fragment {
                if self.link_thought_to_entity(thought_id, entity.clone())? {
                    edited.new_entities.push(entity);
                }
            }
        }

        Ok(edited)
    }

    fn link_entity_from_description(&self, described: u32, linked: entities::Id) -> Result<()> {
        self.conn.execute(
            "INSERT INTO entities (name, description) VALUES (?1, ?2) ON CONFLICT(name) DO NOTHING",
            // descriptions are empty by default
            params![linked, ""],
        )?;

        let mut stmt = self.conn.prepare("SELECT id FROM entities WHERE name=?1")?;
        let mut rows = stmt.query_map(params![linked], |row| row.get::<usize, u32>(0))?;
        let linked_id = rows.next().unwrap()?;

        self.conn.execute(
            "INSERT INTO entity_description_entities (entity_id, entity_ref_id) VALUES (?1, ?2) ON CONFLICT(entity_id, entity_ref_id) DO NOTHING",
            params![described, linked_id],
        )?;
        Ok(())
    }

    fn link_thought_to_entity(&self, thought_id: u32, entity: entities::Id) -> Result<bool> {
        let mut select_entity = self.conn.prepare("SELECT id FROM entities WHERE name=?1")?;
        let mut rows = select_entity.query(params![entity])?;
        let mut added_entity = false;
        let entity_id = if let Some(id) = rows.next()? {
            id.get(0)
        } else {
            added_entity = true;
            let insert_entity = self
                .conn
                .prepare("INSERT INTO entities (name, description) VALUES (?1, ?2)");
            insert_entity?.insert(params![entity, ""])
        }?;

        self.conn.execute(
            "INSERT INTO thoughts_entities (thought_id, entity_id) VALUES (?1, ?2) ON CONFLICT(thought_id, entity_id) DO NOTHING",
            params![thought_id, entity_id],
        )?;

        Ok(added_entity)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::types::FromSqlError;
    use std::error::Error;
    #[test]
    fn sqlite_store_error_display() {
        let err = SqliteStoreError {
            message: String::from("this is an error"),
        };
        assert_eq!(err.to_string(), "SQLite store error: this is an error")
    }

    #[test]
    fn sqlite_store_error_error() {
        let err = SqliteStoreError {
            message: String::from("this is an error"),
        };
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
