/// Integration tests for entity description storage and retrieval
use wetware::errors::ThoughtError;
use wetware::models::entity::Entity;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::entities_repository::EntitiesRepository;
use wetware::storage::migrations::run_migrations;

// T018: Integration test for description storage/retrieval
#[test]
fn test_description_storage_and_retrieval() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Add description
    let description = Some("A systems programming language.".to_string());
    EntitiesRepository::update_description(&conn, "rust", description.clone()).unwrap();

    // Retrieve and verify
    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should exist");
    assert_eq!(retrieved.description, description, "Description should match");
}

// T018: Integration test for multi-paragraph description storage
#[test]
fn test_multiline_description_storage() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Add multi-paragraph description
    let description = Some(
        "Rust is a systems programming language.\n\nIt focuses on safety and concurrency.\n\nSee the official docs."
            .to_string(),
    );
    EntitiesRepository::update_description(&conn, "rust", description.clone()).unwrap();

    // Retrieve and verify
    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should exist");
    assert_eq!(
        retrieved.description, description,
        "Multi-paragraph description should be preserved exactly"
    );
}

// T018: Integration test for description removal (None)
#[test]
fn test_description_removal() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity with description
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    let description = Some("Original description".to_string());
    EntitiesRepository::update_description(&conn, "rust", description).unwrap();

    // Remove description
    EntitiesRepository::update_description(&conn, "rust", None).unwrap();

    // Verify removed
    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should still exist");
    assert_eq!(retrieved.description, None, "Description should be removed (None)");
}

// T018: Integration test for description update
#[test]
fn test_description_update() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity with description
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    let original = Some("Original description".to_string());
    EntitiesRepository::update_description(&conn, "rust", original).unwrap();

    // Update description
    let updated = Some("Updated description".to_string());
    EntitiesRepository::update_description(&conn, "rust", updated.clone()).unwrap();

    // Verify updated
    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should exist");
    assert_eq!(retrieved.description, updated, "Description should be updated");
}

// T019: Integration test for entity reference auto-creation in descriptions
#[test]
fn test_entity_reference_auto_creation() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Description contains entity references
    let description = Some("See [programming] and [the guide](rust-guide) for more.".to_string());

    // Note: Auto-creation happens in the CLI layer, not repository
    // This test verifies the repository behavior when we manually create referenced entities

    // Simulate CLI behavior: extract entities and create them
    use wetware::services::entity_parser;
    let references = entity_parser::extract_unique_entities(description.as_ref().unwrap());

    for ref_name in references {
        let ref_entity = Entity::new(ref_name);
        EntitiesRepository::find_or_create(&conn, &ref_entity).unwrap();
    }

    // Update description
    EntitiesRepository::update_description(&conn, "rust", description).unwrap();

    // Verify all entities exist
    let all_entities = EntitiesRepository::list_all(&conn).unwrap();
    let entity_names: Vec<String> = all_entities.iter().map(|e| e.canonical_name.clone()).collect();

    assert!(
        entity_names.contains(&"rust".to_string()),
        "Original entity should exist"
    );
    assert!(
        entity_names.contains(&"programming".to_string()),
        "Referenced entity @programming should exist"
    );
    assert!(
        entity_names.contains(&"rust-guide".to_string()),
        "Referenced entity @rust-guide should exist"
    );
}

// T019: Integration test for auto-created entities have no description
#[test]
fn test_auto_created_entities_no_description() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity via find_or_create (simulating auto-creation)
    let entity = Entity::new("auto-created".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Verify it has no description
    let retrieved = EntitiesRepository::find_by_name(&conn, "auto-created")
        .unwrap()
        .expect("Entity should exist");
    assert_eq!(
        retrieved.description, None,
        "Auto-created entity should have no description"
    );
}

// T020: Integration test for whitespace validation
#[test]
fn test_whitespace_trimming_in_storage() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Note: Whitespace trimming happens in CLI layer, not repository
    // Repository should store exactly what it receives
    // This test verifies repository preserves data as-is

    let description_with_whitespace = Some("  Description with spaces  ".to_string());
    EntitiesRepository::update_description(&conn, "rust", description_with_whitespace.clone()).unwrap();

    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should exist");

    assert_eq!(
        retrieved.description, description_with_whitespace,
        "Repository should preserve whitespace exactly (CLI layer handles trimming)"
    );
}

// T020: Integration test for empty description after trimming (CLI layer responsibility)
#[test]
fn test_whitespace_only_description_storage() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity
    let entity = Entity::new("rust".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // CLI layer should convert whitespace-only to None before calling repository
    // This test verifies None storage works correctly
    EntitiesRepository::update_description(&conn, "rust", None).unwrap();

    let retrieved = EntitiesRepository::find_by_name(&conn, "rust")
        .unwrap()
        .expect("Entity should exist");

    assert_eq!(retrieved.description, None, "Whitespace-only should be stored as None");
}

// Additional test: Case-insensitive entity name matching
#[test]
fn test_description_update_case_insensitive() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Create entity with specific capitalization
    let entity = Entity::new("RustLang".to_string());
    let _id = EntitiesRepository::find_or_create(&conn, &entity).unwrap();

    // Update using different capitalization
    let description = Some("A programming language".to_string());
    EntitiesRepository::update_description(&conn, "rustlang", description.clone()).unwrap();

    // Retrieve using yet another capitalization
    let retrieved = EntitiesRepository::find_by_name(&conn, "RUSTLANG")
        .unwrap()
        .expect("Entity should exist");

    assert_eq!(
        retrieved.description, description,
        "Description update should work case-insensitively"
    );
}

// Additional test: Error when updating non-existent entity
#[test]
fn test_description_update_nonexistent_entity() {
    let conn = get_memory_connection().unwrap();
    run_migrations(&conn).unwrap();

    // Try to update description for non-existent entity
    let result = EntitiesRepository::update_description(&conn, "nonexistent", Some("Test description".to_string()));

    assert!(result.is_err(), "Should fail for non-existent entity");
    match result {
        Err(ThoughtError::EntityNotFound(name)) => {
            assert_eq!(name, "nonexistent", "Should return correct entity name");
        }
        _ => panic!("Expected EntityNotFound error"),
    }
}
