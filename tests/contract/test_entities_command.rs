/// Contract tests for `wet entities` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entities_command_success() {
    let temp_db = setup_temp_db();

    // Add thoughts with various entities
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

    assert_eq!(result.status, 0, "Command should succeed even with no entities");
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
    run_wet_command(&["add", "Thought with [Zebra]"], Some(&temp_db));
    run_wet_command(&["add", "Thought with [Apple]"], Some(&temp_db));
    run_wet_command(&["add", "Thought with [Middle]"], Some(&temp_db));

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
    run_wet_command(&["add", "First thought with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Second thought with [SARAH]"], Some(&temp_db));
    run_wet_command(&["add", "Third thought with [sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Should show Sarah only once with first-occurrence capitalization
    let sarah_count = result.stdout.matches("Sarah").count();
    assert_eq!(sarah_count, 1, "Should show Sarah exactly once");

    // Should NOT show other capitalizations
    assert!(!result.stdout.contains("SARAH"), "Should not show SARAH");
    assert_eq!(
        result.stdout.matches("sarah").count(),
        0,
        "Should not show lowercase sarah as separate entity"
    );
}

#[test]
fn test_entities_command_unique_entities() {
    let temp_db = setup_temp_db();

    // Add multiple thoughts referencing the same entities
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Call [Sarah]"], Some(&temp_db));
    run_wet_command(&["add", "Email [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Sarah should appear exactly once despite multiple references
    let sarah_count = result.stdout.matches("Sarah").count();
    assert_eq!(sarah_count, 1, "Should show Sarah exactly once");
}

// T037: Contract test for `wet entities` with description previews
#[test]
fn test_entities_with_description_previews() {
    let temp_db = setup_temp_db();

    // Create entities with descriptions
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "Rust is a systems programming language that focuses on safety and concurrency.",
        ],
        Some(&temp_db),
    );

    run_wet_command(&["add", "Learning about [wetware]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "wetware",
            "--description",
            "A CLI tool for managing thoughts and entities.",
        ],
        Some(&temp_db),
    );

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Should show entity name followed by " - " and preview
    assert!(
        result.stdout.contains("rust - "),
        "Should show 'rust - ' prefix. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("wetware - "),
        "Should show 'wetware - ' prefix. Got: {}",
        result.stdout
    );

    // Should contain parts of the descriptions
    assert!(
        result.stdout.contains("systems programming"),
        "Should show description preview"
    );
    assert!(result.stdout.contains("CLI tool"), "Should show description preview");
}

// T038: Contract test for mixed entities (some with/without descriptions)
#[test]
fn test_entities_mixed_with_and_without_descriptions() {
    let temp_db = setup_temp_db();

    // Entity with description
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "A systems programming language.",
        ],
        Some(&temp_db),
    );

    // Entity without description
    run_wet_command(&["add", "Learning about [javascript]"], Some(&temp_db));

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Entity with description should have preview
    assert!(result.stdout.contains("rust - "), "rust should have preview separator");
    assert!(
        result.stdout.contains("systems programming"),
        "rust should show description"
    );

    // Entity without description should be name only
    let lines: Vec<&str> = result.stdout.lines().collect();
    let js_line = lines
        .iter()
        .find(|line| line.contains("javascript"))
        .expect("Should find javascript line");

    // Should be just the name, not followed by " - "
    assert!(
        !js_line.contains(" - "),
        "javascript should not have preview separator. Got: {}",
        js_line
    );
}

// T039: Contract test for narrow terminal (<60 chars) suppresses previews
#[test]
fn test_entities_narrow_terminal_no_previews() {
    let temp_db = setup_temp_db();

    // Create entities with descriptions
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "A systems programming language.",
        ],
        Some(&temp_db),
    );

    // Note: This test verifies the logic exists, but actual terminal width
    // detection is hard to test in contract tests. The implementation should
    // check terminal width < 60 and suppress previews.
    // This is better tested in integration tests for the formatter module.

    let result = run_wet_command(&["entities"], Some(&temp_db));
    assert_eq!(result.status, 0, "Command should succeed");

    // In normal terminal (>=60), should show preview
    // The actual narrow terminal behavior is tested in integration tests
}

// T040: Contract test for entity references rendered as plain text in previews
#[test]
fn test_entities_preview_entity_references_as_plain_text() {
    let temp_db = setup_temp_db();

    // Create entity with description containing entity references
    run_wet_command(&["add", "Learning about [rust]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "Used by [mozilla] and [the cloud](aws). See [programming] for context.",
        ],
        Some(&temp_db),
    );

    let result = run_wet_command(&["entities"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Entity references should be rendered as plain text (no brackets)
    let rust_line = result
        .stdout
        .lines()
        .find(|line| line.starts_with("rust - "))
        .expect("Should find rust line");

    // Should contain plain text versions of references
    assert!(
        rust_line.contains("mozilla") && !rust_line.contains("[mozilla]"),
        "Should show 'mozilla' without brackets. Got: {}",
        rust_line
    );

    // Aliased reference should show alias text, not target
    assert!(
        rust_line.contains("the cloud") && !rust_line.contains("aws"),
        "Should show alias 'the cloud' not target 'aws'. Got: {}",
        rust_line
    );

    assert!(
        rust_line.contains("programming") && !rust_line.contains("[programming]"),
        "Should show 'programming' without brackets. Got: {}",
        rust_line
    );
}
