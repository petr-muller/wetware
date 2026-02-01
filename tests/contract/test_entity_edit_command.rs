/// Contract tests for `wet entity edit` command
use crate::test_helpers::{run_wet_command, setup_temp_db};
use std::fs;

// T012: Contract test for `wet entity edit --description`
#[test]
fn test_entity_edit_inline_description() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Add description using --description flag
    let result = run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "A systems programming language.",
        ],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description updated for entity 'rust'"),
        "Should show success message. Got: {}",
        result.stdout
    );
}

// T012: Contract test for multi-paragraph inline description
#[test]
fn test_entity_edit_inline_multiline_description() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Add multi-paragraph description
    let description = "Rust is a systems programming language.\n\nIt focuses on safety and concurrency.";
    let result = run_wet_command(
        &["entity", "edit", "rust", "--description", description],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description updated for entity 'rust'"),
        "Should show success message"
    );
}

// T012: Contract test for inline description with entity references
#[test]
fn test_entity_edit_inline_description_with_entity_references() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Add description with entity references
    let result = run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "See [programming] and [the guide](rust-guide)",
        ],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");

    // Verify auto-created entities exist
    let entities_result = run_wet_command(&["entities"], Some(&temp_db));
    assert!(
        entities_result.stdout.contains("programming"),
        "Should auto-create [programming] entity"
    );
    assert!(
        entities_result.stdout.contains("rust-guide"),
        "Should auto-create rust-guide entity"
    );
}

// T013: Contract test for `wet entity edit --description-file`
#[test]
fn test_entity_edit_file_description() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Create description file
    let desc_file = temp_db.path().join("description.txt");
    fs::write(&desc_file, "Rust is a systems programming language.").expect("Failed to write description file");

    // Add description from file
    let result = run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description-file",
            desc_file.to_str().unwrap(),
        ],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description updated for entity 'rust'"),
        "Should show success message"
    );
}

// T013: Contract test for file description with multiple paragraphs
#[test]
fn test_entity_edit_file_multiline_description() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Create description file with multiple paragraphs
    let desc_file = temp_db.path().join("description.txt");
    fs::write(
        &desc_file,
        "Rust is a systems programming language.\n\nIt focuses on safety and performance.\n\nSee [programming] for more.",
    )
    .expect("Failed to write description file");

    let result = run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description-file",
            desc_file.to_str().unwrap(),
        ],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
}

// T014: Contract test for `wet entity edit` interactive editor
#[test]
#[ignore] // Requires interactive terminal - will be tested manually
fn test_entity_edit_interactive_editor() {
    // This test requires actual terminal interaction
    // Testing strategy:
    // 1. Set EDITOR to a script that writes predetermined content
    // 2. Launch wet entity edit without flags
    // 3. Verify description was saved
    //
    // Marked as #[ignore] because it's difficult to automate
}

// T015: Contract test for whitespace-only description (removal)
#[test]
fn test_entity_edit_whitespace_only_removes_description() {
    let temp_db = setup_temp_db();

    // Create entity with description
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &["entity", "edit", "rust", "--description", "Original description"],
        Some(&temp_db),
    );

    // Remove description with whitespace-only input
    let result = run_wet_command(
        &["entity", "edit", "rust", "--description", "   \n  \t  "],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description removed for entity 'rust'"),
        "Should show removal message. Got: {}",
        result.stdout
    );
}

// T015: Contract test for empty string description (removal)
#[test]
fn test_entity_edit_empty_string_removes_description() {
    let temp_db = setup_temp_db();

    // Create entity with description
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &["entity", "edit", "rust", "--description", "Original description"],
        Some(&temp_db),
    );

    // Remove description with empty string
    let result = run_wet_command(&["entity", "edit", "rust", "--description", ""], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description removed for entity 'rust'"),
        "Should show removal message"
    );
}

// T016: Contract test for entity not found error
#[test]
fn test_entity_edit_entity_not_found() {
    let temp_db = setup_temp_db();

    // Try to edit non-existent entity
    let result = run_wet_command(
        &["entity", "edit", "nonexistent", "--description", "Test description"],
        Some(&temp_db),
    );

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("Entity 'nonexistent' not found")
            || result.stdout.contains("Entity 'nonexistent' not found"),
        "Should show entity not found error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

// T017: Contract test for file not found error
#[test]
fn test_entity_edit_file_not_found() {
    let temp_db = setup_temp_db();

    // Create an entity first
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));

    // Try to use non-existent file
    let result = run_wet_command(
        &["entity", "edit", "rust", "--description-file", "/nonexistent/file.txt"],
        Some(&temp_db),
    );

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("not found") || result.stdout.contains("not found"),
        "Should show file not found error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

// Additional contract test: Case-insensitive entity name matching
#[test]
fn test_entity_edit_case_insensitive() {
    let temp_db = setup_temp_db();

    // Create entity with specific capitalization
    run_wet_command(&["add", "Learning about [RustLang]"], Some(&temp_db));

    // Edit using different capitalization
    let result = run_wet_command(
        &["entity", "edit", "rustlang", "--description", "A programming language"],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed with different capitalization");
    assert!(
        result.stdout.contains("Description updated"),
        "Should update description"
    );
}

// Additional contract test: Update existing description
#[test]
fn test_entity_edit_update_existing_description() {
    let temp_db = setup_temp_db();

    // Create entity with description
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &["entity", "edit", "rust", "--description", "Original description"],
        Some(&temp_db),
    );

    // Update with new description
    let result = run_wet_command(
        &["entity", "edit", "rust", "--description", "Updated description"],
        Some(&temp_db),
    );

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Description updated for entity 'rust'"),
        "Should show success message"
    );
}
