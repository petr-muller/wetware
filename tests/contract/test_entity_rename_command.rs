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
fn test_entity_rename_rejects_collision_with_other_entitys_alias() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah] and [John]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "john", "--alias", "boss"], Some(&temp_db));

    let result = run_wet_command(&["entity", "rename", "sarah", "boss"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("already registered as an alias")
            || result.stdout.contains("already registered as an alias"),
        "Should show alias-collision error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_rename_to_own_alias_succeeds() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "sarah", "--alias", "boss"], Some(&temp_db));

    let result = run_wet_command(&["entity", "rename", "sarah", "boss"], Some(&temp_db));
    assert_eq!(result.status, 0, "Renaming to the entity's own alias should succeed");
}

#[test]
fn test_entity_rename_by_alias_lookup() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    let result = run_wet_command(&["entity", "rename", "sar", "Sarah Smith"], Some(&temp_db));
    assert_eq!(
        result.status, 0,
        "Should be able to rename an entity by referencing one of its aliases"
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

#[test]
fn test_entity_rename_rejects_bracket_characters_in_new_name() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    for bad_name in ["Sarah]x", "[Sarah", "Sarah(x)", "Sarah)"] {
        let result = run_wet_command(&["entity", "rename", "sarah", bad_name], Some(&temp_db));

        assert_ne!(result.status, 0, "Command should fail for new name '{}'", bad_name);
        assert!(
            result.stderr.contains("cannot contain") || result.stdout.contains("cannot contain"),
            "Should show reserved-character error for '{}'. stderr: {}, stdout: {}",
            bad_name,
            result.stderr,
            result.stdout
        );
    }

    // Original content must be untouched by the rejected attempts.
    let thoughts_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Sarah") && !thoughts_result.stdout.contains("Sarah]x"),
        "Original content should be unchanged. Got: {}",
        thoughts_result.stdout
    );
}
