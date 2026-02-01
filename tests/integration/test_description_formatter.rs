/// Integration tests for description formatter
use wetware::services::description_formatter;

// T041: Integration test for preview generation (first paragraph extraction)
#[test]
fn test_extract_first_paragraph() {
    let text = "First paragraph here.\n\nSecond paragraph.\n\nThird paragraph.";
    let first = description_formatter::extract_first_paragraph(text);
    assert_eq!(first, "First paragraph here.");
}

#[test]
fn test_extract_first_paragraph_single_paragraph() {
    let text = "Only one paragraph.";
    let first = description_formatter::extract_first_paragraph(text);
    assert_eq!(first, "Only one paragraph.");
}

#[test]
fn test_extract_first_paragraph_with_single_newlines() {
    let text = "First line\nSecond line\n\nSecond paragraph";
    let first = description_formatter::extract_first_paragraph(text);
    assert_eq!(first, "First line\nSecond line");
}

#[test]
fn test_extract_first_paragraph_empty() {
    let text = "";
    let first = description_formatter::extract_first_paragraph(text);
    assert_eq!(first, "");
}

// T042: Integration test for ellipsization with word boundaries
#[test]
fn test_ellipsize_word_boundary() {
    let text = "This is a long text that needs to be truncated";
    let ellipsized = description_formatter::ellipsize_at_word_boundary(text, 20);

    // Should truncate at word boundary and add ellipsis
    // Note: Using chars().count() because '…' is a multi-byte UTF-8 character
    assert!(
        ellipsized.chars().count() <= 20,
        "Should not exceed limit (character count)"
    );
    assert!(ellipsized.ends_with("…"), "Should end with ellipsis");
    assert!(ellipsized.contains("This is"), "Should start with beginning");

    // Should not break words - verify last word before ellipsis is complete
    let without_ellipsis = ellipsized.trim_end_matches('…');
    let words: Vec<&str> = without_ellipsis.split_whitespace().collect();
    let last_word = words.last().unwrap();

    // Last word should be a complete word from the original text
    assert!(
        text.contains(&format!("{} ", last_word)) || text.contains(&format!(" {}", last_word)),
        "Last word '{}' should be complete, not broken",
        last_word
    );
}

#[test]
fn test_ellipsize_no_truncation_needed() {
    let text = "Short text";
    let ellipsized = description_formatter::ellipsize_at_word_boundary(text, 50);
    assert_eq!(ellipsized, "Short text", "Should not add ellipsis when text fits");
}

#[test]
fn test_ellipsize_exact_length() {
    let text = "Exact";
    let ellipsized = description_formatter::ellipsize_at_word_boundary(text, 5);
    assert_eq!(ellipsized, "Exact", "Should not truncate when exactly at limit");
}

#[test]
fn test_ellipsize_no_spaces() {
    let text = "verylongtextwithoutanyspaces";
    let ellipsized = description_formatter::ellipsize_at_word_boundary(text, 10);

    // Should hard truncate at limit-1 and add ellipsis
    // Note: Using chars().count() because '…' is a multi-byte UTF-8 character
    assert_eq!(
        ellipsized.chars().count(),
        10,
        "Should truncate to limit (character count)"
    );
    assert!(ellipsized.ends_with("…"), "Should end with ellipsis");
}

// T043: Integration test for entity reference markup stripping
#[test]
fn test_strip_entity_markup_brackets() {
    let text = "See [programming] and [rust] for more.";
    let stripped = description_formatter::strip_entity_markup(text);
    assert_eq!(stripped, "See programming and rust for more.");
}

#[test]
fn test_strip_entity_markup_aliased() {
    let text = "Used by [Mozilla](mozilla) and [AWS](amazon-web-services).";
    let stripped = description_formatter::strip_entity_markup(text);
    assert_eq!(stripped, "Used by Mozilla and AWS.");
}

#[test]
fn test_strip_entity_markup_mixed() {
    let text = "[rust] and [the cloud](aws) are great.";
    let stripped = description_formatter::strip_entity_markup(text);
    assert_eq!(stripped, "rust and the cloud are great.");
}

#[test]
fn test_strip_entity_markup_no_entities() {
    let text = "Plain text without entities.";
    let stripped = description_formatter::strip_entity_markup(text);
    assert_eq!(stripped, "Plain text without entities.");
}

// T044: Integration test for newline collapsing
#[test]
fn test_collapse_newlines_single() {
    let text = "Line one\nLine two\nLine three";
    let collapsed = description_formatter::collapse_newlines(text);
    assert_eq!(collapsed, "Line one Line two Line three");
}

#[test]
fn test_collapse_newlines_multiple() {
    let text = "Line one\n\nLine two";
    let collapsed = description_formatter::collapse_newlines(text);
    assert_eq!(collapsed, "Line one Line two");
}

#[test]
fn test_collapse_newlines_with_whitespace() {
    let text = "Line one  \n  Line two";
    let collapsed = description_formatter::collapse_newlines(text);

    // Should collapse newlines and normalize spaces
    assert!(!collapsed.contains("\n"), "Should not contain newlines");
    assert_eq!(collapsed, "Line one Line two");
}

#[test]
fn test_collapse_newlines_no_newlines() {
    let text = "Single line";
    let collapsed = description_formatter::collapse_newlines(text);
    assert_eq!(collapsed, "Single line");
}

// T041: Integration test for full preview generation
#[test]
fn test_generate_preview_full_pipeline() {
    let description = "First paragraph with [entity] references.\n\nSecond paragraph ignored.";
    let entity_name = "rust";
    let terminal_width = 80;

    let preview = description_formatter::generate_preview(description, entity_name, terminal_width);

    // Should extract first paragraph
    assert!(!preview.contains("Second paragraph"));

    // Should strip entity markup
    assert!(!preview.contains("[entity]"));
    assert!(preview.contains("entity"));

    // Should collapse newlines (if any within first paragraph)
    assert!(!preview.contains("\n"));
}

#[test]
fn test_generate_preview_long_description() {
    let description = "This is a very long description that will definitely exceed the terminal width and should be truncated at a word boundary with an ellipsis at the end.";
    let entity_name = "test";
    let terminal_width = 60;

    let preview = description_formatter::generate_preview(description, entity_name, terminal_width);

    // Should be truncated
    assert!(preview.len() < description.len());

    // Should end with ellipsis
    assert!(preview.ends_with("…"));

    // Should not break words
    assert!(!preview.contains("defini"), "Should not break 'definitely'");
}

#[test]
fn test_generate_preview_with_entity_name_calculation() {
    let description = "Short description.";
    let entity_name = "verylongentityname";
    let terminal_width = 40;

    // Available width = 40 - len("verylongentityname") - len(" - ") = 40 - 18 - 3 = 19
    let preview = description_formatter::generate_preview(description, entity_name, terminal_width);

    // Preview should fit within available space
    let total_line_length = entity_name.len() + 3 + preview.len(); // name + " - " + preview
    assert!(
        total_line_length <= terminal_width,
        "Total line should fit terminal width"
    );
}

#[test]
fn test_generate_preview_narrow_terminal_minimum() {
    let description = "Description text here.";
    let entity_name = "test";
    let terminal_width = 30; // Very narrow

    let preview = description_formatter::generate_preview(description, entity_name, terminal_width);

    // Should still generate some preview (or empty if too narrow)
    // The implementation should handle this gracefully
    assert!(preview.len() <= terminal_width);
}
