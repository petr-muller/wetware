/// Repository for directed entity relations (parent/child) persistence
use crate::errors::ThoughtError;
use crate::models::entity::Entity;
use rusqlite::Connection;

/// Recursive CTE computing all entity ids reachable as descendants of `?1`
/// (following child edges downward), including `?1` itself.
const DESCENDANTS_CTE: &str = "
    WITH RECURSIVE descendants(id) AS (
        SELECT ?1
        UNION
        SELECT er.child_id FROM entity_relations er JOIN descendants d ON er.parent_id = d.id
    )
";

/// Entity relations repository for database operations
pub struct EntityRelationsRepository;

impl EntityRelationsRepository {
    /// Mark `child_id` as a child of `parent_id`. Idempotent - a no-op if the
    /// relation already exists.
    pub fn add_relation(conn: &Connection, child_id: i64, parent_id: i64) -> Result<(), ThoughtError> {
        conn.execute(
            "INSERT OR IGNORE INTO entity_relations (child_id, parent_id) VALUES (?1, ?2)",
            (child_id, parent_id),
        )?;
        Ok(())
    }

    /// Remove the child/parent relation if present. Safe to call even if no such
    /// relation exists (no-op).
    pub fn remove_relation(conn: &Connection, child_id: i64, parent_id: i64) -> Result<(), ThoughtError> {
        conn.execute(
            "DELETE FROM entity_relations WHERE child_id = ?1 AND parent_id = ?2",
            (child_id, parent_id),
        )?;
        Ok(())
    }

    /// True if `parent_id` is already reachable as a descendant of `child_id`,
    /// meaning adding a `child_id` -> `parent_id` edge would make `child_id` its
    /// own (transitive) ancestor.
    pub fn would_create_cycle(conn: &Connection, child_id: i64, parent_id: i64) -> Result<bool, ThoughtError> {
        let sql = format!("{DESCENDANTS_CTE} SELECT EXISTS(SELECT 1 FROM descendants WHERE id = ?2)");
        let exists: bool = conn.query_row(&sql, (child_id, parent_id), |row| row.get(0))?;
        Ok(exists)
    }

    /// Direct (non-transitive) parents of the given entity.
    pub fn list_parents(conn: &Connection, entity_id: i64) -> Result<Vec<Entity>, ThoughtError> {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.name, e.canonical_name, e.description
             FROM entities e
             INNER JOIN entity_relations er ON er.parent_id = e.id
             WHERE er.child_id = ?1
             ORDER BY e.canonical_name ASC",
        )?;

        let entities = stmt
            .query_map([entity_id], Self::row_to_entity)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    /// Direct (non-transitive) children of the given entity.
    pub fn list_children(conn: &Connection, entity_id: i64) -> Result<Vec<Entity>, ThoughtError> {
        let mut stmt = conn.prepare(
            "SELECT e.id, e.name, e.canonical_name, e.description
             FROM entities e
             INNER JOIN entity_relations er ON er.child_id = e.id
             WHERE er.parent_id = ?1
             ORDER BY e.canonical_name ASC",
        )?;

        let entities = stmt
            .query_map([entity_id], Self::row_to_entity)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    /// All relation edges as `(child_id, parent_id)` pairs.
    pub fn list_all_edges(conn: &Connection) -> Result<Vec<(i64, i64)>, ThoughtError> {
        let mut stmt = conn.prepare("SELECT child_id, parent_id FROM entity_relations")?;

        let edges = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(edges)
    }

    fn row_to_entity(row: &rusqlite::Row) -> rusqlite::Result<Entity> {
        Ok(Entity {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            canonical_name: row.get(2)?,
            description: row.get(3)?,
        })
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
    fn test_add_relation_creates_parent_child_link() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");
        let amazon = make_entity(&conn, "Amazon");

        EntityRelationsRepository::add_relation(&conn, aws, amazon).unwrap();

        let children = EntityRelationsRepository::list_children(&conn, amazon).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].canonical_name, "AWS");

