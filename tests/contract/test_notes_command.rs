/// Contract tests for `wet notes` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_notes_command_lists_all_notes() {
    let temp_db = setup_temp_db();

    // Add several notes
    run_wet_command(&["add", "First note"], Some(&temp_db));
    run_wet_command(&["add", "Second note"], Some(&temp_db));
    run_wet_command(&["add", "Third note"], Some(&temp_db));

    let result = run_wet_command(&["notes"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("First note"),
        "Should list first note"
    );
    assert!(
        result.stdout.contains("Second note"),
        "Should list second note"
    );
    assert!(
        result.stdout.contains("Third note"),
        "Should list third note"
    );
}

#[test]
fn test_notes_command_empty_database() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["notes"], Some(&temp_db));

    assert_eq!(
        result.status, 0,
        "Command should succeed even with no notes"
    );
    assert!(
        result.stdout.contains("No notes") || result.stdout.is_empty(),
        "Should indicate no notes found. Got: {}",
        result.stdout
    );
}

#[test]
fn test_notes_command_chronological_order() {
    let temp_db = setup_temp_db();

    // Add notes in sequence
    run_wet_command(&["add", "Oldest note"], Some(&temp_db));
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    run_wet_command(&["add", "Middle note"], Some(&temp_db));
    std::thread::sleep(std::time::Duration::from_millis(10));
    run_wet_command(&["add", "Newest note"], Some(&temp_db));

    let result = run_wet_command(&["notes"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Verify chronological order by checking positions
    let stdout = result.stdout;
    let oldest_pos = stdout
        .find("Oldest note")
        .expect("Should contain oldest note");
    let middle_pos = stdout
        .find("Middle note")
        .expect("Should contain middle note");
    let newest_pos = stdout
        .find("Newest note")
        .expect("Should contain newest note");

    assert!(
        oldest_pos < middle_pos,
        "Oldest should appear before middle"
    );
    assert!(
        middle_pos < newest_pos,
        "Middle should appear before newest"
    );
}

#[test]
fn test_notes_command_filter_by_entity() {
    let temp_db = setup_temp_db();

    // Add notes with different entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John] about project"], Some(&temp_db));
    run_wet_command(&["add", "Email [Sarah] the report"], Some(&temp_db));
    run_wet_command(&["add", "Note without entities"], Some(&temp_db));

    // Filter by Sarah
    let result = run_wet_command(&["notes", "--on", "Sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Meeting with [Sarah]"),
        "Should include first Sarah note"
    );
    assert!(
        result.stdout.contains("Email [Sarah] the report"),
        "Should include second Sarah note"
    );
    assert!(
        !result.stdout.contains("Call [John]"),
        "Should not include John note"
    );
    assert!(
        !result.stdout.contains("Note without entities"),
        "Should not include note without entities"
    );
}

#[test]
fn test_notes_command_filter_non_existent_entity() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Note with [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["notes", "--on", "NonExistent"], Some(&temp_db));

    assert_eq!(
        result.status, 0,
        "Command should succeed even with non-existent entity"
    );
    assert!(
        result.stdout.contains("No notes") || result.stdout.is_empty(),
        "Should indicate no notes found. Got: {}",
        result.stdout
    );
}

#[test]
fn test_notes_command_filter_case_insensitive() {
    let temp_db = setup_temp_db();

    // Add note with specific capitalization
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John]"], Some(&temp_db));

    // Filter with different capitalization
    let result = run_wet_command(&["notes", "--on", "sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Meeting with [Sarah]"),
        "Should find Sarah note (case-insensitive)"
    );
    assert!(
        !result.stdout.contains("Call [John]"),
        "Should not include John note"
    );
}
