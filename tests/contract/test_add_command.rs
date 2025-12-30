/// Contract tests for `wet add` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_add_command_success() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "This is a test note"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Note added"),
        "Should confirm note was added. Got: {}",
        result.stdout
    );
    assert!(
        result.stderr.is_empty(),
        "Should have no errors. Got stderr: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_empty_content() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", ""], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail with empty content");
    assert!(
        result.stderr.contains("empty") || result.stderr.contains("cannot be empty"),
        "Should report empty content error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_oversized_content() {
    let temp_db = setup_temp_db();
    // Create content larger than 10,000 characters
    let large_content = "a".repeat(10_001);
    let result = run_wet_command(&["add", &large_content], Some(&temp_db));

    assert_ne!(
        result.status, 0,
        "Command should fail with oversized content"
    );
    assert!(
        result.stderr.contains("exceeds")
            || result.stderr.contains("too long")
            || result.stderr.contains("maximum"),
        "Should report size limit error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_whitespace_only() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "   "], Some(&temp_db));

    assert_ne!(
        result.status, 0,
        "Command should fail with whitespace-only content"
    );
    assert!(
        result.stderr.contains("empty") || result.stderr.contains("cannot be empty"),
        "Should report empty content error. Got: {}",
        result.stderr
    );
}
