/// Integration tests for entity rename
///
/// Verifies that renaming an entity rewrites literal references in linked thoughts
/// and entity descriptions, leaves unrelated content untouched, and preserves
/// `thought_entities` links (which are keyed by ID, not name).
use wetware::errors::ThoughtError;
use wetware::models::entity::Entity;
use wetware::models::thought::Thought;
use wetware::services::entity_parser::rewrite_entity_references;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::entities_repository::EntitiesRepository;
use wetware::storage::migrations::run_migrations;
use wetware::storage::thoughts_repository::ThoughtsRepository;

/// Mirrors the orchestration in `src/cli/entity_rename.rs`, without going through the CLI,
/// so the atomicity/rewrite behavior can be exercised directly against an in-memory DB.
fn rename_entity(conn: &mut rusqlite::Connection, old_name: &str, new_name: &str) -> Result<(), ThoughtError> {
    let entity = EntitiesRepository::find_by_name(conn, old_name)?.expect("entity should exist");

    let tx = conn.transaction()?;

    for other in EntitiesRepository::list_all(&tx)? {
        if let Some(desc) = &other.description {
            let rewritten = rewrite_entity_references(desc, &entity.name, new_name);
            if rewritten != *desc {
                EntitiesRepository::update_description(&tx, &other.name, Some(rewritten))?;
            }
        }
    }

    for thought in ThoughtsRepository::list_by_entity(&tx, &entity.name)? {
        let rewritten = rewrite_entity_references(&thought.content, &entity.name, new_name);
        if rewritten != thought.content {
            ThoughtsRepository::update(&tx, thought.id.unwrap(), &rewritten, thought.created_at)?;
        }
    }

    EntitiesRepository::rename(&tx, &entity.name, new_name)?;

    tx.commit()?;
    Ok(())
}

#[test]
fn test_rename_rewrites_bare_content() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let thought = Thought::new("Meeting with [Sarah]".to_string()).unwrap();
    let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
    let entity = Entity::new("Sarah".to_string());
    let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
    EntitiesRepository::link_to_thought(&conn, entity_id, thought_id).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    let thought = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(thought.content, "Meeting with [Sarah Smith]");
}

#[test]
fn test_rename_rewrites_aliased_content_preserving_alias_text() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let thought = Thought::new("Called [Sarah](sarah) again".to_string()).unwrap();
    let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
    let entity = Entity::new("Sarah".to_string());
    let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
    EntitiesRepository::link_to_thought(&conn, entity_id, thought_id).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    let thought = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(thought.content, "Called [Sarah](Sarah Smith) again");
}

#[test]
fn test_rename_leaves_unrelated_thought_untouched() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let sarah_thought = Thought::new("Meeting with [Sarah]".to_string()).unwrap();
    let sarah_thought_id = ThoughtsRepository::save(&conn, &sarah_thought).unwrap();
    let entity = Entity::new("Sarah".to_string());
    let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
    EntitiesRepository::link_to_thought(&conn, entity_id, sarah_thought_id).unwrap();

    let unrelated_thought = Thought::new("Unrelated note about [John]".to_string()).unwrap();
    let unrelated_thought_id = ThoughtsRepository::save(&conn, &unrelated_thought).unwrap();
    let john = Entity::new("John".to_string());
    let john_id = EntitiesRepository::find_or_create(&conn, &john).unwrap();
    EntitiesRepository::link_to_thought(&conn, john_id, unrelated_thought_id).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    let unrelated = ThoughtsRepository::get_by_id(&conn, unrelated_thought_id).unwrap();
    assert_eq!(unrelated.content, "Unrelated note about [John]");
}

#[test]
fn test_rename_preserves_link_by_id() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let thought = Thought::new("Meeting with [Sarah]".to_string()).unwrap();
    let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
    let entity = Entity::new("Sarah".to_string());
    let entity_id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();
    EntitiesRepository::link_to_thought(&conn, entity_id, thought_id).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    // Querying by the new name must still find the thought - the link is keyed by ID.
    let thoughts = ThoughtsRepository::list_by_entity(&conn, "sarah smith").unwrap();
    assert_eq!(thoughts.len(), 1);
    assert_eq!(thoughts[0].id, Some(thought_id));

    let link_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM thought_entities WHERE thought_id = ?1 AND entity_id = ?2",
            (thought_id, entity_id),
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(link_count, 1, "Original link row must be untouched");
}

#[test]
fn test_rename_rewrites_other_entitys_description() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let sarah = Entity::new("Sarah".to_string());
    EntitiesRepository::find_or_create(&conn, &sarah).unwrap();

    let team = Entity::new("Team".to_string());
    EntitiesRepository::find_or_create(&conn, &team).unwrap();
    EntitiesRepository::update_description(&conn, "team", Some("Led by [Sarah]".to_string())).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    let team_entity = EntitiesRepository::find_by_name(&conn, "team").unwrap().unwrap();
    assert_eq!(team_entity.description, Some("Led by [Sarah Smith]".to_string()));
}

#[test]
fn test_rename_rewrites_own_description() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let sarah = Entity::new("Sarah".to_string());
    EntitiesRepository::find_or_create(&conn, &sarah).unwrap();
    EntitiesRepository::update_description(&conn, "sarah", Some("AKA [Sarah] on the team".to_string())).unwrap();

    rename_entity(&mut conn, "sarah", "Sarah Smith").unwrap();

    let renamed = EntitiesRepository::find_by_name(&conn, "sarah smith").unwrap().unwrap();
    assert_eq!(renamed.description, Some("AKA [Sarah Smith] on the team".to_string()));
}

#[test]
fn test_rename_rollback_on_collision_leaves_everything_unchanged() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let thought = Thought::new("Meeting with [Sarah]".to_string()).unwrap();
    let thought_id = ThoughtsRepository::save(&conn, &thought).unwrap();
    let sarah = Entity::new("Sarah".to_string());
    let sarah_id = EntitiesRepository::find_or_create(&conn, &sarah).unwrap();
    EntitiesRepository::link_to_thought(&conn, sarah_id, thought_id).unwrap();

    let john = Entity::new("John".to_string());
    EntitiesRepository::find_or_create(&conn, &john).unwrap();

    // Renaming "sarah" -> "john" should fail outright (collision check happens before
    // the transaction even opens in the real orchestration); here we call the repository
    // rename directly inside a transaction to confirm no partial writes occur if it fails
    // after some rewrites were already staged.
    let result = (|| -> Result<(), ThoughtError> {
        let tx = conn.transaction()?;
        let entity = EntitiesRepository::find_by_name(&tx, "sarah")?.unwrap();
        for thought in ThoughtsRepository::list_by_entity(&tx, &entity.name)? {
            let rewritten = rewrite_entity_references(&thought.content, &entity.name, "John");
            if rewritten != thought.content {
                ThoughtsRepository::update(&tx, thought.id.unwrap(), &rewritten, thought.created_at)?;
            }
        }
        EntitiesRepository::rename(&tx, &entity.name, "John")?;
        tx.commit()?;
        Ok(())
    })();

    assert!(result.is_err());

    // Nothing should have been persisted: the thought content is unchanged (the
    // transaction was rolled back on Drop before commit).
    let thought = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(thought.content, "Meeting with [Sarah]");
    assert!(EntitiesRepository::find_by_name(&conn, "sarah").unwrap().is_some());
}
