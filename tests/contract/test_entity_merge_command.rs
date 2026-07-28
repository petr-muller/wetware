/// Contract tests for `wet entity merge` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entity_merge_happy_path() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));
    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "alice", "--into", "Bob"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed. stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("Merged entity 'Alice' into 'Bob'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let entities_result = run_wet_command(&["entities"], Some(&temp_db));
    assert!(
        !entities_result.stdout.to_lowercase().contains("alice"),
        "Merged-away entity should be gone from the entities list. Got: {}",
        entities_result.stdout
    );
    assert!(
        entities_result.stdout.contains("Bob"),
        "Surviving entity should still be listed. Got: {}",
        entities_result.stdout
    );
}

#[test]
fn test_entity_merge_moves_thoughts_to_target() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));
    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));
    run_wet_command(&["entity", "merge", "alice", "--into", "Bob"], Some(&temp_db));

    let show_result = run_wet_command(&["entity", "show", "Bob"], Some(&temp_db));

    assert_eq!(show_result.status, 0, "Command should succeed");
    assert!(
        show_result.stdout.contains("Lunch with"),
        "The merged entity's thought should now belong to the target. Got: {}",
        show_result.stdout
    );
    assert!(
        show_result.stdout.contains("Standup with"),
        "The target's own thought should still be listed. Got: {}",
        show_result.stdout
    );
}

#[test]
fn test_entity_merge_preserves_original_wording() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));
    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));
    run_wet_command(&["entity", "merge", "alice", "--into", "Bob"], Some(&temp_db));

    let thoughts_result = run_wet_command(&["thoughts"], Some(&temp_db));

    assert!(
        thoughts_result.stdout.contains("Lunch with Alice"),
        "Thought should still read as originally written. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_merge_resolves_source_through_alias() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));
    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "alice", "--alias", "Ali"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "Ali", "--into", "Bob"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed. stderr: {}", result.stderr);
    assert!(
        result.stdout.contains("Merged entity 'Alice' into 'Bob'"),
        "Should report canonical names, not the alias. Got: {}",
        result.stdout
    );
}

#[test]
fn test_entity_merge_nonexistent_source() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "nobody", "--into", "Bob"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("not found"),
        "Should show not-found error. stderr: {}",
        result.stderr
    );
}

#[test]
fn test_entity_merge_nonexistent_target() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "alice", "--into", "nobody"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("not found"),
        "Should show not-found error. stderr: {}",
        result.stderr
    );
}

#[test]
fn test_entity_merge_into_target_with_parentheses_is_rejected() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Coffee with [Alice (HR)]"], Some(&temp_db));
    run_wet_command(&["add", "Standup with [Bob]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "Bob", "--into", "Alice (HR)"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("'(' or ')'"),
        "Should explain the reserved characters. stderr: {}",
        result.stderr
    );

    let thoughts_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Standup with Bob"),
        "Thought text should be untouched by the rejected merge. Got: {}",
        thoughts_result.stdout
    );
    assert!(
        !thoughts_result.stdout.contains("Bob(Alice"),
        "Must not write markup the parser cannot read back. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_merge_into_itself_is_rejected() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Lunch with [Alice]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "merge", "alice", "--into", "Alice"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("into itself"),
        "Should show self-merge error. stderr: {}",
        result.stderr
    );

    let entities_result = run_wet_command(&["entities"], Some(&temp_db));
    assert!(
        entities_result.stdout.contains("Alice"),
        "Entity should survive a rejected self-merge. Got: {}",
        entities_result.stdout
    );
}
