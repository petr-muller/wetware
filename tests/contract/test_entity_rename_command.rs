/// Contract tests for `wet entity rename` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entity_rename_happy_path() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah] about [project-alpha]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "rename", "sarah", "Sarah Smith"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Entity 'sarah' renamed to 'Sarah Smith'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let thoughts_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Sarah Smith"),
        "Thought text should reflect new name. Got: {}",
        thoughts_result.stdout
    );

    let entities_result = run_wet_command(&["entities"], Some(&temp_db));
    assert!(
        entities_result.stdout.contains("Sarah Smith"),
        "Entities list should show new name. Got: {}",
        entities_result.stdout
    );

    let entity_count = entities_result
        .stdout
        .lines()
        .filter(|line| line.to_lowercase().contains("sarah"))
        .count();
    assert_eq!(
        entity_count, 1,
        "Should be exactly one entity mentioning 'sarah' (no leftover duplicate). Got: {}",
        entities_result.stdout
    );
}

#[test]
fn test_entity_rename_collision_with_different_entity() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah] and [John]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "rename", "sarah", "John"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("already exists") || result.stdout.contains("already exists"),
        "Should show collision error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_rename_nonexistent_entity() {
    let temp_db = setup_temp_db();

    let result = run_wet_command(&["entity", "rename", "nonexistent", "New Name"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("Entity 'nonexistent' not found")
            || result.stdout.contains("Entity 'nonexistent' not found"),
        "Should show entity not found error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}
