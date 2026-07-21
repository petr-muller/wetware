/// Repository for entity aliases persistence
use crate::errors::ThoughtError;
use crate::models::entity::Entity;
use rusqlite::Connection;

/// Entity aliases repository for database operations
///
/// Aliases are unique per entity, not globally: the same alias string may be
/// registered for more than one entity, so callers resolving an alias to an
/// entity must handle the possibility of more than one match (see
/// `find_entities_by_alias` and `EntitiesRepository::resolve`).
pub struct EntityAliasesRepository;

impl EntityAliasesRepository {
    /// Register `alias` for `entity_id`. Idempotent - a no-op if the entity already
    /// has this alias registered.
    ///
    /// `INSERT OR IGNORE` also suppresses CHECK constraint failures in SQLite, so an
    /// empty/whitespace-only alias is rejected explicitly here rather than relying on
    /// the table's CHECK constraint alone.
    pub fn add_alias(conn: &Connection, entity_id: i64, alias: &str) -> Result<(), ThoughtError> {
        let trimmed = alias.trim();
        if trimmed.is_empty() {
            return Err(ThoughtError::InvalidInput("Alias cannot be empty".to_string()));
        }

        conn.execute(
            "INSERT OR IGNORE INTO entity_aliases (entity_id, alias) VALUES (?1, ?2)",
            (entity_id, trimmed),
        )?;
        Ok(())
    }

    /// Remove `alias` from `entity_id` if present. Safe to call even if no such alias
    /// is registered (no-op). Trims `alias` first, matching `add_alias`'s storage form.
    pub fn remove_alias(conn: &Connection, entity_id: i64, alias: &str) -> Result<(), ThoughtError> {
        conn.execute(
            "DELETE FROM entity_aliases WHERE entity_id = ?1 AND alias = ?2",
            (entity_id, alias.trim()),
        )?;
        Ok(())
    }

    /// All aliases registered for `entity_id`, alphabetical.
    pub fn list_for_entity(conn: &Connection, entity_id: i64) -> Result<Vec<String>, ThoughtError> {
        let mut stmt = conn.prepare("SELECT alias FROM entity_aliases WHERE entity_id = ?1 ORDER BY alias ASC")?;

        let aliases = stmt
            .query_map([entity_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(aliases)
    }

    /// All distinct entities that have `alias` registered (case-insensitive).
    ///
    /// Empty vec means the alias isn't registered for any entity. More than one
    /// entry means the same alias string was registered for multiple entities -
    /// callers must treat that as ambiguous rather than picking one.
    pub fn find_entities_by_alias(conn: &Connection, alias: &str) -> Result<Vec<Entity>, ThoughtError> {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.name, e.canonical_name, e.description
             FROM entities e
             INNER JOIN entity_aliases ea ON ea.entity_id = e.id
             WHERE ea.alias = ?1
             ORDER BY e.canonical_name ASC",
        )?;

        let entities = stmt
            .query_map([alias], |row| {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::entities_repository::EntitiesRepository;
    use crate::storage::migrations::run_migrations;

    fn make_entity(conn: &Connection, name: &str) -> i64 {
        let entity = Entity::new(name.to_string());
        EntitiesRepository::find_or_create(conn, &entity).unwrap()
    }

    #[test]
    fn test_add_alias_creates_entry() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert_eq!(aliases, vec!["sar"]);
    }

    #[test]
    fn test_add_alias_idempotent() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert_eq!(aliases.len(), 1);
    }

    #[test]
    fn test_add_alias_same_alias_two_different_entities() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        let john = make_entity(&conn, "John");

        EntityAliasesRepository::add_alias(&conn, sarah, "boss").unwrap();
        EntityAliasesRepository::add_alias(&conn, john, "boss").unwrap();

        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "boss").unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_remove_alias_removes_existing() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();
        EntityAliasesRepository::remove_alias(&conn, sarah, "sar").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_remove_alias_nonexistent_is_noop() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        let result = EntityAliasesRepository::remove_alias(&conn, sarah, "sar");
        assert!(result.is_ok());
    }

    #[test]
    fn test_find_entities_by_alias_no_matches() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "nonexistent").unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_find_entities_by_alias_single_match() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();

        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "sar").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].canonical_name, "Sarah");
    }

    #[test]
    fn test_find_entities_by_alias_case_insensitive() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "Sar").unwrap();

        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "SAR").unwrap();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_list_for_entity_alphabetical() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "zeta").unwrap();
        EntityAliasesRepository::add_alias(&conn, sarah, "alpha").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert_eq!(aliases, vec!["alpha", "zeta"]);
    }

    #[test]
    fn test_deleting_entity_cascades_alias_removal() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();

        conn.execute("DELETE FROM entities WHERE id = ?1", [sarah]).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM entity_aliases WHERE entity_id = ?1",
                [sarah],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_add_alias_empty_rejected() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        let result = EntityAliasesRepository::add_alias(&conn, sarah, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_alias_trims_surrounding_whitespace() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "  sar  ").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert_eq!(aliases, vec!["sar"]);

        // A later lookup for the untrimmed form still matches, since it's stored trimmed.
        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "sar").unwrap();
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_remove_alias_trims_surrounding_whitespace() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah = make_entity(&conn, "Sarah");
        EntityAliasesRepository::add_alias(&conn, sarah, "sar").unwrap();
        EntityAliasesRepository::remove_alias(&conn, sarah, "  sar  ").unwrap();

        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah).unwrap();
        assert!(aliases.is_empty());
    }
}
