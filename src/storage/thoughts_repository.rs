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
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
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
        let mut stmt = conn.prepare("SELECT id, content, created_at FROM thoughts ORDER BY created_at ASC")?;

        let thoughts = stmt
            .query_map([], |row| {
                let created_at_str: String = row.get(2)?;
                let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
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

    /// Update an existing thought's content and/or date
    ///
    /// Applies the given content and created_at as the new values for the thought.
    /// Returns `ThoughtNotFound` if no thought with the given ID exists.
    ///
    /// # Arguments
    /// * `conn` - Database connection (or transaction)
    /// * `id` - ID of the thought to update
    /// * `content` - New content string
    /// * `created_at` - New timestamp
    pub fn update(conn: &Connection, id: i64, content: &str, created_at: DateTime<Utc>) -> Result<(), ThoughtError> {
        let rows_changed = conn.execute(
            "UPDATE thoughts SET content = ?1, created_at = ?2 WHERE id = ?3",
            (content, created_at.to_rfc3339(), id),
        )?;

        if rows_changed == 0 {
            return Err(ThoughtError::ThoughtNotFound(id));
        }

        Ok(())
    }

    /// Delete a thought by ID
    ///
    /// Returns `ThoughtNotFound` if no thought with the given ID exists.
    /// Associated `thought_entities` rows are removed by ON DELETE CASCADE.
    pub fn delete(conn: &Connection, id: i64) -> Result<(), ThoughtError> {
        let rows_changed = conn.execute("DELETE FROM thoughts WHERE id = ?1", [id])?;

        if rows_changed == 0 {
            return Err(ThoughtError::ThoughtNotFound(id));
        }

        Ok(())
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
                        rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
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
    fn test_update_thought_content() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("Original content".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();
        let original_date = thought.created_at;

        ThoughtsRepository::update(&conn, id, "Updated content", original_date).unwrap();

        let retrieved = ThoughtsRepository::get_by_id(&conn, id).unwrap();
        assert_eq!(retrieved.content, "Updated content");
        // Date should be preserved (same as we passed in)
        assert_eq!(retrieved.created_at.date_naive(), original_date.date_naive());
    }

    #[test]
    fn test_update_thought_date() {
        use chrono::{Datelike, NaiveDate};

        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("Content stays the same".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();

        let new_date = NaiveDate::from_ymd_opt(2026, 1, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        ThoughtsRepository::update(&conn, id, &thought.content, new_date).unwrap();

        let retrieved = ThoughtsRepository::get_by_id(&conn, id).unwrap();
        assert_eq!(retrieved.content, "Content stays the same");
        assert_eq!(retrieved.created_at.date_naive().year(), 2026);
        assert_eq!(retrieved.created_at.date_naive().month(), 1);
        assert_eq!(retrieved.created_at.date_naive().day(), 15);
    }

    #[test]
    fn test_update_thought_nonexistent_id() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        use chrono::Utc;
        let result = ThoughtsRepository::update(&conn, 9999, "content", Utc::now());
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(crate::errors::ThoughtError::ThoughtNotFound(9999))
        ));
    }

    #[test]
    fn test_delete_thought() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("To be deleted".to_string()).unwrap();
        let id = ThoughtsRepository::save(&conn, &thought).unwrap();

        ThoughtsRepository::delete(&conn, id).unwrap();

        let result = ThoughtsRepository::get_by_id(&conn, id);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_thought_nonexistent_id() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let result = ThoughtsRepository::delete(&conn, 9999);
        assert!(matches!(
            result,
            Err(crate::errors::ThoughtError::ThoughtNotFound(9999))
        ));
    }

    #[test]
    fn test_delete_thought_cascades_to_entity_associations() {
        use crate::models::entity::Entity;
        use crate::storage::entities_repository::EntitiesRepository;

        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thought = Thought::new("Meeting with [Sarah]".to_string()).unwrap();
        let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
        let entity = Entity::new("Sarah".to_string());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
        EntitiesRepository::link_to_thought(&conn, entity_id, thought_id).unwrap();

        ThoughtsRepository::delete(&conn, thought_id).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM thought_entities WHERE thought_id = ?1",
                [thought_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
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
