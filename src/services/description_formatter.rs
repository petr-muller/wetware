// Description formatter for entity description previews
// Handles terminal width detection, paragraph extraction, ellipsization

/// Module for formatting entity descriptions for display in terminal
///
/// This module provides functionality to:
/// - Detect terminal width
/// - Extract first paragraph from multi-paragraph text
/// - Strip entity reference markup
/// - Collapse newlines to single-line text
/// - Ellipsize text to fit terminal width with word-boundary awareness
use crate::services::entity_parser::ENTITY_PATTERN;
use terminal_size::{Width, terminal_size};

/// Get current terminal width, defaulting to 80 if detection fails
///
/// # Returns
/// Terminal width in characters, or 80 if detection fails
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::get_terminal_width;
///
/// let width = get_terminal_width();
/// assert!(width > 0);
/// ```
pub fn get_terminal_width() -> usize {
    terminal_size().map(|(Width(w), _)| w as usize).unwrap_or(80)
}

/// Extract first paragraph from text (split on double newline)
///
/// # Arguments
/// * `text` - Multi-paragraph text
///
/// # Returns
/// First paragraph only (text before first "\n\n")
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::extract_first_paragraph;
///
/// let text = "First paragraph.\n\nSecond paragraph.";
/// assert_eq!(extract_first_paragraph(text), "First paragraph.");
/// ```
pub fn extract_first_paragraph(text: &str) -> &str {
    text.split("\n\n").next().unwrap_or("")
}

/// Strip entity reference markup, leaving display text only
///
/// Converts:
/// - `[entity]` → `entity`
/// - `[alias](entity)` → `alias`
///
/// # Arguments
/// * `text` - Text with entity references
///
/// # Returns
/// Text with entity markup removed, showing display text only
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::strip_entity_markup;
///
/// let text = "See [rust] and [the guide](rust-guide)";
/// assert_eq!(strip_entity_markup(text), "See rust and the guide");
/// ```
pub fn strip_entity_markup(text: &str) -> String {
    ENTITY_PATTERN
        .replace_all(text, |caps: &regex::Captures| {
            // Group 1: alias (or entity for traditional syntax)
            // Group 2: entity reference (if aliased syntax)
            let display = caps[1].trim();
            display.to_string()
        })
        .to_string()
}

/// Collapse all newlines to spaces and normalize whitespace
///
/// # Arguments
/// * `text` - Multi-line text
///
/// # Returns
/// Single-line text with normalized spaces
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::collapse_newlines;
///
/// let text = "Line one\nLine two\nLine three";
/// assert_eq!(collapse_newlines(text), "Line one Line two Line three");
/// ```
pub fn collapse_newlines(text: &str) -> String {
    // Replace newlines with spaces
    let collapsed = text.replace('\n', " ");

    // Normalize multiple spaces to single space
    let mut result = String::new();
    let mut prev_space = false;

    for ch in collapsed.chars() {
        if ch.is_whitespace() {
            if !prev_space {
                result.push(' ');
                prev_space = true;
            }
        } else {
            result.push(ch);
            prev_space = false;
        }
    }

    result.trim().to_string()
}

/// Ellipsize text at word boundary to fit within length limit
///
/// # Arguments
/// * `text` - Text to truncate
/// * `max_length` - Maximum length (including ellipsis if added)
///
/// # Returns
/// Truncated text with "…" appended if truncation occurred
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::ellipsize_at_word_boundary;
///
/// let text = "This is a long text";
/// let result = ellipsize_at_word_boundary(text, 10);
/// assert!(result.ends_with("…"));
/// assert!(result.len() <= 10);
/// ```
pub fn ellipsize_at_word_boundary(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }

    // Find last space before max_length - 1 (reserve 1 char for ellipsis)
    let truncate_at = max_length.saturating_sub(1);

    if let Some(last_space_pos) = text[..truncate_at].rfind(' ') {
        // Truncate at word boundary
        format!("{}…", &text[..last_space_pos])
    } else {
        // No space found - hard truncate
        format!("{}…", &text[..truncate_at])
    }
}

/// Calculate available width for preview text
///
/// Available width = terminal_width - entity_name_length - separator_length
///
/// # Arguments
/// * `entity_name` - Name of the entity
/// * `terminal_width` - Total terminal width
///
/// # Returns
/// Available width for preview, or 0 if insufficient space
fn calculate_available_width(entity_name: &str, terminal_width: usize) -> usize {
    const SEPARATOR: &str = " - "; // 3 characters
    let used = entity_name.len() + SEPARATOR.len();

    terminal_width.saturating_sub(used)
}

