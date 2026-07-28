/// Integration tests for entity merge
///
/// Drives `cli::entity_merge::merge` directly against an in-memory database to verify
/// that references are redirected (keeping their original wording), `thought_entities`
/// links are re-pointed, and the merged entity's description, aliases and relations
/// survive on the target.
use wetware::cli::entity_merge::merge;
use wetware::errors::ThoughtError;
use wetware::models::entity::Entity;
use wetware::models::thought::Thought;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::entities_repository::EntitiesRepository;
use wetware::storage::entity_aliases_repository::EntityAliasesRepository;
use wetware::storage::entity_relations_repository::EntityRelationsRepository;
use wetware::storage::migrations::run_migrations;
use wetware::storage::thoughts_repository::ThoughtsRepository;

/// Create an entity, returning its ID.
fn entity(conn: &rusqlite::Connection, name: &str) -> i64 {
    EntitiesRepository::find_or_create(conn, &Entity::new(name.to_string())).unwrap()
}

/// Save a thought and link it to the given entities, returning its ID.
fn thought(conn: &rusqlite::Connection, content: &str, entity_ids: &[i64]) -> i64 {
    let thought_id = ThoughtsRepository::save(conn, &Thought::new(content.to_string()).unwrap()).unwrap();
    for id in entity_ids {
        EntitiesRepository::link_to_thought(conn, *id, thought_id).unwrap();
    }
    thought_id
}

fn description_of(conn: &rusqlite::Connection, name: &str) -> Option<String> {
    EntitiesRepository::find_by_name(conn, name)
        .unwrap()
        .unwrap()
        .description
}

#[test]
fn test_merge_redirects_references_and_removes_source() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    entity(&conn, "Bob");
    let thought_id = thought(&conn, "Lunch with [Alice] and [Al](Alice)", &[alice]);

    let summary = merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(summary.source, "Alice");
    assert_eq!(summary.target, "Bob");
    assert_eq!(summary.thoughts_updated, 1);
    assert_eq!(summary.links_moved, 1);
    assert_eq!(summary.relations_dropped, 0);

    let updated = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(updated.content, "Lunch with [Alice](Bob) and [Al](Bob)");
    assert!(EntitiesRepository::find_by_name(&conn, "Alice").unwrap().is_none());
}

#[test]
fn test_merge_counts_links_moved_via_registered_alias() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    entity(&conn, "Bob");
    EntityAliasesRepository::add_alias(&conn, alice, "Ali").unwrap();
    thought(&conn, "Hi [Alice]", &[alice]);
    // No text names the source, so this one moves without being rewritten.
    thought(&conn, "Coffee with [Ali]", &[alice]);

    let summary = merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(
        summary.thoughts_updated, 1,
        "Only one thought's text mentions the source"
    );
    assert_eq!(summary.links_moved, 2, "But both thoughts move onto the target");
}

#[test]
fn test_merge_into_target_with_parentheses_is_rejected() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    entity(&conn, "Bob");
    entity(&conn, "Alice (HR)");
    let thought_id = thought(&conn, "Standup with [Bob]", &[]);

    // `[Bob](Alice (HR))` would re-parse as a bare `[Bob]`, silently unlinking the
    // thought from the survivor the next time it is edited.
    match merge(&mut conn, "Bob", "Alice (HR)") {
        Err(ThoughtError::InvalidInput(msg)) => {
            assert!(
                msg.contains("Alice (HR)"),
                "Error should name the offending entity: {msg}"
            );
            assert!(msg.contains("rename"), "Error should point at the workaround: {msg}");
        }
        other => panic!("Expected InvalidInput error, got {:?}", other),
    }

    assert!(EntitiesRepository::find_by_name(&conn, "Bob").unwrap().is_some());
    let unchanged = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(unchanged.content, "Standup with [Bob]");
}

#[test]
fn test_merge_allows_parentheses_in_the_source_name() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice (HR)");
    entity(&conn, "Bob");
    let thought_id = thought(&conn, "Coffee with [Alice (HR)]", &[alice]);

    merge(&mut conn, "Alice (HR)", "Bob").unwrap();

    // Group 1 permits parentheses, so this re-parses correctly as a reference to Bob.
    let updated = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(updated.content, "Coffee with [Alice (HR)](Bob)");
    assert_eq!(
        wetware::services::entity_parser::extract_entities(&updated.content),
        vec!["Bob"]
    );
}

#[test]
fn test_merge_repoints_thought_links_to_target() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    entity(&conn, "Bob");
    thought(&conn, "Lunch with [Alice]", &[alice]);

    merge(&mut conn, "alice", "bob").unwrap();

    let bobs_thoughts = ThoughtsRepository::list_by_entity(&conn, "bob").unwrap();
    assert_eq!(bobs_thoughts.len(), 1);
    assert_eq!(bobs_thoughts[0].content, "Lunch with [Alice](Bob)");
}

#[test]
fn test_merge_keeps_single_link_when_thought_mentions_both() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    thought(&conn, "[Alice] met [Bob]", &[alice, bob]);

    merge(&mut conn, "alice", "bob").unwrap();

    let bobs_thoughts = ThoughtsRepository::list_by_entity(&conn, "bob").unwrap();
    assert_eq!(bobs_thoughts.len(), 1, "Thought should not be listed twice");
    assert_eq!(bobs_thoughts[0].content, "[Alice](Bob) met [Bob]");
}

