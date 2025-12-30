/// Contract tests for `wet thoughts` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_notes_command_lists_all_notes() {
    let temp_db = setup_temp_db();

    // Add several thoughts
    run_wet_command(&["add", "First thought"], Some(&temp_db));
    run_wet_command(&["add", "Second thought"], Some(&temp_db));
    run_wet_command(&["add", "Third thought"], Some(&temp_db));

    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("First thought"),
        "Should list first thought"
    );
    assert!(
        result.stdout.contains("Second thought"),
        "Should list second thought"
    );
    assert!(
        result.stdout.contains("Third thought"),
        "Should list third thought"
    );
}

#[test]
fn test_notes_command_empty_database() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(
        result.status, 0,
        "Command should succeed even with no thoughts"
    );
    assert!(
        result.stdout.contains("No thoughts") || result.stdout.is_empty(),
        "Should indicate no thoughts found. Got: {}",
        result.stdout
    );
}

#[test]
fn test_notes_command_chronological_order() {
    let temp_db = setup_temp_db();

    // Add thoughts in sequence
    run_wet_command(&["add", "Oldest thought"], Some(&temp_db));
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
    run_wet_command(&["add", "Middle thought"], Some(&temp_db));
    std::thread::sleep(std::time::Duration::from_millis(10));
    run_wet_command(&["add", "Newest thought"], Some(&temp_db));

    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Verify chronological order by checking positions
    let stdout = result.stdout;
    let oldest_pos = stdout
        .find("Oldest thought")
        .expect("Should contain oldest thought");
    let middle_pos = stdout
        .find("Middle thought")
        .expect("Should contain middle thought");
    let newest_pos = stdout
        .find("Newest thought")
        .expect("Should contain newest thought");

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

    // Add thoughts with different entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John] about project"], Some(&temp_db));
    run_wet_command(&["add", "Email [Sarah] the report"], Some(&temp_db));
    run_wet_command(&["add", "Thought without entities"], Some(&temp_db));

    // Filter by Sarah
    let result = run_wet_command(&["thoughts", "--on", "Sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Meeting with [Sarah]"),
        "Should include first Sarah thought"
    );
    assert!(
        result.stdout.contains("Email [Sarah] the report"),
        "Should include second Sarah thought"
    );
    assert!(
        !result.stdout.contains("Call [John]"),
        "Should not include John thought"
    );
    assert!(
        !result.stdout.contains("Thought without entities"),
        "Should not include thought without entities"
    );
}

#[test]
fn test_notes_command_filter_non_existent_entity() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Thought with [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["thoughts", "--on", "NonExistent"], Some(&temp_db));

    assert_eq!(
        result.status, 0,
        "Command should succeed even with non-existent entity"
    );
    assert!(
        result.stdout.contains("No thoughts") || result.stdout.is_empty(),
        "Should indicate no thoughts found. Got: {}",
        result.stdout
    );
}

#[test]
fn test_notes_command_filter_case_insensitive() {
    let temp_db = setup_temp_db();

    // Add thought with specific capitalization
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John]"], Some(&temp_db));

    // Filter with different capitalization
    let result = run_wet_command(&["thoughts", "--on", "sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Meeting with [Sarah]"),
        "Should find Sarah thought (case-insensitive)"
    );
    assert!(
        !result.stdout.contains("Call [John]"),
        "Should not include John thought"
    );
}
