/// Contract tests for `wet entities` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entities_command_success() {
    let temp_db = setup_temp_db();

    // Add notes with various entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [John]"], Some(&temp_db));
    run_wet_command(&["add", "Email [Alice] and [Bob]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(result.stdout.contains("Sarah"), "Should list Sarah");
    assert!(result.stdout.contains("John"), "Should list John");
    assert!(result.stdout.contains("Alice"), "Should list Alice");
    assert!(result.stdout.contains("Bob"), "Should list Bob");
}

#[test]
fn test_entities_command_empty_database() {
    let temp_db = setup_temp_db();

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(
        result.status, 0,
        "Command should succeed even with no entities"
    );
    assert!(
        result.stdout.contains("No entities") || result.stdout.is_empty(),
        "Should indicate no entities found. Got: {}",
        result.stdout
    );
}

#[test]
fn test_entities_command_alphabetical_order() {
    let temp_db = setup_temp_db();

    // Add entities in non-alphabetical order
    run_wet_command(&["add", "Note with [Zebra]"], Some(&temp_db));
    run_wet_command(&["add", "Note with [Apple]"], Some(&temp_db));
    run_wet_command(&["add", "Note with [Middle]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Verify alphabetical order by checking positions
    let stdout = result.stdout;
    let apple_pos = stdout.find("Apple").expect("Should contain Apple");
    let middle_pos = stdout.find("Middle").expect("Should contain Middle");
    let zebra_pos = stdout.find("Zebra").expect("Should contain Zebra");

    assert!(apple_pos < middle_pos, "Apple should appear before Middle");
    assert!(middle_pos < zebra_pos, "Middle should appear before Zebra");
}

#[test]
fn test_entities_command_canonical_capitalization() {
    let temp_db = setup_temp_db();

    // Add same entity with different capitalizations
    run_wet_command(&["add", "First note with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Second note with [SARAH]"], Some(&temp_db));
    run_wet_command(&["add", "Third note with [sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Should show Sarah only once with first-occurrence capitalization
    let sarah_count = result.stdout.matches("Sarah").count();
    assert_eq!(sarah_count, 1, "Should show Sarah exactly once");

    // Should NOT show other capitalizations
    assert!(!result.stdout.contains("SARAH"), "Should not show SARAH");
    assert!(
        result.stdout.matches("sarah").count() == 0,
        "Should not show lowercase sarah as separate entity"
    );
}

#[test]
fn test_entities_command_unique_entities() {
    let temp_db = setup_temp_db();

    // Add multiple notes referencing the same entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Email [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Sarah should appear exactly once despite multiple references
    let sarah_count = result.stdout.matches("Sarah").count();
    assert_eq!(sarah_count, 1, "Should show Sarah exactly once");
}
