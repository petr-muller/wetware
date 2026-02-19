use wetware::models::entity::Entity;
/// Integration tests for edit operation atomicity
///
/// Verifies that failed edit operations leave the thought in its original state,
/// satisfying FR-015 and SC-006.
use wetware::storage::connection::get_memory_connection;
use wetware::storage::entities_repository::EntitiesRepository;
use wetware::storage::migrations::run_migrations;
use wetware::storage::thoughts_repository::ThoughtsRepository;

#[test]
fn test_update_with_valid_id_succeeds() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let thought = wetware::models::thought::Thought::new("Original".to_string()).unwrap();
    let id = ThoughtsRepository::save(&conn, &thought).unwrap();

    let result = ThoughtsRepository::update(&conn, id, "Updated", chrono::Utc::now());
    assert!(result.is_ok());

    let retrieved = ThoughtsRepository::get_by_id(&conn, id).unwrap();
    assert_eq!(retrieved.content, "Updated");
}

#[test]
fn test_update_nonexistent_thought_leaves_db_unchanged() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Save a real thought
    let thought = wetware::models::thought::Thought::new("Real thought".to_string()).unwrap();
    let real_id = ThoughtsRepository::save(&conn, &thought).unwrap();

    // Attempt to update a nonexistent ID
    let result = ThoughtsRepository::update(&conn, 9999, "Should not appear", chrono::Utc::now());
    assert!(result.is_err());

    // Real thought must be unchanged
    let retrieved = ThoughtsRepository::get_by_id(&conn, real_id).unwrap();
    assert_eq!(retrieved.content, "Real thought");
}

#[test]
fn test_unlink_then_relink_entity_associations() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Set up thought with entity links
    conn.execute(
        "INSERT INTO thoughts (content, created_at) VALUES ('Met [Alice] and [Bob]', datetime('now'))",
        [],
    )
    .unwrap();
    let thought_id = conn.last_insert_rowid();

    let alice = Entity::new("Alice".to_string());
    let bob = Entity::new("Bob".to_string());
    let alice_id = EntitiesRepository::find_or_create(&conn, &alice).unwrap();
    let bob_id = EntitiesRepository::find_or_create(&conn, &bob).unwrap();
    EntitiesRepository::link_to_thought(&conn, alice_id, thought_id).unwrap();
    EntitiesRepository::link_to_thought(&conn, bob_id, thought_id).unwrap();

    // Simulate what edit does: unlink all, then relink with new entity set
    EntitiesRepository::unlink_all_from_thought(&conn, thought_id).unwrap();

    let charlie = Entity::new("Charlie".to_string());
    let charlie_id = EntitiesRepository::find_or_create(&conn, &charlie).unwrap();
    EntitiesRepository::link_to_thought(&conn, charlie_id, thought_id).unwrap();

    // Only Charlie should be linked now
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM thought_entities WHERE thought_id = ?1",
            [thought_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "Only one entity should be linked after re-association");

    let charlie_linked: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM thought_entities te \
             INNER JOIN entities e ON te.entity_id = e.id \
             WHERE te.thought_id = ?1 AND e.name = 'charlie'",
            [thought_id],
            |row| row.get(0),
        )
        .unwrap();
    assert!(charlie_linked, "Charlie should be linked");
}
