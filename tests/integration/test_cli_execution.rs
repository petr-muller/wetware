/// Integration tests for CLI execute functions (for coverage)
use tempfile::TempDir;
use wetware::cli::{add, notes};

#[test]
fn test_add_execute_success() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute("Test note".to_string(), Some(&db_path));
    assert!(result.is_ok());
}

#[test]
fn test_add_execute_empty_fails() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute("".to_string(), Some(&db_path));
    assert!(result.is_err());
}

#[test]
fn test_add_execute_too_long_fails() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let long_content = "a".repeat(10_001);
    let result = add::execute(long_content, Some(&db_path));
    assert!(result.is_err());
}

#[test]
fn test_notes_execute_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = notes::execute(Some(&db_path), None);
    assert!(result.is_ok());
}

#[test]
fn test_notes_execute_with_notes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add some notes first
    add::execute("First note".to_string(), Some(&db_path)).unwrap();
    add::execute("Second note".to_string(), Some(&db_path)).unwrap();

    // List them
    let result = notes::execute(Some(&db_path), None);
    assert!(result.is_ok());
}

#[test]
fn test_notes_execute_with_entity_filter() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add notes with entities
    add::execute("Meeting with [Sarah]".to_string(), Some(&db_path)).unwrap();
    add::execute("Call [John]".to_string(), Some(&db_path)).unwrap();
    add::execute("Email [Sarah] the report".to_string(), Some(&db_path)).unwrap();

    // Filter by Sarah
    let result = notes::execute(Some(&db_path), Some("Sarah"));
    assert!(result.is_ok());

    // Filter by non-existent entity
    let result = notes::execute(Some(&db_path), Some("NonExistent"));
    assert!(result.is_ok());
}

#[test]
fn test_entities_execute_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = wetware::cli::entities::execute(Some(&db_path));
    assert!(result.is_ok());
}

#[test]
fn test_entities_execute_with_entities() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add notes with entities
    add::execute("Meeting with [Sarah]".to_string(), Some(&db_path)).unwrap();
    add::execute("Call [John]".to_string(), Some(&db_path)).unwrap();
    add::execute("Email [Alice]".to_string(), Some(&db_path)).unwrap();

    // List entities
    let result = wetware::cli::entities::execute(Some(&db_path));
    assert!(result.is_ok());
}
