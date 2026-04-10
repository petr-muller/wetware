/// Integration tests for CLI execute functions (for coverage)
use tempfile::TempDir;
use wetware::cli::{add, thoughts};
use wetware::services::color_mode::ColorMode;

#[test]
fn test_add_execute_success() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute("Test thought".to_string(), None, &db_path);
    assert!(result.is_ok());
}

#[test]
fn test_add_execute_empty_fails() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute("".to_string(), None, &db_path);
    assert!(result.is_err());
}

#[test]
fn test_add_execute_too_long_fails() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let long_content = "a".repeat(10_001);
    let result = add::execute(long_content, None, &db_path);
    assert!(result.is_err());
}

#[test]
fn test_notes_execute_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = thoughts::execute(&db_path, None, ColorMode::Never);
    assert!(result.is_ok());
}

#[test]
fn test_notes_execute_with_notes() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add some thoughts first
    add::execute("First thought".to_string(), None, &db_path).unwrap();
    add::execute("Second thought".to_string(), None, &db_path).unwrap();

    // List them
    let result = thoughts::execute(&db_path, None, ColorMode::Never);
    assert!(result.is_ok());
}

#[test]
fn test_notes_execute_with_entity_filter() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add thoughts with entities
    add::execute("Meeting with [Sarah]".to_string(), None, &db_path).unwrap();
    add::execute("Call [John]".to_string(), None, &db_path).unwrap();
    add::execute("Email [Sarah] the report".to_string(), None, &db_path).unwrap();

    // Filter by Sarah
    let result = thoughts::execute(&db_path, Some("Sarah"), ColorMode::Never);
    assert!(result.is_ok());

    // Filter by non-existent entity
    let result = thoughts::execute(&db_path, Some("NonExistent"), ColorMode::Never);
    assert!(result.is_ok());
}

#[test]
fn test_entities_execute_empty_db() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = wetware::cli::entities::execute(&db_path);
    assert!(result.is_ok());
}

#[test]
fn test_entities_execute_with_entities() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add thoughts with entities
    add::execute("Meeting with [Sarah]".to_string(), None, &db_path).unwrap();
    add::execute("Call [John]".to_string(), None, &db_path).unwrap();
    add::execute("Email [Alice]".to_string(), None, &db_path).unwrap();

    // List entities
    let result = wetware::cli::entities::execute(&db_path);
    assert!(result.is_ok());
}

#[test]
fn test_add_execute_with_date() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute(
        "Backdated thought".to_string(),
        Some("2024-03-15".to_string()),
        &db_path,
    );
    assert!(result.is_ok());
}

#[test]
fn test_add_execute_with_invalid_date() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = add::execute(
        "Bad date thought".to_string(),
        Some("not-a-date".to_string()),
        &db_path,
    );
    assert!(result.is_err());
}
