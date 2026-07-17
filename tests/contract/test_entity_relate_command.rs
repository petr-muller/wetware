/// Contract tests for `wet entity relate` / `wet entity unrelate` commands
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entity_relate_happy_path_expands_thoughts_filter() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Ordering from [Amazon]"], Some(&temp_db));
    run_wet_command(&["add", "Setting up [AWS]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "relate", "aws", "--parent", "amazon"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("'aws' is now a child of 'amazon'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let thoughts_result = run_wet_command(&["thoughts", "--on", "amazon"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Ordering from") && thoughts_result.stdout.contains("Setting up"),
        "Filtering by 'amazon' should include the AWS-tagged thought too. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_relate_missing_entity_errors() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Ordering from [Amazon]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "relate", "aws", "--parent", "amazon"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("Entity 'aws' not found") || result.stdout.contains("Entity 'aws' not found"),
        "Should show entity not found error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_relate_self_relation_rejected() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "About [Amazon]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "relate", "amazon", "--parent", "amazon"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("cannot be its own parent") || result.stdout.contains("cannot be its own parent"),
        "Should show self-relation error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_relate_cycle_rejected() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "About [Amazon] and [AWS]"], Some(&temp_db));
    run_wet_command(&["entity", "relate", "aws", "--parent", "amazon"], Some(&temp_db));

    let result = run_wet_command(&["entity", "relate", "amazon", "--parent", "aws"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail");
    assert!(
        result.stderr.contains("cycle") || result.stdout.contains("cycle"),
        "Should show cycle error. stderr: {}, stdout: {}",
        result.stderr,
        result.stdout
    );
}

#[test]
fn test_entity_unrelate_removes_relation() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Ordering from [Amazon]"], Some(&temp_db));
    run_wet_command(&["add", "Setting up [AWS]"], Some(&temp_db));
    run_wet_command(&["entity", "relate", "aws", "--parent", "amazon"], Some(&temp_db));

    let result = run_wet_command(&["entity", "unrelate", "aws", "--parent", "amazon"], Some(&temp_db));
    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Removed 'aws' as a child of 'amazon'"),
        "Should show success message. Got: {}",
        result.stdout
    );

    let thoughts_result = run_wet_command(&["thoughts", "--on", "amazon"], Some(&temp_db));
    assert!(
        thoughts_result.stdout.contains("Ordering from") && !thoughts_result.stdout.contains("Setting up"),
        "Filtering by 'amazon' should no longer include the AWS-tagged thought. Got: {}",
        thoughts_result.stdout
    );
}

#[test]
fn test_entity_show_lists_parents_and_children() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "About [Amazon]"], Some(&temp_db));
    run_wet_command(&["add", "About [AWS]"], Some(&temp_db));
    run_wet_command(&["entity", "relate", "aws", "--parent", "amazon"], Some(&temp_db));

    let amazon_result = run_wet_command(&["entity", "show", "amazon"], Some(&temp_db));
    assert!(
        amazon_result.stdout.contains("Children: AWS"),
        "Should list AWS as a child of Amazon. Got: {}",
        amazon_result.stdout
    );

    let aws_result = run_wet_command(&["entity", "show", "aws"], Some(&temp_db));
    assert!(
        aws_result.stdout.contains("Parents: Amazon"),
        "Should list Amazon as a parent of AWS. Got: {}",
        aws_result.stdout
    );
}
