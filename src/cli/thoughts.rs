/// Thoughts command implementation
use crate::errors::ThoughtError;
use crate::services::color_mode::ColorMode;
use crate::services::entity_styler::EntityStyler;
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Execute the thoughts command
///
/// # Arguments
///
/// * `db_path` - Optional path to the database file
/// * `entity_filter` - Optional entity name to filter thoughts by
/// * `color_mode` - Controls whether output should be styled with colors
pub fn execute(db_path: Option<&Path>, entity_filter: Option<&str>, color_mode: ColorMode) -> Result<(), ThoughtError> {
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
        // Create entity styler based on color mode
        let use_colors = color_mode.should_use_colors();
        let mut styler = EntityStyler::new(use_colors);

        for thought in thoughts {
            let styled_content = styler.render_content(&thought.content);
            println!(
                "[{}] {} - {}",
                thought.id.unwrap_or(0),
                thought.created_at.format("%Y-%m-%d %H:%M:%S"),
                styled_content
            );
        }
    }

    Ok(())
}
