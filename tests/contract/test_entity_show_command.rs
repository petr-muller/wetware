/// Contract tests for `wet entity show` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

#[test]
fn test_entity_show_entity_not_found() {
    let temp_db = setup_temp_db();

    let result = run_wet_command(&["entity", "show", "nonexistent"], Some(&temp_db));

    assert_ne!(result.status, 0, "Command should fail for nonexistent entity");
    assert!(
        result.stderr.contains("not found"),
        "Should report entity not found. Got: {}",
        result.stderr
    );
}

#[test]
fn test_entity_show_without_description() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["entity", "show", "sarah"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(result.stdout.contains("Sarah"), "Should show entity header");
    assert!(
        result.stdout.contains("Latest thoughts:"),
        "Should show thoughts section header"
    );
    assert!(
        result.stdout.contains("Meeting with Sarah"),
        "Should list the linked thought. Got: {}",
        result.stdout
    );
}

#[test]
fn test_entity_show_with_full_multi_paragraph_description() {
    let temp_db = setup_temp_db();

    run_wet_command(&["add", "Meeting with [rust]"], Some(&temp_db));
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "First paragraph about rust.\n\nSecond paragraph about rust.",
        ],
        Some(&temp_db),
    );

    let result = run_wet_command(&["entity", "show", "rust"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("First paragraph about rust."),
        "Should show first paragraph. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("Second paragraph about rust."),
        "Should show second paragraph in full (not truncated like the `entities` preview). Got: {}",
        result.stdout
    );
}

#[test]
fn test_entity_show_no_thoughts() {
    let temp_db = setup_temp_db();

    // Referencing `orphan-entity` in another entity's description auto-creates it
    // (per entity_edit's auto-creation of referenced entities) without linking it
    // to any thought, giving us an entity with zero linked thoughts to test against.
    run_wet_command(&["add", "Meeting with [rust]"], Some(&temp_db));
    run_wet_command(
        &["entity", "edit", "rust", "--description", "See also [orphan-entity]"],
        Some(&temp_db),
    );

    let result = run_wet_command(&["entity", "show", "orphan-entity"], Some(&temp_db));
    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains("orphan-entity"),
        "Should show header. Got: {}",
        result.stdout
    );
    assert!(
        result.stdout.contains("No thoughts found for entity"),
        "Should show no-thoughts message. Got: {}",
        result.stdout
    );
}

#[test]
fn test_entity_show_latest_five_thoughts_newest_first() {
    let temp_db = setup_temp_db();

    for i in 1..=7 {
        run_wet_command(
            &[
                "add",
                &format!("Thought number {i} about [rust]"),
                "--date",
                &format!("2026-01-{:02}", i),
            ],
            Some(&temp_db),
        );
    }

    let result = run_wet_command(&["entity", "show", "rust"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");

    // Only the 5 most recent (numbers 3-7) should be shown
    for i in 3..=7 {
        assert!(
            result.stdout.contains(&format!("Thought number {i}")),
            "Should contain thought number {i}. Got: {}",
            result.stdout
        );
    }
    for i in 1..=2 {
        assert!(
            !result.stdout.contains(&format!("Thought number {i} ")),
            "Should not contain older thought number {i}. Got: {}",
            result.stdout
        );
    }

    // Newest first
    let newest_pos = result.stdout.find("Thought number 7").unwrap();
    let oldest_shown_pos = result.stdout.find("Thought number 3").unwrap();
    assert!(newest_pos < oldest_shown_pos, "Thoughts should be newest first");
}

#[test]
fn test_entity_show_color_always_styles_description_and_thoughts_consistently() {
    let temp_db = setup_temp_db();

    run_wet_command(
        &["add", "First thought referencing [rust] the language"],
        Some(&temp_db),
    );
    run_wet_command(
        &[
            "entity",
            "edit",
            "rust",
            "--description",
            "See also [ML](machine-learning) for related work.",
        ],
        Some(&temp_db),
    );

    let result = run_wet_command(&["--color=always", "entity", "show", "rust"], Some(&temp_db));

    assert_eq!(result.status, 0, "Command should succeed");
    assert!(
        result.stdout.contains('\x1b'),
        "Should contain ANSI escape codes with --color=always. Got: {:?}",
        result.stdout
    );
    // Alias display text shown, not the target entity name
    assert!(result.stdout.contains("ML"), "Should show alias display text");
    assert!(
        !result.stdout.contains("[ML](machine-learning)"),
        "Should not contain raw markup. Got: {}",
        result.stdout
    );
}
