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

    /// List thoughts filtered by entity name or alias (case-insensitive), including
    /// thoughts linked to any entity transitively reachable via child relations
    /// (descendants). An unknown name/alias returns an empty list; an alias registered
    /// to more than one entity returns `ThoughtError::AmbiguousAlias`.
    pub fn list_by_entity(conn: &Connection, entity_name: &str) -> Result<Vec<Thought>, ThoughtError> {
        let Some(entity) = crate::storage::entities_repository::EntitiesRepository::resolve(conn, entity_name)? else {
            return Ok(Vec::new());
        };

        let mut stmt = conn.prepare(
            "WITH RECURSIVE reachable(id) AS (
                 SELECT ?1
                 UNION
                 SELECT er.child_id FROM entity_relations er JOIN reachable r ON er.parent_id = r.id
             )
             SELECT DISTINCT t.id, t.content, t.created_at
             FROM thoughts t
             INNER JOIN thought_entities te ON t.id = te.thought_id
             INNER JOIN reachable r ON te.entity_id = r.id
             ORDER BY t.created_at ASC",
        )?;

        let thoughts = stmt
            .query_map([entity.id.unwrap()], |row| {
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

    /// List the most recent thoughts linked to an entity name or alias (case-insensitive),
    /// newest first, including thoughts linked to any entity transitively reachable via
    /// child relations (descendants). An unknown name/alias returns an empty list; an
    /// alias registered to more than one entity returns `ThoughtError::AmbiguousAlias`.
    pub fn list_latest_by_entity(
        conn: &Connection,
        entity_name: &str,
        limit: usize,
    ) -> Result<Vec<Thought>, ThoughtError> {
        let Some(entity) = crate::storage::entities_repository::EntitiesRepository::resolve(conn, entity_name)? else {
            return Ok(Vec::new());
        };

        let mut stmt = conn.prepare(
            "WITH RECURSIVE reachable(id) AS (
                 SELECT ?1
                 UNION
                 SELECT er.child_id FROM entity_relations er JOIN reachable r ON er.parent_id = r.id
             )
             SELECT DISTINCT t.id, t.content, t.created_at
             FROM thoughts t
             INNER JOIN thought_entities te ON t.id = te.thought_id
             INNER JOIN reachable r ON te.entity_id = r.id
             ORDER BY t.created_at DESC
             LIMIT ?2",
        )?;

        let thoughts = stmt
            .query_map((entity.id.unwrap(), limit as i64), |row| {
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

    /// Save a thought with the given content, link it to `entity_name` (creating the
    /// entity if needed), and set its `created_at` explicitly so ordering is deterministic.
    fn save_linked_thought(conn: &Connection, content: &str, entity_name: &str, created_at: DateTime<Utc>) -> i64 {
        use crate::models::entity::Entity;
        use crate::storage::entities_repository::EntitiesRepository;

        let thought = Thought::new(content.to_string()).unwrap();
        let thought_id = ThoughtsRepository::save(conn, &thought).unwrap();
        ThoughtsRepository::update(conn, thought_id, content, created_at).unwrap();

        let entity = Entity::new(entity_name.to_string());
        let entity_id = EntitiesRepository::find_or_create(conn, &entity).unwrap();
        EntitiesRepository::link_to_thought(conn, entity_id, thought_id).unwrap();

        thought_id
    }

    fn day(offset: i64) -> DateTime<Utc> {
        use chrono::Duration;
        Utc::now() + Duration::days(offset)
    }

    #[test]
    fn test_list_latest_by_entity_orders_newest_first() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "Oldest", "rust", day(-2));
        save_linked_thought(&conn, "Middle", "rust", day(-1));
        save_linked_thought(&conn, "Newest", "rust", day(0));

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "rust", 5).unwrap();
        assert_eq!(thoughts.len(), 3);
        assert_eq!(thoughts[0].content, "Newest");
        assert_eq!(thoughts[1].content, "Middle");
        assert_eq!(thoughts[2].content, "Oldest");
    }

    #[test]
    fn test_list_latest_by_entity_respects_limit() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        for i in 0..7 {
            save_linked_thought(&conn, &format!("Thought {i}"), "rust", day(i));
        }

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "rust", 5).unwrap();
        assert_eq!(thoughts.len(), 5);
        assert_eq!(thoughts[0].content, "Thought 6");
        assert_eq!(thoughts[4].content, "Thought 2");
    }

    #[test]
    fn test_list_latest_by_entity_fewer_than_limit_returns_all() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "Only one", "rust", day(0));

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "rust", 5).unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "Only one");
    }

    #[test]
    fn test_list_latest_by_entity_no_thoughts_returns_empty() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "nonexistent", 5).unwrap();
        assert!(thoughts.is_empty());
    }

    #[test]
    fn test_list_latest_by_entity_case_insensitive() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "About Rust", "Rust", day(0));

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "RUST", 5).unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "About Rust");
    }

    fn relate(conn: &Connection, child_name: &str, parent_name: &str) {
        use crate::storage::entities_repository::EntitiesRepository;
        use crate::storage::entity_relations_repository::EntityRelationsRepository;

        let child_id = EntitiesRepository::find_by_name(conn, child_name)
            .unwrap()
            .unwrap()
            .id
            .unwrap();
        let parent_id = EntitiesRepository::find_by_name(conn, parent_name)
            .unwrap()
            .unwrap()
            .id
            .unwrap();
        EntityRelationsRepository::add_relation(conn, child_id, parent_id).unwrap();
    }

    #[test]
    fn test_list_by_entity_no_relations_behaves_as_before() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "About Rust", "Rust", day(0));
        save_linked_thought(&conn, "About Go", "Go", day(0));

        let thoughts = ThoughtsRepository::list_by_entity(&conn, "rust").unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "About Rust");
    }

    #[test]
    fn test_list_by_entity_includes_descendant_thoughts() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "About Amazon", "Amazon", day(-1));
        save_linked_thought(&conn, "About AWS", "AWS", day(0));
        relate(&conn, "AWS", "Amazon");

        let thoughts = ThoughtsRepository::list_by_entity(&conn, "amazon").unwrap();
        let contents: Vec<_> = thoughts.iter().map(|t| t.content.as_str()).collect();
        assert_eq!(contents, vec!["About Amazon", "About AWS"]);
    }

    #[test]
    fn test_list_by_entity_thought_on_root_still_included() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        use crate::models::entity::Entity;
        use crate::storage::entities_repository::EntitiesRepository;

        save_linked_thought(&conn, "About Amazon", "Amazon", day(0));
        EntitiesRepository::find_or_create(&conn, &Entity::new("AWS".to_string())).unwrap();
        relate(&conn, "AWS", "Amazon");

        let thoughts = ThoughtsRepository::list_by_entity(&conn, "amazon").unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "About Amazon");
    }

    #[test]
    fn test_list_by_entity_diamond_shape_no_duplicates() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        // Diamond: D is child of both B and C, both of which are children of A.
        save_linked_thought(&conn, "About A", "A", day(-3));
        save_linked_thought(&conn, "About B", "B", day(-2));
        save_linked_thought(&conn, "About C", "C", day(-1));
        save_linked_thought(&conn, "About D", "D", day(0));
        relate(&conn, "B", "A");
        relate(&conn, "C", "A");
        relate(&conn, "D", "B");
        relate(&conn, "D", "C");

        let thoughts = ThoughtsRepository::list_by_entity(&conn, "a").unwrap();
        assert_eq!(thoughts.len(), 4);
        let d_count = thoughts.iter().filter(|t| t.content == "About D").count();
        assert_eq!(d_count, 1);
    }

    #[test]
    fn test_list_by_entity_unknown_entity_returns_empty() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let thoughts = ThoughtsRepository::list_by_entity(&conn, "nonexistent").unwrap();
        assert!(thoughts.is_empty());
    }

    #[test]
    fn test_list_by_entity_resolves_alias_same_as_canonical_name() {
        use crate::models::entity::Entity;
        use crate::storage::entities_repository::EntitiesRepository;
        use crate::storage::entity_aliases_repository::EntityAliasesRepository;

        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("Rust".to_string());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
        EntityAliasesRepository::add_alias(&conn, entity_id, "rustlang").unwrap();

        let thought = Thought::new("Learning [Rust]".to_string()).unwrap();
        let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
        EntitiesRepository::link_to_thought(&conn, entity_id, thought_id).unwrap();

        let by_canonical = ThoughtsRepository::list_by_entity(&conn, "rust").unwrap();
        let by_alias = ThoughtsRepository::list_by_entity(&conn, "rustlang").unwrap();
        assert_eq!(by_canonical.len(), 1);
        assert_eq!(by_alias.len(), 1);
        assert_eq!(by_canonical[0].id, by_alias[0].id);
    }

    #[test]
    fn test_list_by_entity_ambiguous_alias_errors() {
        use crate::models::entity::Entity;
        use crate::storage::entities_repository::EntitiesRepository;
        use crate::storage::entity_aliases_repository::EntityAliasesRepository;

        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah_id = EntitiesRepository::find_or_create(&conn, &Entity::new("Sarah".to_string())).unwrap();
        let john_id = EntitiesRepository::find_or_create(&conn, &Entity::new("John".to_string())).unwrap();
        EntityAliasesRepository::add_alias(&conn, sarah_id, "boss").unwrap();
        EntityAliasesRepository::add_alias(&conn, john_id, "boss").unwrap();

        let result = ThoughtsRepository::list_by_entity(&conn, "boss");
        assert!(matches!(result, Err(ThoughtError::AmbiguousAlias { .. })));
    }

    #[test]
    fn test_list_latest_by_entity_includes_descendant_thoughts() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "About Amazon", "Amazon", day(-1));
        save_linked_thought(&conn, "About AWS", "AWS", day(0));
        relate(&conn, "AWS", "Amazon");

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "amazon", 5).unwrap();
        assert_eq!(thoughts.len(), 2);
        assert_eq!(thoughts[0].content, "About AWS");
        assert_eq!(thoughts[1].content, "About Amazon");
    }

    #[test]
    fn test_list_latest_by_entity_limit_applies_across_descendants() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        save_linked_thought(&conn, "About Amazon", "Amazon", day(-1));
        save_linked_thought(&conn, "About AWS", "AWS", day(0));
        relate(&conn, "AWS", "Amazon");

        let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, "amazon", 1).unwrap();
        assert_eq!(thoughts.len(), 1);
        assert_eq!(thoughts[0].content, "About AWS");
    }
}
