/// Repository for thoughts persistence
use crate::errors::ThoughtError;
use crate::models::thought::Thought;
use chrono::{DateTime, Utc};
use rusqlite::Connection;

/// Thoughts repository for database operations
pub struct ThoughtsRepository;

impl ThoughtsRepository {
    /// Save a thought to the database
    ///
    /// Returns the ID of the saved thought
    pub fn save(conn: &Connection, thought: &Thought) -> Result<i64, ThoughtError> {
        conn.execute(
            "INSERT INTO thoughts (content, created_at) VALUES (?1, ?2)",
            (&thought.content, thought.created_at.to_rfc3339()),
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Get a thought by ID
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Thought, ThoughtError> {
        let mut stmt = conn.prepare("SELECT id, content, created_at FROM thoughts WHERE id = ?1")?;

        let thought = stmt.query_row([id], |row| {
            let created_at_str: String = row.get(2)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            Ok(Thought {
                id: Some(row.get(0)?),
                content: row.get(1)?,
                created_at,
            })
        })?;

        Ok(thought)
    }

    /// List all thoughts in chronological order (oldest first)
    pub fn list_all(conn: &Connection) -> Result<Vec<Thought>, ThoughtError> {
        let mut stmt =
            conn.prepare("SELECT id, content, created_at FROM thoughts ORDER BY created_at ASC")?;

        let thoughts = stmt
            .query_map([], |row| {
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(Thought {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    created_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(thoughts)
    }

    /// List thoughts filtered by entity name (case-insensitive)
    pub fn list_by_entity(conn: &Connection, entity_name: &str) -> Result<Vec<Thought>, ThoughtError> {
        let lowercase_name = entity_name.to_lowercase();

        let mut stmt = conn.prepare(
            "SELECT DISTINCT t.id, t.content, t.created_at
             FROM thoughts t
             INNER JOIN thought_entities te ON t.id = te.thought_id
             INNER JOIN entities e ON te.entity_id = e.id
             WHERE e.name = ?1
             ORDER BY t.created_at ASC",
        )?;

        let thoughts = stmt
            .query_map([lowercase_name], |row| {
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc);

                Ok(Thought {
                    id: Some(row.get(0)?),
                    content: row.get(1)?,
                    created_at,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(thoughts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::migrations::run_migrations;

    #[test]
    fn test_save_thought() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("Test thought".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_get_thought_by_id() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("Test thought".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();

        let retrieved = ThoughtsRepository::get_by_id(&conn, id).unwrap();
        assert_eq!(retrieved.content, "Test thought");
        assert_eq!(retrieved.id, Some(id));
    }

    #[test]
    fn test_list_all_thoughts_empty() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thoughts = ThoughtsRepository::list_all(&conn).unwrap();
        assert!(thoughts.is_empty());
    }

    #[test]
    fn test_list_all_thoughts() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought1 = Thought::new("First".to_string()).unwrap();
        let thought2 = Thought::new("Second".to_string()).unwrap();

        ThoughtsRepository::save(&conn, &thought1).unwrap();
        ThoughtsRepository::save(&conn, &thought2).unwrap();

        let thoughts = ThoughtsRepository::list_all(&conn).unwrap();
        assert_eq!(thoughts.len(), 2);
        assert_eq!(thoughts[0].content, "First");
        assert_eq!(thoughts[1].content, "Second");
    }
}
