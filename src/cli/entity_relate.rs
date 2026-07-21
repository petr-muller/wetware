/// Entity relate/unrelate command implementations
use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_relations_repository::EntityRelationsRepository;
use crate::storage::migrations::run_migrations;
use rusqlite::Connection;
use std::path::Path;

fn resolve_entity(conn: &Connection, name: &str) -> Result<crate::models::entity::Entity, ThoughtError> {
    let entity = EntitiesRepository::resolve(conn, name)?;
    entity.ok_or_else(|| {
        eprintln!("Error: Entity '{}' not found", name);
        eprintln!();
        eprintln!("Hint: Create the entity first by referencing it in a thought:");
        eprintln!("  wet add \"Learning about [{}] today\"", name);
        ThoughtError::EntityNotFound(name.to_string())
    })
}

/// Execute the entity relate command
///
/// Marks `entity_name` as a child of `parent_name`. Both entities must already exist.
/// Idempotent: relating an already-related pair succeeds without error. Rejects
/// self-relation and any relation that would create a cycle in the parent/child graph.
///
/// # Arguments
/// * `entity_name` - Entity to mark as a child (case-insensitive)
/// * `parent_name` - Parent entity name (case-insensitive)
/// * `db_path` - Database path
pub fn execute_relate(entity_name: &str, parent_name: &str, db_path: &Path) -> Result<(), ThoughtError> {
    let mut conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let child = resolve_entity(&conn, entity_name)?;
    let parent = resolve_entity(&conn, parent_name)?;

    if child.id == parent.id {
        return Err(ThoughtError::SelfRelation(entity_name.to_string()));
    }

    let tx = conn.transaction()?;

    if EntityRelationsRepository::would_create_cycle(&tx, child.id.unwrap(), parent.id.unwrap())? {
        return Err(ThoughtError::RelationCycle {
            child: child.canonical_name,
            parent: parent.canonical_name,
        });
    }

    EntityRelationsRepository::add_relation(&tx, child.id.unwrap(), parent.id.unwrap())?;

    tx.commit()?;

    println!("Entity '{}' is now a child of '{}'.", entity_name, parent_name);

    Ok(())
}

/// Execute the entity unrelate command
///
/// Removes the child/parent relation between `entity_name` and `parent_name`. Both
/// entities must already exist. Idempotent: succeeds even if the relation didn't exist.
///
/// # Arguments
/// * `entity_name` - Child entity name (case-insensitive)
/// * `parent_name` - Parent entity name (case-insensitive)
/// * `db_path` - Database path
pub fn execute_unrelate(entity_name: &str, parent_name: &str, db_path: &Path) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let child = resolve_entity(&conn, entity_name)?;
    let parent = resolve_entity(&conn, parent_name)?;

    EntityRelationsRepository::remove_relation(&conn, child.id.unwrap(), parent.id.unwrap())?;

    println!("Removed '{}' as a child of '{}'.", entity_name, parent_name);

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

    /// Create a temp-file-backed database with the given entities already present,
    /// so tests exercise `execute_relate`/`execute_unrelate` themselves (the real
    /// command functions) rather than reimplementing their logic against an
    /// in-memory connection.
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
    fn test_execute_relate_resolves_alias_on_both_sides() {
        use crate::storage::entity_aliases_repository::EntityAliasesRepository;

        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        let conn = get_connection(&db_path).unwrap();
        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();
        EntityAliasesRepository::add_alias(&conn, amazon.id.unwrap(), "bigco").unwrap();
        EntityAliasesRepository::add_alias(&conn, aws.id.unwrap(), "cloud").unwrap();
        drop(conn);

        let result = execute_relate("cloud", "bigco", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].canonical_name, "AWS");
    }

    #[test]
    fn test_execute_relate_creates_relation() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        let result = execute_relate("AWS", "Amazon", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].canonical_name, "AWS");
    }

    #[test]
    fn test_execute_relate_missing_child_errors() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon"]);

        let result = execute_relate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "AWS"));
    }

    #[test]
    fn test_execute_relate_missing_parent_errors() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["AWS"]);

        let result = execute_relate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "Amazon"));
    }

    #[test]
    fn test_execute_relate_self_relation_rejected() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon"]);

        let result = execute_relate("Amazon", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::SelfRelation(_))));
    }

    #[test]
    fn test_execute_relate_idempotent() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        assert!(execute_relate("AWS", "Amazon", &db_path).is_ok());
        assert!(execute_relate("AWS", "Amazon", &db_path).is_ok());

        let conn = get_connection(&db_path).unwrap();
        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_execute_relate_cycle_rolls_back() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        execute_relate("AWS", "Amazon", &db_path).unwrap();

        let result = execute_relate("Amazon", "AWS", &db_path);
        assert!(matches!(result, Err(ThoughtError::RelationCycle { .. })));

        let conn = get_connection(&db_path).unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();
        let amazon_parents = EntityRelationsRepository::list_parents(&conn, aws.id.unwrap()).unwrap();
        assert_eq!(amazon_parents.len(), 1);
        assert_eq!(amazon_parents[0].canonical_name, "Amazon");
    }

    #[test]
    fn test_execute_unrelate_removes_relation() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        execute_relate("AWS", "Amazon", &db_path).unwrap();
        let result = execute_unrelate("AWS", "Amazon", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_execute_unrelate_nonexistent_relation_is_noop() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon", "AWS"]);

        let result = execute_unrelate("AWS", "Amazon", &db_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_unrelate_requires_both_entities_to_exist() {
        let (_temp_dir, db_path) = temp_db_with_entities(&["Amazon"]);

        let result = execute_unrelate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "AWS"));
    }
}
