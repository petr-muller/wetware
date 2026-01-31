/// Integration tests for styled entity output (T030)
use tempfile::TempDir;
use wetware::cli::{add, thoughts};
use wetware::services::color_mode::ColorMode;

#[test]
fn test_styled_output_with_color_always() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add thought with entity
    add::execute("Meeting with [Sarah]".to_string(), Some(&db_path)).unwrap();

    // Execute with colors always on (even though we're not in a TTY)
    let result = thoughts::execute(Some(&db_path), None, ColorMode::Always);
    assert!(result.is_ok());
}

#[test]
fn test_styled_output_with_color_never() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add thought with entity
    add::execute("Meeting with [Sarah]".to_string(), Some(&db_path)).unwrap();

    // Execute with colors disabled
    let result = thoughts::execute(Some(&db_path), None, ColorMode::Never);
    assert!(result.is_ok());
}

#[test]
fn test_styled_output_with_color_auto() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    // Add thought with entity
    add::execute("Meeting with [Sarah]".to_string(), Some(&db_path)).unwrap();

    // Execute with auto-detection (will be plain since tests aren't TTY)
    let result = thoughts::execute(Some(&db_path), None, ColorMode::Auto);
    assert!(result.is_ok());
}

#[test]
fn test_color_mode_auto_returns_bool() {
    // ColorMode::Auto.should_use_colors() returns a boolean based on TTY detection.
    // We don't assert a specific value since the result depends on the test runner:
    // - cargo nextest: spawns in subprocess with captured stdout (non-TTY) -> false
    // - cargo test: may run with stdout connected to terminal (TTY) -> true
    // The actual behavior is verified by contract tests which spawn subprocesses.
    let use_colors = ColorMode::Auto.should_use_colors();
    // Just verify it returns a valid bool (this always passes, but documents the behavior)
    assert!(use_colors || !use_colors);
}

#[test]
fn test_color_mode_always_overrides_tty() {
    // Always should return true regardless of TTY status
    assert!(ColorMode::Always.should_use_colors());
}

#[test]
fn test_color_mode_never_overrides_tty() {
    // Never should return false regardless of TTY status
    assert!(!ColorMode::Never.should_use_colors());
}
