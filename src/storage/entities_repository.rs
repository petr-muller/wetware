/// Repository for entities persistence
use crate::errors::ThoughtError;
use crate::models::entity::Entity;
use rusqlite::{Connection, OptionalExtension};

/// Entities repository for database operations
pub struct EntitiesRepository;

impl EntitiesRepository {
    /// Find an entity by name (case-insensitive) or create it if it doesn't exist
    ///
    /// Returns the ID of the found or created entity
    pub fn find_or_create(conn: &Connection, entity: &Entity) -> Result<i64, ThoughtError> {
        // Try to find existing entity (case-insensitive via COLLATE NOCASE)
        let mut stmt = conn.prepare("SELECT id FROM entities WHERE name = ?1")?;
        let existing: Option<i64> = stmt.query_row([&entity.name], |row| row.get(0)).optional()?;

        if let Some(id) = existing {
            return Ok(id);
        }

        // Create new entity
        conn.execute(
            "INSERT INTO entities (name, canonical_name) VALUES (?1, ?2)",
            (&entity.name, &entity.canonical_name),
        )?;

        Ok(conn.last_insert_rowid())
    }

    /// Link an entity to a thought
    pub fn link_to_thought(conn: &Connection, entity_id: i64, thought_id: i64) -> Result<(), ThoughtError> {
        conn.execute(
            "INSERT OR IGNORE INTO thought_entities (thought_id, entity_id) VALUES (?1, ?2)",
            (thought_id, entity_id),
        )?;
        Ok(())
    }

    /// Find an entity by name (case-insensitive)
    pub fn find_by_name(conn: &Connection, name: &str) -> Result<Option<Entity>, ThoughtError> {
        let lowercase_name = name.to_lowercase();
        let mut stmt = conn.prepare("SELECT id, name, canonical_name, description FROM entities WHERE name = ?1")?;

        let entity = stmt
            .query_row([lowercase_name], |row| {
                Ok(Entity {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    canonical_name: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .optional()?;

        Ok(entity)
    }

    /// List all entities in alphabetical order
    pub fn list_all(conn: &Connection) -> Result<Vec<Entity>, ThoughtError> {
        let mut stmt =
            conn.prepare("SELECT id, name, canonical_name, description FROM entities ORDER BY canonical_name ASC")?;

        let entities = stmt
            .query_map([], |row| {
                Ok(Entity {
                    id: Some(row.get(0)?),
                    name: row.get(1)?,
                    canonical_name: row.get(2)?,
                    description: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    /// Update entity description (or remove if None)
    ///
    /// Returns error if entity doesn't exist
    pub fn update_description(
        conn: &Connection,
        entity_name: &str,
        description: Option<String>,
    ) -> Result<(), ThoughtError> {
        let lowercase_name = entity_name.to_lowercase();

        // Verify entity exists
        let exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM entities WHERE name = ?1",
            [&lowercase_name],
            |row| row.get(0),
        )?;

        if !exists {
            return Err(ThoughtError::EntityNotFound(entity_name.to_string()));
        }

        // Update description
        conn.execute(
            "UPDATE entities SET description = ?1 WHERE name = ?2",
            (description, &lowercase_name),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::migrations::run_migrations;

    #[test]
    fn test_find_or_create_new_entity() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("TestEntity".to_string());
        let id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_find_or_create_existing_entity() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity1 = Entity::new("TestEntity".to_string());
        let id1 = EntitiesRepository::find_or_create(&conn, &entity1).unwrap();

        let entity2 = Entity::new("testentity".to_string());
        let id2 = EntitiesRepository::find_or_create(&conn, &entity2).unwrap();

        assert_eq!(id1, id2);
    }

    #[test]
    fn test_link_to_thought() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("TestEntity".to_string());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

        // Create a thought manually for testing
        conn.execute(
            "INSERT INTO thoughts (content, created_at) VALUES ('Test', datetime('now'))",
            [],
        )
        .unwrap();
        let thought_id = conn.last_insert_rowid();

        let result = EntitiesRepository::link_to_thought(&conn, entity_id, thought_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_by_name() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("TestEntity".to_string());
        EntitiesRepository::find_or_create(&conn, &entity).unwrap();

        let found = EntitiesRepository::find_by_name(&conn, "testentity").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().canonical_name, "TestEntity");
    }

    #[test]
    fn test_list_all_alphabetical() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity1 = Entity::new("Zebra".to_string());
        let entity2 = Entity::new("Apple".to_string());
        let entity3 = Entity::new("Middle".to_string());

        EntitiesRepository::find_or_create(&conn, &entity1).unwrap();
        EntitiesRepository::find_or_create(&conn, &entity2).unwrap();
        EntitiesRepository::find_or_create(&conn, &entity3).unwrap();

        let entities = EntitiesRepository::list_all(&conn).unwrap();
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].canonical_name, "Apple");
        assert_eq!(entities[1].canonical_name, "Middle");
        assert_eq!(entities[2].canonical_name, "Zebra");
    }

    #[test]
    fn test_update_description() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("TestEntity".to_string());
        EntitiesRepository::find_or_create(&conn, &entity).unwrap();

        // Add description
        let desc = Some("Test description".to_string());
        EntitiesRepository::update_description(&conn, "testentity", desc.clone()).unwrap();

        // Verify description was set
        let found = EntitiesRepository::find_by_name(&conn, "testentity").unwrap().unwrap();
        assert_eq!(found.description, desc);
    }

    #[test]
    fn test_update_description_to_none() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let entity = Entity::new("TestEntity".to_string());
        EntitiesRepository::find_or_create(&conn, &entity).unwrap();

        // Add description
        EntitiesRepository::update_description(&conn, "testentity", Some("Test".to_string())).unwrap();

        // Remove description
        EntitiesRepository::update_description(&conn, "testentity", None).unwrap();

        // Verify description was removed
        let found = EntitiesRepository::find_by_name(&conn, "testentity").unwrap().unwrap();
        assert_eq!(found.description, None);
    }

    #[test]
    fn test_update_description_nonexistent_entity() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let result = EntitiesRepository::update_description(&conn, "nonexistent", Some("Test".to_string()));
        assert!(result.is_err());
        match result {
            Err(ThoughtError::EntityNotFound(name)) => assert_eq!(name, "nonexistent"),
            _ => panic!("Expected EntityNotFound error"),
        }
    }
}
