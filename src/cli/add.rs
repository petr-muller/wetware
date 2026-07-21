/// Add command implementation
use crate::errors::ThoughtError;
use crate::models::thought::Thought;
use crate::services::{entity_parser, entity_resolution};
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use chrono::NaiveDate;
use std::path::Path;

/// Execute the add command
pub fn execute(content: String, date: Option<String>, db_path: &Path) -> Result<(), ThoughtError> {
    // Create and validate thought
    let thought = if let Some(ref date_str) = date {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            ThoughtError::InvalidInput(format!("Invalid date format '{}'. Expected YYYY-MM-DD.", date_str))
        })?;
        let datetime = naive.and_hms_opt(0, 0, 0).unwrap().and_utc();
        Thought::new_with_date(content.clone(), datetime)?
    } else {
        Thought::new(content.clone())?
    };

    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Save thought
    let thought_id = ThoughtsRepository::save(&conn, &thought)?;

    // Extract and save entities
    let entity_names = entity_parser::extract_unique_entities(&content);
    for entity_name in &entity_names {
        if let Some(entity_id) = entity_resolution::resolve_or_create_entity(&conn, entity_name)? {
            EntitiesRepository::link_to_thought(&conn, entity_id, thought_id)?;
        }
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
