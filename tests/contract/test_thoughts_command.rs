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
    assert!(result.stdout.contains("First thought"), "Should list first thought");
    assert!(result.stdout.contains("Second thought"), "Should list second thought");
    assert!(result.stdout.contains("Third thought"), "Should list third thought");
}

#[test]
fn test_notes_command_empty_database() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed even with no thoughts");
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
    let oldest_pos = stdout.find("Oldest thought").expect("Should contain oldest thought");
    let middle_pos = stdout.find("Middle thought").expect("Should contain middle thought");
    let newest_pos = stdout.find("Newest thought").expect("Should contain newest thought");

    assert!(oldest_pos < middle_pos, "Oldest should appear before middle");
    assert!(middle_pos < newest_pos, "Middle should appear before newest");
}

#[test]
fn test_notes_command_filter_by_entity() {
    let temp_db = setup_temp_db();

    // Add thoughts with different entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John] about project"], Some(&temp_db));
    run_wet_command(&["add", "Email [Sarah] the report"], Some(&temp_db));
    run_wet_command(&["add", "Thought without entities"], Some(&temp_db));

    // Filter by Sarah (output will have brackets stripped)
    let result = run_wet_command(&["thoughts", "--on", "Sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Note: Entity brackets are now stripped in output
    assert!(
        result.stdout.contains("Meeting with Sarah"),
        "Should include first Sarah thought (without brackets). Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("Email Sarah the report"),
        "Should include second Sarah thought (without brackets)"
    );
    assert!(!result.stdout.contains("Call John"), "Should not include John thought");
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

    assert_eq!(result.status, 0, "Command should succeed even with non-existent entity");
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
    // Note: Entity brackets are now stripped in output
    assert!(
        result.stdout.contains("Meeting with Sarah"),
        "Should find Sarah thought (case-insensitive, without brackets). Got: {}",
        result.stdout
    );
    assert!(!result.stdout.contains("Call John"), "Should not include John thought");
}

// T027: Piped output contains no ANSI escape codes
#[test]
fn test_notes_command_piped_output_no_ansi_codes() {
    let temp_db = setup_temp_db();

    // Add thoughts with entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    // Run command - when run via Command::output(), stdout is not a TTY
    // so colors should be automatically disabled
    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Should not contain ANSI escape codes (ESC character is \x1b)
    assert!(
        !result.stdout.contains('\x1b'),
        "Piped output should not contain ANSI escape codes. Got: {:?}",
        result.stdout
    );
    // But should still contain the entity name (without brackets)
    assert!(result.stdout.contains("Sarah"), "Should contain entity name");
}

// T028: Output contains entity text without brackets when piped
#[test]
fn test_notes_command_piped_output_strips_brackets() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah] about [project-alpha]"], Some(&temp_db));

    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Entity names should be present without brackets
    assert!(result.stdout.contains("Sarah"), "Should contain entity name Sarah");
    assert!(
        result.stdout.contains("project-alpha"),
        "Should contain entity name project-alpha"
    );
    // Brackets should be stripped
    assert!(
        !result.stdout.contains("[Sarah]"),
        "Should not contain bracketed entity. Got: {}",
        result.stdout
    );
    assert!(
        !result.stdout.contains("[project-alpha]"),
        "Should not contain bracketed entity"
    );
}

// T031: --color=never disables styling in terminal
#[test]
fn test_notes_command_color_never_disables_styling() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    // Use --color=never to force plain output
    let result = run_wet_command(&["thoughts", "--color=never"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Should not contain ANSI escape codes
    assert!(
        !result.stdout.contains('\x1b'),
        "--color=never should disable ANSI codes. Got: {:?}",
        result.stdout
    );
    // Should still contain entity name without brackets
    assert!(result.stdout.contains("Sarah"), "Should contain entity name");
}

// T032: --color=always enables styling when piped
#[test]
fn test_notes_command_color_always_enables_styling() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    // Use --color=always to force colored output even when piped
    let result = run_wet_command(&["thoughts", "--color=always"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Should contain ANSI escape codes (ESC character is \x1b)
    assert!(
        result.stdout.contains('\x1b'),
        "--color=always should include ANSI codes. Got: {:?}",
        result.stdout
    );
    // Should contain entity name
    assert!(result.stdout.contains("Sarah"), "Should contain entity name");
}

// T036: --color=always with redirect produces ANSI codes
#[test]
fn test_notes_command_color_always_with_multiple_entities() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah] about [project]"], Some(&temp_db));
    run_wet_command(&["add", "Call [Sarah] tomorrow"], Some(&temp_db));

    // Force colors to verify consistent styling
    let result = run_wet_command(&["thoughts", "--color=always"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Should contain ANSI escape codes
    assert!(result.stdout.contains('\x1b'), "Should include ANSI codes");
    // Both occurrences of Sarah should be present
    let sarah_count = result.stdout.matches("Sarah").count();
    assert_eq!(sarah_count, 2, "Should have two occurrences of Sarah");
}

// T040: Edge case - 0 entities in output produces normal output
#[test]
fn test_notes_command_no_entities_normal_output() {
    let temp_db = setup_temp_db();

    // Add thoughts without any entity markup
    run_wet_command(&["add", "Plain thought without entities"], Some(&temp_db));
    run_wet_command(&["add", "Another plain thought"], Some(&temp_db));

    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Plain thought without entities"),
        "Should contain first thought"
    );
    assert!(
        result.stdout.contains("Another plain thought"),
        "Should contain second thought"
    );
    // Should have no ANSI codes since no entities to style
    assert!(
        !result.stdout.contains('\x1b'),
        "Should not have ANSI codes for thoughts without entities"
    );
}

// T041: Edge case - entity with special characters renders correctly
#[test]
fn test_notes_command_entity_with_special_characters() {
    let temp_db = setup_temp_db();

    // Add thoughts with entities containing special characters
    run_wet_command(&["add", "Task [bug-123_fix]"], Some(&temp_db));
    run_wet_command(&["add", "Review [PR #456]"], Some(&temp_db));
    run_wet_command(&["add", "Meeting about [project-alpha_v2.0]"], Some(&temp_db));

    let result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    // Verify special characters are preserved in entity names
    assert!(
        result.stdout.contains("bug-123_fix"),
        "Should contain entity with hyphens and underscores"
    );
    assert!(result.stdout.contains("PR #456"), "Should contain entity with hash");
    assert!(
        result.stdout.contains("project-alpha_v2.0"),
        "Should contain entity with dots"
    );
    // Brackets should be stripped
    assert!(
        !result.stdout.contains("[bug-123_fix]"),
        "Should not contain bracketed entity"
    );
}
