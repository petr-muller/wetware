/// Integration tests for thought persistence and retrieval
use wetware::errors::ThoughtError;
use wetware::models::thought::Thought;
use wetware::storage::connection::get_memory_connection;
use wetware::storage::migrations::run_migrations;
use wetware::storage::thoughts_repository::ThoughtsRepository;

#[test]
fn test_save_and_retrieve_note() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let thought = Thought::new("Test thought content".to_string())?;
    let saved_id = ThoughtsRepository::save(&conn, &thought)?;

    let retrieved = ThoughtsRepository::get_by_id(&conn, saved_id)?;
    assert_eq!(retrieved.content, "Test thought content");
    assert_eq!(retrieved.id, Some(saved_id));

    Ok(())
}

#[test]
fn test_list_all_notes() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    // Add multiple thoughts
    let note1 = Thought::new("First thought".to_string())?;
    let note2 = Thought::new("Second thought".to_string())?;
    let note3 = Thought::new("Third thought".to_string())?;

    ThoughtsRepository::save(&conn, &note1)?;
    ThoughtsRepository::save(&conn, &note2)?;
    ThoughtsRepository::save(&conn, &note3)?;

    let all_notes = ThoughtsRepository::list_all(&conn)?;

    assert_eq!(all_notes.len(), 3);
    assert_eq!(all_notes[0].content, "First thought");
    assert_eq!(all_notes[1].content, "Second thought");
    assert_eq!(all_notes[2].content, "Third thought");

    Ok(())
}

#[test]
fn test_list_notes_chronological_order() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let note1 = Thought::new("Oldest".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let note2 = Thought::new("Middle".to_string())?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    let note3 = Thought::new("Newest".to_string())?;

    ThoughtsRepository::save(&conn, &note1)?;
    ThoughtsRepository::save(&conn, &note2)?;
    ThoughtsRepository::save(&conn, &note3)?;

    let all_notes = ThoughtsRepository::list_all(&conn)?;

    assert_eq!(all_notes.len(), 3);
    // Should be in chronological order (oldest first)
    assert!(all_notes[0].created_at <= all_notes[1].created_at);
    assert!(all_notes[1].created_at <= all_notes[2].created_at);

    Ok(())
}

#[test]
fn test_empty_database_returns_empty_list() -> Result<(), ThoughtError> {
    let conn = get_memory_connection()?;
    run_migrations(&conn)?;

    let all_notes = ThoughtsRepository::list_all(&conn)?;
    assert!(all_notes.is_empty());

    Ok(())
}
