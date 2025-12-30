/// Notes command implementation
use crate::errors::NoteError;
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::storage::notes_repository::NotesRepository;
use std::path::Path;

/// Execute the notes command
pub fn execute(db_path: Option<&Path>, entity_filter: Option<&str>) -> Result<(), NoteError> {
    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Get notes (filtered by entity if specified)
    let notes = if let Some(entity_name) = entity_filter {
        NotesRepository::list_by_entity(&conn, entity_name)?
    } else {
        NotesRepository::list_all(&conn)?
    };

    if notes.is_empty() {
        if let Some(entity_name) = entity_filter {
            println!("No notes found for entity: {}", entity_name);
        } else {
            println!("No notes found.");
        }
    } else {
        for note in notes {
            println!(
                "[{}] {} - {}",
                note.id.unwrap_or(0),
                note.created_at.format("%Y-%m-%d %H:%M:%S"),
                note.content
            );
        }
    }

    Ok(())
}
