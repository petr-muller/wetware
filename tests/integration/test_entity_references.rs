/// Integration tests for entity extraction and persistence
use tempfile::TempDir;
use wetware::cli::add;
use wetware::errors::ThoughtError;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::entities_repository::EntitiesRepository;
use wetware::storage::migrations::run_migrations;
use wetware::storage::thoughts_repository::ThoughtsRepository;

#[test]
fn test_entity_extraction_and_persistence() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Add a thought with entities
    let note_content = "Meeting with [Sarah] about [project-alpha]";
    let thought = wetware::models::thought::Thought::new(note_content.to_string())?;
    let thought_id = ThoughtsRepository::save(&conn, &thought)?;

    // Extract and save entities
    let entities = wetware::services::entity_parser::extract_unique_entities(note_content);
    assert_eq!(entities.len(), 2);

    for entity_name in &entities {
        let entity = wetware::models::entity::Entity::new(entity_name.clone());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity)?;
        EntitiesRepository::link_to_thought(&conn, entity_id, thought_id)?;
    }

    // Verify entities were created
    let all_entities = EntitiesRepository::list_all(&conn)?;
    assert_eq!(all_entities.len(), 2);

    Ok(())
}

#[test]
fn test_case_insensitive_entity_matching() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create entities with different cases
    let entity1 = wetware::models::entity::Entity::new("Sarah".to_string());
    let id1 = EntitiesRepository::find_or_create(&conn, &entity1)?;

    let entity2 = wetware::models::entity::Entity::new("sarah".to_string());
    let id2 = EntitiesRepository::find_or_create(&conn, &entity2)?;

    let entity3 = wetware::models::entity::Entity::new("SARAH".to_string());
    let id3 = EntitiesRepository::find_or_create(&conn, &entity3)?;

    // All should return the same ID
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);

    // Only one entity should exist
    let all_entities = EntitiesRepository::list_all(&conn)?;
    assert_eq!(all_entities.len(), 1);

    Ok(())
}

#[test]
fn test_first_occurrence_capitalization_preserved() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create entity with specific capitalization
    let entity_first = wetware::models::entity::Entity::new("Sarah".to_string());
    EntitiesRepository::find_or_create(&conn, &entity_first)?;

    // Create same entity with different capitalization
    let entity_second = wetware::models::entity::Entity::new("SARAH".to_string());
    EntitiesRepository::find_or_create(&conn, &entity_second)?;

    // Retrieve the entity
    let found = EntitiesRepository::find_by_name(&conn, "sarah")?;
    assert!(found.is_some());
    let found_entity = found.unwrap();

    // Should preserve first occurrence capitalization
    assert_eq!(found_entity.canonical_name, "Sarah");

    Ok(())
}

#[test]
fn test_add_command_extracts_entities() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add a thought with entities
    let result = add::execute(
        "Discussed [project-alpha] with [Sarah] and [John]".to_string(),
        Some(&db_path),
    );
    assert!(result.is_ok());

    // Verify entities were extracted and saved
    let conn = wetware::storage::connection::get_connection(Some(&db_path)).unwrap();
    let entities = EntitiesRepository::list_all(&conn).unwrap();

    assert_eq!(entities.len(), 3);
    let names: Vec<String> = entities.iter().map(|e| e.canonical_name.clone()).collect();
    assert!(names.contains(&"project-alpha".to_string()));
    assert!(names.contains(&"Sarah".to_string()));
    assert!(names.contains(&"John".to_string()));
}

#[test]
fn test_add_command_no_entities() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add a thought without entities
    let result = add::execute("Regular thought without entities".to_string(), Some(&db_path));
    assert!(result.is_ok());

    // Verify no entities were created
    let conn = wetware::storage::connection::get_connection(Some(&db_path)).unwrap();
    let entities = EntitiesRepository::list_all(&conn).unwrap();
    assert_eq!(entities.len(), 0);
}

