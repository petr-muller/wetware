/// Contract tests for `wet entity alias` / `wet entity unalias` commands
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entity_alias_happy_path_filters_thoughts() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Alias 'sar' added for entity 'Sarah'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let thoughts_result = run_wet_command(&["thoughts", "--on", "sar"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Meeting with"),
        "Filtering by alias 'sar' should return the same thoughts as 'sarah'. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_alias_missing_entity_errors() {
    let temp_db = setup_temp_db();

    let result = run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("Entity 'sarah' not found") || result.stdout.contains("Entity 'sarah' not found"),
        "Should show entity not found error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_alias_same_alias_two_entities_both_succeed() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "About [Sarah] and [John]"], Some(&temp_db));

    let result1 = run_wet_command(&["entity", "alias", "sarah", "--alias", "boss"], Some(&temp_db));
    let result2 = run_wet_command(&["entity", "alias", "john", "--alias", "boss"], Some(&temp_db));

    assert_eq!(result1.status, 0, "First alias registration should succeed");
    assert_eq!(
        result2.status, 0,
        "Second alias registration (same alias, different entity) should succeed"
    );

    let filter_result = run_wet_command(&["thoughts", "--on", "boss"], Some(&temp_db));
    assert_ne!(filter_result.status, 0, "Filtering by an ambiguous alias should fail");
    assert!(
        filter_result.stderr.contains("multiple entities") || filter_result.stdout.contains("multiple entities"),
        "Should show ambiguous-alias error. stderr: {}, stdout: {}",
        filter_result.stderr,
        filter_result.stdout
    );
}

#[test]
fn test_entity_unalias_removes_alias() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    let result = run_wet_command(&["entity", "unalias", "sarah", "--alias", "sar"], Some(&temp_db));
    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Alias 'sar' removed from entity 'Sarah'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let thoughts_result = run_wet_command(&["thoughts", "--on", "sar"], Some(&temp_db));
    assert!(
        !thoughts_result.stdout.contains("Meeting with"),
        "Filtering by the removed alias should no longer match the thought. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_alias_bracket_mention_resolves_to_aliased_entity() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    let add_result = run_wet_command(&["add", "Talked to [sar] again"], Some(&temp_db));
    assert_eq!(add_result.status, 0, "Add should succeed");

    let entities_result = run_wet_command(&["entities"], Some(&temp_db));
    let sar_count = entities_result
        .stdout
        .lines()
        .filter(|line| line.to_lowercase().contains("sar"))
        .count();
    assert_eq!(
        sar_count, 1,
        "No new literal 'sar' entity should be created; only 'Sarah' should mention 'sar'. Got: {}",
        entities_result.stdout
    );
}

#[test]
fn test_entity_show_lists_aliases() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["entity", "alias", "sarah", "--alias", "sar"], Some(&temp_db));

    let result = run_wet_command(&["entity", "show", "sarah"], Some(&temp_db));
    assert!(
        result.stdout.contains("Aliases: sar"),
        "Should list 'sar' as an alias. Got: {}",
        result.stdout
    );
}