/// Generate preview text for entity description
///
/// Full pipeline:
/// 1. Extract first paragraph
/// 2. Strip entity reference markup
/// 3. Collapse newlines to single line
/// 4. Ellipsize to fit terminal width
///
/// # Arguments
/// * `description` - Full entity description
/// * `entity_name` - Name of the entity (for width calculation)
/// * `terminal_width` - Terminal width in characters
///
/// # Returns
/// Preview text ready for display (single line, ellipsized if needed)
///
/// # Examples
/// ```
/// use wetware::services::description_formatter::generate_preview;
///
/// let desc = "First paragraph.\n\nSecond paragraph.";
/// let preview = generate_preview(desc, "rust", 80);
/// assert!(!preview.contains("\n"));
/// assert!(!preview.contains("Second"));
/// ```
pub fn generate_preview(description: &str, entity_name: &str, terminal_width: usize) -> String {
    // Step 1: Extract first paragraph
    let first_para = extract_first_paragraph(description);

    // Step 2: Strip entity markup
    let stripped = strip_entity_markup(first_para);

    // Step 3: Collapse newlines
    let collapsed = collapse_newlines(&stripped);

    // Step 4: Calculate available width and ellipsize
    let available = calculate_available_width(entity_name, terminal_width);

    // Minimum preview length check (don't show preview if too narrow)
    const MIN_PREVIEW_WIDTH: usize = 20;
    if available < MIN_PREVIEW_WIDTH {
        return String::new();
    }

    ellipsize_at_word_boundary(&collapsed, available)
}

#[cfg(test)]
mod tests {
    use super::*;

    // T045: Unit tests for terminal width detection
    #[test]
    fn test_get_terminal_width_returns_positive() {
        let width = get_terminal_width();
        assert!(width > 0, "Terminal width should be positive");
    }

    #[test]
    fn test_get_terminal_width_has_reasonable_default() {
        // Can't easily test fallback, but verify function doesn't panic
        let width = get_terminal_width();
        assert!(width >= 80, "Terminal width should be at least 80 (default)");
    }

    // T046: Unit tests for preview length calculation
    #[test]
    fn test_calculate_available_width_normal() {
        let available = calculate_available_width("rust", 80);
        // 80 - 4 (rust) - 3 (" - ") = 73
        assert_eq!(available, 73);
    }

    #[test]
    fn test_calculate_available_width_long_entity_name() {
        let available = calculate_available_width("verylongentityname", 40);
        // 40 - 18 - 3 = 19
        assert_eq!(available, 19);
    }

    #[test]
    fn test_calculate_available_width_insufficient_space() {
        let available = calculate_available_width("longname", 10);
        // 10 - 8 - 3 = -1 → saturates to 0
        assert_eq!(available, 0);
    }

    #[test]
    fn test_extract_first_paragraph_multiple() {
        let text = "First.\n\nSecond.";
        assert_eq!(extract_first_paragraph(text), "First.");
    }

    #[test]
    fn test_extract_first_paragraph_single() {
        let text = "Only one.";
        assert_eq!(extract_first_paragraph(text), "Only one.");
    }

    #[test]
    fn test_strip_entity_markup_traditional() {
        let text = "[entity] text";
        assert_eq!(strip_entity_markup(text), "entity text");
    }

    #[test]
    fn test_strip_entity_markup_aliased() {
        let text = "[alias](target)";
        assert_eq!(strip_entity_markup(text), "alias");
    }

    #[test]
    fn test_collapse_newlines_basic() {
        let text = "a\nb\nc";
        assert_eq!(collapse_newlines(text), "a b c");
    }

    #[test]
    fn test_collapse_newlines_multiple_spaces() {
        let text = "a  \n  b";
        assert_eq!(collapse_newlines(text), "a b");
    }

    #[test]
    fn test_ellipsize_word_boundary_truncates() {
        let text = "word1 word2 word3";
        let result = ellipsize_at_word_boundary(text, 10);
        assert!(result.len() <= 10);
        assert!(result.ends_with("…"));
    }

    #[test]
    fn test_ellipsize_word_boundary_no_truncation() {
        let text = "short";
        let result = ellipsize_at_word_boundary(text, 10);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_generate_preview_full_pipeline() {
        let desc = "First para with [entity].\n\nSecond para.";
        let preview = generate_preview(desc, "test", 80);

        assert!(!preview.contains("\n"));
        assert!(!preview.contains("Second"));
        assert!(!preview.contains("[entity]"));
        assert!(preview.contains("entity"));
    }

    #[test]
    fn test_generate_preview_too_narrow() {
        let desc = "Description";
        let preview = generate_preview(desc, "verylongentityname", 20);
        // Available: 20 - 18 - 3 = -1 → 0, which is < MIN_PREVIEW_WIDTH
        // Should return empty string
        assert_eq!(preview, "");
    }
}