#[test]
fn test_filter_notes_by_entity() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create thoughts with entities
    let note1 = wetware::models::thought::Thought::new("Meeting with [Sarah]".to_string())?;
    let note1_id = ThoughtsRepository::save(&conn, &note1)?;
    let entity1 = wetware::models::entity::Entity::new("Sarah".to_string());
    let entity1_id = EntitiesRepository::find_or_create(&conn, &entity1)?;
    EntitiesRepository::link_to_thought(&conn, entity1_id, note1_id)?;

    let note2 = wetware::models::thought::Thought::new("Call [John]".to_string())?;
    let note2_id = ThoughtsRepository::save(&conn, &note2)?;
    let entity2 = wetware::models::entity::Entity::new("John".to_string());
    let entity2_id = EntitiesRepository::find_or_create(&conn, &entity2)?;
    EntitiesRepository::link_to_thought(&conn, entity2_id, note2_id)?;

    let note3 = wetware::models::thought::Thought::new("Email [Sarah] the report".to_string())?;
    let note3_id = ThoughtsRepository::save(&conn, &note3)?;
    EntitiesRepository::link_to_thought(&conn, entity1_id, note3_id)?;

    // Filter by Sarah
    let sarah_notes = ThoughtsRepository::list_by_entity(&conn, "sarah")?;
    assert_eq!(sarah_notes.len(), 2);
    assert!(sarah_notes.iter().any(|n| n.content.contains("Meeting with [Sarah]")));
    assert!(sarah_notes.iter().any(|n| n.content.contains("Email [Sarah]")));

    // Filter by John
    let john_notes = ThoughtsRepository::list_by_entity(&conn, "john")?;
    assert_eq!(john_notes.len(), 1);
    assert_eq!(john_notes[0].content, "Call [John]");

    Ok(())
}

#[test]
fn test_filter_notes_with_multiple_entities() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create thought with multiple entities
    let thought = wetware::models::thought::Thought::new("Meeting with [Sarah] and [John]".to_string())?;
    let thought_id = ThoughtsRepository::save(&conn, &thought)?;

    let entity1 = wetware::models::entity::Entity::new("Sarah".to_string());
    let entity1_id = EntitiesRepository::find_or_create(&conn, &entity1)?;
    EntitiesRepository::link_to_thought(&conn, entity1_id, thought_id)?;

    let entity2 = wetware::models::entity::Entity::new("John".to_string());
    let entity2_id = EntitiesRepository::find_or_create(&conn, &entity2)?;
    EntitiesRepository::link_to_thought(&conn, entity2_id, thought_id)?;

    // Should appear when filtering by either entity
    let sarah_notes = ThoughtsRepository::list_by_entity(&conn, "sarah")?;
    assert_eq!(sarah_notes.len(), 1);
    assert_eq!(sarah_notes[0].content, "Meeting with [Sarah] and [John]");

    let john_notes = ThoughtsRepository::list_by_entity(&conn, "john")?;
    assert_eq!(john_notes.len(), 1);
    assert_eq!(john_notes[0].content, "Meeting with [Sarah] and [John]");

    Ok(())
}

#[test]
fn test_list_all_entities_unique() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create multiple thoughts with same and different entities
    let note1 = wetware::models::thought::Thought::new("Meeting with [Sarah]".to_string())?;
    let note1_id = ThoughtsRepository::save(&conn, &note1)?;
    let entity1 = wetware::models::entity::Entity::new("Sarah".to_string());
    let entity1_id = EntitiesRepository::find_or_create(&conn, &entity1)?;
    EntitiesRepository::link_to_thought(&conn, entity1_id, note1_id)?;

    let note2 = wetware::models::thought::Thought::new("Call [Sarah]".to_string())?;
    let note2_id = ThoughtsRepository::save(&conn, &note2)?;
    EntitiesRepository::link_to_thought(&conn, entity1_id, note2_id)?;

    let note3 = wetware::models::thought::Thought::new("Email [John]".to_string())?;
    let note3_id = ThoughtsRepository::save(&conn, &note3)?;
    let entity2 = wetware::models::entity::Entity::new("John".to_string());
    let entity2_id = EntitiesRepository::find_or_create(&conn, &entity2)?;
    EntitiesRepository::link_to_thought(&conn, entity2_id, note3_id)?;

    // List all entities
    let all_entities = EntitiesRepository::list_all(&conn)?;

    // Should have exactly 2 unique entities
    assert_eq!(all_entities.len(), 2);

    let names: Vec<String> = all_entities.iter().map(|e| e.canonical_name.clone()).collect();
    assert!(names.contains(&"Sarah".to_string()));
    assert!(names.contains(&"John".to_string()));

    Ok(())
}

#[test]
fn test_list_all_entities_alphabetical() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Create entities in non-alphabetical order
    let entity_z = wetware::models::entity::Entity::new("Zebra".to_string());
    EntitiesRepository::find_or_create(&conn, &entity_z)?;

    let entity_a = wetware::models::entity::Entity::new("Apple".to_string());
    EntitiesRepository::find_or_create(&conn, &entity_a)?;

    let entity_m = wetware::models::entity::Entity::new("Middle".to_string());
    EntitiesRepository::find_or_create(&conn, &entity_m)?;

    // List all entities
    let all_entities = EntitiesRepository::list_all(&conn)?;

    // Should be in alphabetical order
    assert_eq!(all_entities.len(), 3);
    assert_eq!(all_entities[0].canonical_name, "Apple");
    assert_eq!(all_entities[1].canonical_name, "Middle");
    assert_eq!(all_entities[2].canonical_name, "Zebra");

    Ok(())
}
