/// Entities command implementation
use crate::errors::NoteError;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use std::path::Path;

/// Execute the entities command
pub fn execute(db_path: Option<&Path>) -> Result<(), NoteError> {
    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Get all entities (already in alphabetical order from repository)
    let entities = EntitiesRepository::list_all(&conn)?;

    if entities.is_empty() {
        println!("No entities found.");
    } else {
        for entity in entities {
            println!("{}", entity.canonical_name);
        }
    }

    Ok(())
}
