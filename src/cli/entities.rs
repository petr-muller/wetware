/// Entities command implementation
use crate::errors::ThoughtError;
use crate::services::description_formatter;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use std::path::Path;

/// Execute the entities command
///
/// Lists all entities in alphabetical order. If terminal width >= 60 characters,
/// entities with descriptions show ellipsized previews on a single line.
///
/// # Arguments
/// * `db_path` - Optional database path
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(ThoughtError)` - Database or other errors
///
/// # Output Format
/// Wide terminal (>= 60 chars):
/// ```text
/// entity-name - Preview of description text…
/// entity-without-description
/// ```
///
/// Narrow terminal (< 60 chars):
/// ```text
/// entity-name
/// entity-without-description
/// ```
pub fn execute(db_path: Option<&Path>) -> Result<(), ThoughtError> {
    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Get all entities (already in alphabetical order from repository)
    let entities = EntitiesRepository::list_all(&conn)?;

    if entities.is_empty() {
        println!("No entities found.");
        return Ok(());
    }

    // T053: Detect terminal width
    let terminal_width = description_formatter::get_terminal_width();

    // T054: Check if terminal is too narrow for previews
    const MIN_WIDTH_FOR_PREVIEW: usize = 60;
    let show_previews = terminal_width >= MIN_WIDTH_FOR_PREVIEW;

    for entity in entities {
        if show_previews && entity.has_description() {
            // T055: Generate and display preview for entities with descriptions
            let preview = description_formatter::generate_preview(
                entity.description_or_empty(),
                &entity.canonical_name,
                terminal_width,
            );

            if !preview.is_empty() {
                println!("{} - {}", entity.canonical_name, preview);
            } else {
                // Preview empty (entity name too long or other issue) - show name only
                println!("{}", entity.canonical_name);
            }
        } else {
            // T056: Display entity name only (no description or narrow terminal)
            println!("{}", entity.canonical_name);
        }
    }

    Ok(())
}