        let parents = EntityRelationsRepository::list_parents(&conn, aws).unwrap();
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].canonical_name, "Amazon");
    }

    #[test]
    fn test_add_relation_idempotent() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");
        let amazon = make_entity(&conn, "Amazon");

        EntityRelationsRepository::add_relation(&conn, aws, amazon).unwrap();
        EntityRelationsRepository::add_relation(&conn, aws, amazon).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entity_relations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_remove_relation_removes_existing() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");
        let amazon = make_entity(&conn, "Amazon");

        EntityRelationsRepository::add_relation(&conn, aws, amazon).unwrap();
        EntityRelationsRepository::remove_relation(&conn, aws, amazon).unwrap();

        let children = EntityRelationsRepository::list_children(&conn, amazon).unwrap();
        assert!(children.is_empty());
    }

    #[test]
    fn test_remove_relation_nonexistent_is_noop() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");
        let amazon = make_entity(&conn, "Amazon");

        let result = EntityRelationsRepository::remove_relation(&conn, aws, amazon);
        assert!(result.is_ok());
    }

    #[test]
    fn test_would_create_cycle_direct() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let a = make_entity(&conn, "A");
        let b = make_entity(&conn, "B");

        // B is a child of A
        EntityRelationsRepository::add_relation(&conn, b, a).unwrap();

        // Making A a child of B would create a cycle
        assert!(EntityRelationsRepository::would_create_cycle(&conn, a, b).unwrap());
    }

    #[test]
    fn test_would_create_cycle_indirect_chain() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let a = make_entity(&conn, "A");
        let b = make_entity(&conn, "B");
        let c = make_entity(&conn, "C");

        // C child-of B child-of A
        EntityRelationsRepository::add_relation(&conn, b, a).unwrap();
        EntityRelationsRepository::add_relation(&conn, c, b).unwrap();

        // Making A a child of C would create a cycle (A -> C -> B -> A)
        assert!(EntityRelationsRepository::would_create_cycle(&conn, a, c).unwrap());
    }

    #[test]
    fn test_would_create_cycle_unrelated_entities() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let a = make_entity(&conn, "A");
        let b = make_entity(&conn, "B");

        assert!(!EntityRelationsRepository::would_create_cycle(&conn, a, b).unwrap());
    }

    #[test]
    fn test_would_create_cycle_multi_parent_dag() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let amazon = make_entity(&conn, "Amazon");
        let satellite_domain = make_entity(&conn, "Satellite Connectivity Domain");
        let amazon_leo = make_entity(&conn, "Amazon Leo");
        let other = make_entity(&conn, "Other");

        // Amazon Leo is a child of both Amazon and satellite connectivity domain
        EntityRelationsRepository::add_relation(&conn, amazon_leo, amazon).unwrap();
        EntityRelationsRepository::add_relation(&conn, amazon_leo, satellite_domain).unwrap();

        // Making Amazon a child of Amazon Leo would create a cycle
        assert!(EntityRelationsRepository::would_create_cycle(&conn, amazon, amazon_leo).unwrap());

        // Making an unrelated third entity a further child of Amazon Leo is fine
        assert!(!EntityRelationsRepository::would_create_cycle(&conn, other, amazon_leo).unwrap());
    }

    #[test]
    fn test_list_all_edges() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");
        let amazon = make_entity(&conn, "Amazon");

        EntityRelationsRepository::add_relation(&conn, aws, amazon).unwrap();

        let edges = EntityRelationsRepository::list_all_edges(&conn).unwrap();
        assert_eq!(edges, vec![(aws, amazon)]);
    }

    #[test]
    fn test_list_parents_and_children_empty_when_no_relations() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let aws = make_entity(&conn, "AWS");

        assert!(EntityRelationsRepository::list_parents(&conn, aws).unwrap().is_empty());
        assert!(EntityRelationsRepository::list_children(&conn, aws).unwrap().is_empty());
    }
}
