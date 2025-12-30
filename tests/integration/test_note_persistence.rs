/// Integration tests for note persistence and retrieval
use wetware::errors::NoteError;
use wetware::models::note::Note;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::migrations::run_migrations;
use wetware::storage::notes_repository::NotesRepository;

#[test]
fn test_save_and_retrieve_note() -> Result<(), NoteError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let note = Note::new("Test note content".to_string())?;
    let saved_id = NotesRepository::save(&conn, &note)?;

    let retrieved = NotesRepository::get_by_id(&conn, saved_id)?;
    assert_eq!(retrieved.content, "Test note content");
    assert_eq!(retrieved.id, Some(saved_id));

    Ok(())
}

#[test]
fn test_list_all_notes() -> Result<(), NoteError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Add multiple notes
    let note1 = Note::new("First note".to_string())?;
    let note2 = Note::new("Second note".to_string())?;
    let note3 = Note::new("Third note".to_string())?;

    NotesRepository::save(&conn, &note1)?;
    NotesRepository::save(&conn, &note2)?;
    NotesRepository::save(&conn, &note3)?;

    let all_notes = NotesRepository::list_all(&conn)?;

    assert_eq!(all_notes.len(), 3);
    assert_eq!(all_notes[0].content, "First note");
    assert_eq!(all_notes[1].content, "Second note");
    assert_eq!(all_notes[2].content, "Third note");

    Ok(())
}

#[test]
fn test_list_notes_chronological_order() -> Result<(), NoteError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let note1 = Note::new("Oldest".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let note2 = Note::new("Middle".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let note3 = Note::new("Newest".to_string())?;

    NotesRepository::save(&conn, &note1)?;
    NotesRepository::save(&conn, &note2)?;
    NotesRepository::save(&conn, &note3)?;

    let all_notes = NotesRepository::list_all(&conn)?;

    assert_eq!(all_notes.len(), 3);
    // Should be in chronological order (oldest first)
    assert!(all_notes[0].created_at <= all_notes[1].created_at);
    assert!(all_notes[1].created_at <= all_notes[2].created_at);

    Ok(())
}

#[test]
fn test_empty_database_returns_empty_list() -> Result<(), NoteError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let all_notes = NotesRepository::list_all(&conn)?;
    assert!(all_notes.is_empty());

    Ok(())
}
