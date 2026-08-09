/// Contract tests for `wet add` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_add_command_success() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "This is a test thought"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Thought added"),
        "Should confirm thought was added. Got: {}",
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

    assert_ne!(result.status, 0, "Command should fail with oversized content");
    assert!(
        result.stderr.contains("exceeds") || result.stderr.contains("too long") || result.stderr.contains("maximum"),
        "Should report size limit error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_whitespace_only() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "   "], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail with whitespace-only content");
    assert!(
        result.stderr.contains("empty") || result.stderr.contains("cannot be empty"),
        "Should report empty content error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_with_date() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "Backdated thought", "--date", "2024-03-15"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed with valid date");
    assert!(
        result.stdout.contains("Thought added"),
        "Should confirm thought was added. Got: {}",
        result.stdout
    );
}

#[test]
fn test_add_command_with_invalid_date() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "Bad date thought", "--date", "not-a-date"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail with invalid date");
    assert!(
        result.stderr.contains("Invalid date"),
        "Should report invalid date error. Got: {}",
        result.stderr
    );
    assert!(
        result.stderr.contains("YYYY-MM-DD"),
        "Should list the accepted forms. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_with_relative_date() {
    let temp_db = setup_temp_db();

    for date in ["today", "yesterday", "-3d", "-2w", "mon"] {
        let result = run_wet_command(&["add", "Relative dated thought", "--date", date], Some(&temp_db));

        assert_eq!(
            result.status, 0,
            "--date {} should be accepted. Got: {}",
            date, result.stderr
        );
        assert!(
            result.stdout.contains("Thought added"),
            "Should confirm thought was added for --date {}. Got: {}",
            date,
            result.stdout
        );
    }
}

#[test]
fn test_add_command_without_content_requires_a_terminal() {
    let temp_db = setup_temp_db();
    // Test processes get piped stdio, so the composer cannot start here. It must
    // say so plainly rather than panicking or garbling the terminal.
    let result = run_wet_command(&["add"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail without a terminal");
    assert!(
        result.stderr.contains("terminal"),
        "Should explain that a terminal is needed. Got: {}",
        result.stderr
    );
}

#[test]
fn test_add_command_interactive_flag_conflicts_with_content() {
    let temp_db = setup_temp_db();
    let result = run_wet_command(&["add", "-i", "some content"], Some(&temp_db));

    assert_ne!(result.status, 0, "-i and CONTENT are mutually exclusive");
}