#[test]
fn test_merge_rewrites_references_in_other_entities_descriptions() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    entity(&conn, "Alice");
    entity(&conn, "Bob");
    entity(&conn, "Payments");
    EntitiesRepository::update_description(&conn, "payments", Some("Owned by [Alice]".to_string())).unwrap();

    let summary = merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(summary.descriptions_updated, 1);
    assert_eq!(
        description_of(&conn, "payments"),
        Some("Owned by [Alice](Bob)".to_string())
    );
}

#[test]
fn test_merge_appends_source_description_to_target() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    entity(&conn, "Alice");
    entity(&conn, "Bob");
    EntitiesRepository::update_description(&conn, "alice", Some("Runs payments.".to_string())).unwrap();
    EntitiesRepository::update_description(&conn, "bob", Some("On the platform team.".to_string())).unwrap();

    merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(
        description_of(&conn, "bob"),
        Some("On the platform team.\n\nRuns payments.".to_string())
    );
}

#[test]
fn test_merge_adopts_source_description_when_target_has_none() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    entity(&conn, "Alice");
    entity(&conn, "Bob");
    EntitiesRepository::update_description(&conn, "alice", Some("Runs payments.".to_string())).unwrap();

    merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(description_of(&conn, "bob"), Some("Runs payments.".to_string()));
}

#[test]
fn test_merge_transfers_aliases_to_target() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    EntityAliasesRepository::add_alias(&conn, alice, "Ali").unwrap();

    merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(
        EntityAliasesRepository::list_for_entity(&conn, bob).unwrap(),
        vec!["Ali"]
    );
}

#[test]
fn test_merge_skips_alias_that_names_the_target() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    EntityAliasesRepository::add_alias(&conn, alice, "Bob").unwrap();

    merge(&mut conn, "alice", "bob").unwrap();

    assert!(
        EntityAliasesRepository::list_for_entity(&conn, bob).unwrap().is_empty(),
        "An entity should not end up aliased to its own name"
    );
}

#[test]
fn test_merge_transfers_parent_and_child_relations() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    let team = entity(&conn, "Team");
    let intern = entity(&conn, "Intern");
    EntityRelationsRepository::add_relation(&conn, alice, team).unwrap();
    EntityRelationsRepository::add_relation(&conn, intern, alice).unwrap();

    merge(&mut conn, "alice", "bob").unwrap();

    let parents = EntityRelationsRepository::list_parents(&conn, bob).unwrap();
    assert_eq!(
        parents.iter().map(|e| e.canonical_name.as_str()).collect::<Vec<_>>(),
        vec!["Team"]
    );

    let children = EntityRelationsRepository::list_children(&conn, bob).unwrap();
    assert_eq!(
        children.iter().map(|e| e.canonical_name.as_str()).collect::<Vec<_>>(),
        vec!["Intern"]
    );
}

#[test]
fn test_merge_drops_relation_that_would_become_self_relation() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    // Alice is already a child of Bob - collapsing them must not create Bob -> Bob.
    EntityRelationsRepository::add_relation(&conn, alice, bob).unwrap();

    let summary = merge(&mut conn, "alice", "bob").unwrap();

    assert!(EntityRelationsRepository::list_parents(&conn, bob).unwrap().is_empty());
    assert!(EntityRelationsRepository::list_children(&conn, bob).unwrap().is_empty());
    assert_eq!(summary.relations_dropped, 1, "The dropped edge should be reported");
}

#[test]
fn test_merge_reports_dropped_cycle_relation() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    let team = entity(&conn, "Team");
    // Bob -> Team, and Team -> Alice. Collapsing Alice into Bob would close Bob -> Team -> Bob.
    EntityRelationsRepository::add_relation(&conn, bob, team).unwrap();
    EntityRelationsRepository::add_relation(&conn, team, alice).unwrap();

    let summary = merge(&mut conn, "alice", "bob").unwrap();

    assert_eq!(summary.relations_dropped, 1);
}

#[test]
fn test_merge_resolves_both_sides_through_aliases() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let bob = entity(&conn, "Bob");
    EntityAliasesRepository::add_alias(&conn, alice, "Ali").unwrap();
    EntityAliasesRepository::add_alias(&conn, bob, "Bobby").unwrap();

    let summary = merge(&mut conn, "Ali", "Bobby").unwrap();

    assert_eq!(summary.source, "Alice");
    assert_eq!(summary.target, "Bob");
}

#[test]
fn test_merge_into_itself_is_rejected() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    entity(&conn, "Alice");

    match merge(&mut conn, "alice", "Alice") {
        Err(ThoughtError::SelfMerge(name)) => assert_eq!(name, "Alice"),
        other => panic!("Expected SelfMerge error, got {:?}", other),
    }
}

#[test]
fn test_merge_missing_entity_leaves_database_untouched() {
    let mut conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    let alice = entity(&conn, "Alice");
    let thought_id = thought(&conn, "Lunch with [Alice]", &[alice]);

    match merge(&mut conn, "alice", "nobody") {
        Err(ThoughtError::EntityNotFound(name)) => assert_eq!(name, "nobody"),
        other => panic!("Expected EntityNotFound error, got {:?}", other),
    }

    assert!(EntitiesRepository::find_by_name(&conn, "Alice").unwrap().is_some());
    let unchanged = ThoughtsRepository::get_by_id(&conn, thought_id).unwrap();
    assert_eq!(unchanged.content, "Lunch with [Alice]");
}
