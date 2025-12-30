/// Add command implementation
use crate::errors::NoteError;
use crate::models::entity::Entity;
use crate::models::note::Note;
use crate::services::entity_parser;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::notes_repository::NotesRepository;
use std::path::Path;

/// Execute the add command
pub fn execute(content: String, db_path: Option<&Path>) -> Result<(), NoteError> {
    // Create and validate note
    let note = Note::new(content.clone())?;

    // Get database connection
    let conn = get_connection(db_path)?;

    // Run migrations if needed
    run_migrations(&conn)?;

    // Save note
    let note_id = NotesRepository::save(&conn, &note)?;

    // Extract and save entities
    let entity_names = entity_parser::extract_unique_entities(&content);
    for entity_name in &entity_names {
        let entity = Entity::new(entity_name.clone());
        let entity_id = EntitiesRepository::find_or_create(&conn, &entity)?;
        EntitiesRepository::link_to_note(&conn, entity_id, note_id)?;
    }

    // Success message with entity count
    if entity_names.is_empty() {
        println!("Note added successfully (ID: {})", note_id);
    } else {
        println!(
            "Note added successfully (ID: {}, {} entity reference{})",
            note_id,
            entity_names.len(),
            if entity_names.len() == 1 { "" } else { "s" }
        );
    }

    Ok(())
}
