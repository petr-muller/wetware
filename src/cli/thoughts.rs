/// Thoughts command implementation
use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Execute the thoughts command
pub fn execute(db_path: Option<&Path>, entity_filter: Option<&str>) -> Result<(), ThoughtError> {
    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Get thoughts (filtered by entity if specified)
    let thoughts = if let Some(entity_name) = entity_filter {
        ThoughtsRepository::list_by_entity(&conn, entity_name)?
    } else {
        ThoughtsRepository::list_all(&conn)?
    };

    if thoughts.is_empty() {
        if let Some(entity_name) = entity_filter {
            println!("No thoughts found for entity: {}", entity_name);
        } else {
            println!("No thoughts found.");
        }
    } else {
        for thought in thoughts {
            println!(
                "[{}] {} - {}",
                thought.id.unwrap_or(0),
                thought.created_at.format("%Y-%m-%d %H:%M:%S"),
                thought.content
            );
        }
    }

    Ok(())
}
