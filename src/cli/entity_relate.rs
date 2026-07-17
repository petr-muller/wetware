/// Entity relate/unrelate command implementations
use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_relations_repository::EntityRelationsRepository;
use crate::storage::migrations::run_migrations;
use rusqlite::Connection;
use std::path::Path;

fn resolve_entity(conn: &Connection, name: &str) -> Result<crate::models::entity::Entity, ThoughtError> {
    let entity = EntitiesRepository::find_by_name(conn, name)?;
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
    use crate::storage::connection::get_memory_connection;

    fn setup_entity(conn: &Connection, name: &str) {
        let entity = Entity::new(name.to_string());
        EntitiesRepository::find_or_create(conn, &entity).unwrap();
    }

    fn relate_in_memory(child: &str, parent: &str) -> (Connection, Result<(), ThoughtError>) {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, child);
        setup_entity(&conn, parent);

        let child_entity = EntitiesRepository::find_by_name(&conn, child).unwrap().unwrap();
        let parent_entity = EntitiesRepository::find_by_name(&conn, parent).unwrap().unwrap();

        if child_entity.id == parent_entity.id {
            return (conn, Err(ThoughtError::SelfRelation(child.to_string())));
        }

        let would_cycle =
            EntityRelationsRepository::would_create_cycle(&conn, child_entity.id.unwrap(), parent_entity.id.unwrap())
                .unwrap();
        if would_cycle {
            return (
                conn,
                Err(ThoughtError::RelationCycle {
                    child: child_entity.canonical_name,
                    parent: parent_entity.canonical_name,
                }),
            );
        }

        let result =
            EntityRelationsRepository::add_relation(&conn, child_entity.id.unwrap(), parent_entity.id.unwrap());
        (conn, result)
    }

    #[test]
    fn test_relate_creates_relation() {
        let (conn, result) = relate_in_memory("AWS", "Amazon");
        assert!(result.is_ok());

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].canonical_name, "AWS");
    }

    #[test]
    fn test_relate_self_relation_rejected() {
        let (_conn, result) = relate_in_memory("Amazon", "Amazon");
        assert!(matches!(result, Err(ThoughtError::SelfRelation(_))));
    }

    #[test]
    fn test_relate_cycle_rejected() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        setup_entity(&conn, "AWS");

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();

        EntityRelationsRepository::add_relation(&conn, aws.id.unwrap(), amazon.id.unwrap()).unwrap();

        let would_cycle =
            EntityRelationsRepository::would_create_cycle(&conn, amazon.id.unwrap(), aws.id.unwrap()).unwrap();
        assert!(would_cycle);
    }

    #[test]
    fn test_relate_idempotent() {
        let (conn, first) = relate_in_memory("AWS", "Amazon");
        assert!(first.is_ok());

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();

        let second = EntityRelationsRepository::add_relation(&conn, aws.id.unwrap(), amazon.id.unwrap());
        assert!(second.is_ok());

        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_unrelate_removes_relation() {
        let (conn, result) = relate_in_memory("AWS", "Amazon");
        assert!(result.is_ok());

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();

        EntityRelationsRepository::remove_relation(&conn, aws.id.unwrap(), amazon.id.unwrap()).unwrap();

        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_unrelate_nonexistent_relation_is_noop() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        setup_entity(&conn, "AWS");

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();

        let result = EntityRelationsRepository::remove_relation(&conn, aws.id.unwrap(), amazon.id.unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_relate_end_to_end() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        setup_entity(&conn, "AWS");
        drop(conn);

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
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        drop(conn);

        let result = execute_relate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "AWS"));
    }

    #[test]
    fn test_execute_relate_missing_parent_errors() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "AWS");
        drop(conn);

        let result = execute_relate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "Amazon"));
    }

    #[test]
    fn test_execute_relate_cycle_rolls_back() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        setup_entity(&conn, "AWS");
        drop(conn);

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
    fn test_execute_unrelate_end_to_end() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        setup_entity(&conn, "AWS");
        drop(conn);

        execute_relate("AWS", "Amazon", &db_path).unwrap();
        let result = execute_unrelate("AWS", "Amazon", &db_path);
        assert!(result.is_ok());

        let conn = get_connection(&db_path).unwrap();
        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let children = EntityRelationsRepository::list_children(&conn, amazon.id.unwrap()).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_execute_unrelate_requires_both_entities_to_exist() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let conn = get_connection(&db_path).unwrap();
        run_migrations(&conn).unwrap();
        setup_entity(&conn, "Amazon");
        drop(conn);

        let result = execute_unrelate("AWS", "Amazon", &db_path);
        assert!(matches!(result, Err(ThoughtError::EntityNotFound(name)) if name == "AWS"));
    }
}
