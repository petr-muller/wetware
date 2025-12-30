/// Add command implementation
use crate::errors::ThoughtError;
use crate::models::entity::Entity;
use crate::models::thought::Thought;
use crate::services::entity_parser;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Execute the add command
pub fn execute(content: String, db_path: Option<&Path>) -> Result<(), ThoughtError> {
    // Create and validate thought
    let thought = Thought::new(content.clone())?;

    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Save thought
    let thought_id = ThoughtsRepository::save(&conn, &thought)?;

    // Extract and save entities
    let entity_names = entity_parser::extract_unique_entities(&content);
    for entity_name in &entity_names {
        let entity = Entity::new(entity_name.clone());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity)?;
        EntitiesRepository::link_to_thought(&conn, entity_id, thought_id)?;
    }

    // Success message with entity count
    if entity_names.is_empty() {
        println!("Thought added successfully (ID: {})", thought_id);
    } else {
        println!(
            "Thought added successfully (ID: {}, {} entity reference{})",
            thought_id,
            entity_names.len(),
            if entity_names.len() == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
