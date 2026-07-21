/// Entity alias/unalias command implementations
use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_aliases_repository::EntityAliasesRepository;
use crate::storage::migrations::run_migrations;
use rusqlite::Connection;
use std::path::Path;

fn require_entity(conn: &Connection, name: &str) -> Result<crate::models::entity::Entity, ThoughtError> {
    EntitiesRepository::resolve(conn, name)?.ok_or_else(|| {
        eprintln!("Error: Entity '{}' not found", name);
        eprintln!();
        eprintln!("Hint: Create the entity first by referencing it in a thought:");
        eprintln!("  wet add \"Learning about [{}] today\"", name);
        ThoughtError::EntityNotFound(name.to_string())
    })
}

/// Execute the entity alias command
///
/// Registers `alias` as an alternate name for `entity_name`, resolving everywhere the
/// entity is looked up by name (filtering, `entity show`, thought/description mentions,
/// etc). Idempotent: registering an already-registered alias succeeds without error.
/// The same alias may be registered for more than one entity - aliases are unique per
/// entity, not globally.
///
/// # Arguments
/// * `entity_name` - Entity to register the alias for (case-insensitive; may itself be
///   an existing alias)
/// * `alias` - Alias to register
/// * `db_path` - Database path
pub fn execute_alias(entity_name: &str, alias: &str, db_path: &Path) -> Result<(), ThoughtError> {
    if alias.trim().is_empty() {
        return Err(ThoughtError::InvalidInput("Alias cannot be empty".to_string()));
    }

    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let entity = require_entity(&conn, entity_name)?;

    EntityAliasesRepository::add_alias(&conn, entity.id.unwrap(), alias)?;

    println!("Alias '{}' added for entity '{}'.", alias, entity.canonical_name);

    Ok(())
}

/// Execute the entity unalias command
///
/// Removes `alias` from `entity_name`. Idempotent: succeeds even if the alias wasn't
/// registered.
///
/// # Arguments
/// * `entity_name` - Entity to remove the alias from (case-insensitive; may itself be
///   an existing alias)
/// * `alias` - Alias to remove
/// * `db_path` - Database path
pub fn execute_unalias(entity_name: &str, alias: &str, db_path: &Path) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let entity = require_entity(&conn, entity_name)?;

    EntityAliasesRepository::remove_alias(&conn, entity.id.unwrap(), alias)?;

    println!("Alias '{}' removed from entity '{}'.", alias, entity.canonical_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entity::Entity;
    use tempfile::TempDir;

    fn setup_entity(conn: &Connection, name: &str) {
        let entity = Entity::new(name.to_string());
        EntitiesRepository::find_or_create(conn, &entity).unwrap();
    }

    fn temp_db_with_entities(names: &[&str]) -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        for name in names {
            setup_entity(&conn, name);
        }

        (temp_dir, db_path)
    }

    #[test]
    fn test_execute_alias_creates_alias() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        let result = execute_alias("Sarah", "sar", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let sarah = EntitiesRepository::find_by_name(&conn, "sarah").unwrap().unwrap();
        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah.id.unwrap()).unwrap();
        assert_eq!(aliases, vec!["sar"]);
    }

    #[test]
    fn test_execute_alias_missing_entity_errors() {
        let (_temp_dir, db_path) = temp_db_with_entities(&[]);

        let result = execute_alias("Sarah", "sar", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "Sarah"));
    }

    #[test]
    fn test_execute_alias_idempotent() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        assert!(execute_alias("Sarah", "sar", &db_path).is_ok());
        assert!(execute_alias("Sarah", "sar", &db_path).is_ok());

        let conn = get_connection(&db_path).unwrap();
        let sarah = EntitiesRepository::find_by_name(&conn, "sarah").unwrap().unwrap();
        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah.id.unwrap()).unwrap();
        assert_eq!(aliases.len(), 1);
    }

    #[test]
    fn test_execute_alias_empty_alias_rejected() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        let result = execute_alias("Sarah", "  ", &db_path);
        assert!(matches!(result, Err(ThoughtError::InvalidInput(_))));
    }

    #[test]
    fn test_execute_alias_same_alias_two_entities_both_succeed() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah", "John"]);

        assert!(execute_alias("Sarah", "boss", &db_path).is_ok());
        assert!(execute_alias("John", "boss", &db_path).is_ok());

        let conn = get_connection(&db_path).unwrap();
        let matches = EntityAliasesRepository::find_entities_by_alias(&conn, "boss").unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_execute_alias_by_existing_alias_name() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        execute_alias("Sarah", "sar", &db_path).unwrap();
        // Register a second alias by referencing the entity via its first alias.
        let result = execute_alias("sar", "sarita", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let sarah = EntitiesRepository::find_by_name(&conn, "sarah").unwrap().unwrap();
        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah.id.unwrap()).unwrap();
        assert_eq!(aliases, vec!["sar", "sarita"]);
    }

    #[test]
    fn test_execute_unalias_removes_alias() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        execute_alias("Sarah", "sar", &db_path).unwrap();
        let result = execute_unalias("Sarah", "sar", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let sarah = EntitiesRepository::find_by_name(&conn, "sarah").unwrap().unwrap();
        let aliases = EntityAliasesRepository::list_for_entity(&conn, sarah.id.unwrap()).unwrap();
        assert!(aliases.is_empty());
    }

    #[test]
    fn test_execute_unalias_nonexistent_alias_is_noop() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Sarah"]);

        let result = execute_unalias("Sarah", "sar", &db_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_unalias_requires_entity_to_exist() {
        let (_temp_dir, db_path) = temp_db_with_entities(&[]);

        let result = execute_unalias("Sarah", "sar", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "Sarah"));
    }
}
