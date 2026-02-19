/// Contract tests for `wet edit` command
use crate::test_helpers::{run_wet_command, setup_temp_db};

// ── US1: Edit thought text via direct CLI argument ──────────────────────────

#[test]
fn test_edit_updates_thought_content() {
    let temp_db = setup_temp_db();

    // Add a thought and capture its ID
    let add_result = run_wet_command(&["add", "Original content"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    // ID is printed in format "Thought added successfully (ID: N)"
    let id = extract_id_from_add_output(&add_result.stdout);

    // Edit the thought
    let edit_result = run_wet_command(&["edit", &id, "Updated content"], Some(&temp_db));
    assert_eq!(
        edit_result.status, 0,
        "Edit should succeed. stderr: {}",
        edit_result.stderr
    );
    assert!(
        edit_result.stdout.contains("updated"),
        "Should confirm update. Got: {}",
        edit_result.stdout
    );

    // Verify the update by listing thoughts
    let list_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        list_result.stdout.contains("Updated content"),
        "Updated content should appear in listing. Got: {}",
        list_result.stdout
    );
    assert!(
        !list_result.stdout.contains("Original content"),
        "Old content should not appear. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_edit_removes_old_entity_associations() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Met with [Alice] about project"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    // Verify Alice is linked
    let list_alice = run_wet_command(&["thoughts", "--on", "Alice"], Some(&temp_db));
    assert!(
        list_alice.stdout.contains("Alice"),
        "Alice should be linked before edit"
    );

    // Edit to remove Alice reference, add Bob
    let edit_result = run_wet_command(&["edit", &id, "Met with [Bob] instead"], Some(&temp_db));
    assert_eq!(edit_result.status, 0);

    // Alice should no longer be linked
    let list_alice_after = run_wet_command(&["thoughts", "--on", "Alice"], Some(&temp_db));
    assert!(
        !list_alice_after.stdout.contains("Bob"),
        "Alice filter should not return thought with only Bob. Got: {}",
        list_alice_after.stdout
    );

    // Bob should be linked
    let list_bob = run_wet_command(&["thoughts", "--on", "Bob"], Some(&temp_db));
    assert!(
        list_bob.stdout.contains("Bob"),
        "Bob should be linked after edit. Got: {}",
        list_bob.stdout
    );
}

#[test]
fn test_edit_adds_new_entity_when_not_existing() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Plain thought with no entities"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    // Edit to add a new entity reference
    let edit_result = run_wet_command(&["edit", &id, "Thought referencing [NewEntity]"], Some(&temp_db));
    assert_eq!(edit_result.status, 0);

    // NewEntity should now be linked
    let list_result = run_wet_command(&["thoughts", "--on", "NewEntity"], Some(&temp_db));
    assert!(
        list_result.stdout.contains("NewEntity"),
        "NewEntity should be linked after edit. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_edit_clears_all_entities_when_text_has_none() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Linked to [Alice] and [Bob]"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    // Edit to remove all entity refs
    let edit_result = run_wet_command(&["edit", &id, "Plain text now"], Some(&temp_db));
    assert_eq!(edit_result.status, 0);

    let list_alice = run_wet_command(&["thoughts", "--on", "Alice"], Some(&temp_db));
    assert!(
        list_alice.stdout.is_empty() || !list_alice.stdout.contains("Plain text"),
        "Thought should not appear under Alice after edit. Got: {}",
        list_alice.stdout
    );
}

// ── US1: Error cases ────────────────────────────────────────────────────────

#[test]
fn test_edit_nonexistent_id_returns_error() {
    let temp_db = setup_temp_db();

    let result = run_wet_command(&["edit", "9999", "some text"], Some(&temp_db));
    assert_ne!(result.status, 0, "Should fail for nonexistent ID");
    assert!(
        result.stderr.contains("9999") || result.stderr.contains("not found"),
        "Error should mention ID or not found. Got: {}",
        result.stderr
    );
}

#[test]
fn test_edit_empty_content_returns_error() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Original"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    let result = run_wet_command(&["edit", &id, ""], Some(&temp_db));
    assert_ne!(result.status, 0, "Should fail with empty content");
    assert!(
        result.stderr.contains("empty") || result.stderr.contains("cannot be empty"),
        "Should report empty content error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_edit_no_arguments_returns_error() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Original"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    // Provide only ID, no content/date/editor
    let result = run_wet_command(&["edit", &id], Some(&temp_db));
    assert_ne!(result.status, 0, "Should fail when no edit argument provided");
}

// ── US3: Edit thought date ───────────────────────────────────────────────────

#[test]
fn test_edit_date_updates_thought_date() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "A dated thought"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    let edit_result = run_wet_command(&["edit", &id, "--date", "2026-01-15"], Some(&temp_db));
    assert_eq!(
        edit_result.status, 0,
        "Date edit should succeed. stderr: {}",
        edit_result.stderr
    );

    // Verify date appears in listing (format: [id] 2026-01-15 ...)
    let list_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        list_result.stdout.contains("2026-01-15"),
        "Updated date should appear in listing. Got: {}",
        list_result.stdout
    );
}

#[test]
fn test_edit_date_preserves_content_and_entities() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Thought with [Alice]"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    let edit_result = run_wet_command(&["edit", &id, "--date", "2026-01-15"], Some(&temp_db));
    assert_eq!(edit_result.status, 0);

    // Content should be unchanged
    let list_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        list_result.stdout.contains("Alice"),
        "Entity reference should remain after date edit. Got: {}",
        list_result.stdout
    );

    // Entity association should be preserved
    let list_alice = run_wet_command(&["thoughts", "--on", "Alice"], Some(&temp_db));
    assert!(
        list_alice.stdout.contains("Alice"),
        "Alice should still be linked after date edit. Got: {}",
        list_alice.stdout
    );
}

#[test]
fn test_edit_invalid_date_format_returns_error() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "A thought"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    let result = run_wet_command(&["edit", &id, "--date", "not-a-date"], Some(&temp_db));
    assert_ne!(result.status, 0, "Should fail with invalid date format");
    assert!(
        result.stderr.contains("date") || result.stderr.contains("YYYY-MM-DD") || result.stderr.contains("Invalid"),
        "Should report date format error. Got: {}",
        result.stderr
    );
}

#[test]
fn test_edit_content_and_date_together() {
    let temp_db = setup_temp_db();

    let add_result = run_wet_command(&["add", "Original content"], Some(&temp_db));
    assert_eq!(add_result.status, 0);
    let id = extract_id_from_add_output(&add_result.stdout);

    let edit_result = run_wet_command(
        &["edit", &id, "Updated content with [Alice]", "--date", "2026-01-20"],
        Some(&temp_db),
    );
    assert_eq!(
        edit_result.status, 0,
        "Combined edit should succeed. stderr: {}",
        edit_result.stderr
    );

    let list_result = run_wet_command(&["thoughts"], Some(&temp_db));
    assert!(
        list_result.stdout.contains("Updated content"),
        "Content should be updated. Got: {}",
        list_result.stdout
    );
    assert!(
        list_result.stdout.contains("2026-01-20"),
        "Date should be updated. Got: {}",
        list_result.stdout
    );
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Extract thought ID from `wet add` success output like "Thought added successfully (ID: 3)"
fn extract_id_from_add_output(stdout: &str) -> String {
    stdout
        .lines()
        .find(|l| l.contains("ID:"))
        .and_then(|l| {
            let start = l.find("ID: ")? + 4;
            let rest = &l[start..];
            let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
            Some(rest[..end].to_string())
        })
        .expect("Could not extract ID from add output")
}
