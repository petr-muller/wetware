//! Entity styling service for colored and bold entity output
//!
//! This module provides the [`EntityStyler`] struct for rendering thought content
//! with styled entities. Entities are displayed with bold text and consistent colors,
//! while bracket markup is removed for clean output.

use owo_colors::{AnsiColors, OwoColorize};
use std::collections::HashMap;

use super::entity_parser::ENTITY_PATTERN;

/// Available colors for entity styling.
///
/// This palette uses 12 distinct ANSI colors that work well on both light and dark
/// terminal backgrounds. White and black are excluded to avoid contrast issues.
const ENTITY_COLORS: [AnsiColors; 12] = [
    AnsiColors::Cyan,
    AnsiColors::Green,
    AnsiColors::Yellow,
    AnsiColors::Blue,
    AnsiColors::Magenta,
    AnsiColors::Red,
    AnsiColors::BrightCyan,
    AnsiColors::BrightGreen,
    AnsiColors::BrightYellow,
    AnsiColors::BrightBlue,
    AnsiColors::BrightMagenta,
    AnsiColors::BrightRed,
];

/// Manages consistent color assignment for entities within a single execution.
///
/// The styler ensures that:
/// - Each unique entity gets a consistent color throughout the output
/// - Entity names are matched case-insensitively
/// - Colors cycle when there are more entities than available colors
/// - Bracket markup is removed from entity display
///
/// # Examples
///
/// ```
/// use wetware::services::entity_styler::EntityStyler;
///
/// let mut styler = EntityStyler::new(false); // plain mode
/// let output = styler.render_content("Meeting with [Sarah] about [project]");
/// assert_eq!(output, "Meeting with Sarah about project");
/// ```
pub struct EntityStyler {
    /// Maps lowercase entity names to their assigned colors
    color_map: HashMap<String, AnsiColors>,
    /// Index of the next color to assign
    next_color: usize,
    /// Whether to apply styling (bold + colors)
    use_colors: bool,
}

impl EntityStyler {
    /// Create a new EntityStyler.
    ///
    /// # Arguments
    ///
    /// * `use_colors` - Whether to apply ANSI styling (bold + colors) to entities.
    ///   When `false`, entities are rendered as plain text without brackets.
    ///
    /// # Examples
    ///
    /// ```
    /// use wetware::services::entity_styler::EntityStyler;
    ///
    /// // For terminal output
    /// let styled = EntityStyler::new(true);
    ///
    /// // For piped/redirected output
    /// let plain = EntityStyler::new(false);
    /// ```
    pub fn new(use_colors: bool) -> Self {
        Self {
            color_map: HashMap::new(),
            next_color: 0,
            use_colors,
        }
    }

    /// Get or assign a color for the given entity.
    ///
    /// Colors are assigned on first encounter and cached for consistency.
    /// Entity names are matched case-insensitively.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity name (without brackets)
    ///
    /// # Returns
    ///
    /// The ANSI color assigned to this entity
    fn get_color(&mut self, entity: &str) -> AnsiColors {
        let key = entity.to_lowercase();
        if let Some(&color) = self.color_map.get(&key) {
            return color;
        }
        let color = ENTITY_COLORS[self.next_color % ENTITY_COLORS.len()];
        self.color_map.insert(key, color);
        self.next_color += 1;
        color
    }

    /// Render thought content with styled entities.
    ///
    /// This method:
    /// - Finds all entity references (text in square brackets)
    /// - Removes the bracket markup
    /// - Applies bold and color styling when `use_colors` is true
    /// - Preserves plain text segments unchanged
    ///
    /// # Arguments
    ///
    /// * `content` - The thought content with entity markup (e.g., `"Meeting with [Sarah]"`)
    ///
    /// # Returns
    ///
    /// The rendered string with entities styled or plain (without brackets)
    ///
    /// # Examples
    ///
    /// ```
    /// use wetware::services::entity_styler::EntityStyler;
    ///
    /// let mut styler = EntityStyler::new(false);
    /// let output = styler.render_content("[Sarah] called about [project-alpha]");
    /// assert_eq!(output, "Sarah called about project-alpha");
    /// ```
    pub fn render_content(&mut self, content: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for cap in ENTITY_PATTERN.captures_iter(content) {
            let full_match = cap.get(0).unwrap();
            let entity_name = cap[1].trim();

            // Add text before this entity
            result.push_str(&content[last_end..full_match.start()]);

            // Add styled or plain entity name
            if self.use_colors {
                let color = self.get_color(entity_name);
                let styled = entity_name.bold().color(color).to_string();
                result.push_str(&styled);
            } else {
                result.push_str(entity_name);
            }

            last_end = full_match.end();
        }

        // Add remaining text after last entity
        result.push_str(&content[last_end..]);

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T010: Same entity gets same color (case-insensitive)
    #[test]
    fn test_same_entity_same_color_case_insensitive() {
        let mut styler = EntityStyler::new(true);
        let color1 = styler.get_color("Sarah");
        let color2 = styler.get_color("sarah");
        let color3 = styler.get_color("SARAH");
        assert_eq!(color1, color2);
        assert_eq!(color2, color3);
    }

    // T011: Different entities get different colors
    #[test]
    fn test_different_entities_different_colors() {
        let mut styler = EntityStyler::new(true);
        let color1 = styler.get_color("Sarah");
        let color2 = styler.get_color("John");
        let color3 = styler.get_color("project-alpha");
        assert_ne!(color1, color2);
        assert_ne!(color2, color3);
        assert_ne!(color1, color3);
    }

    // T012: Colors cycle when entities exceed palette size (12+)
    #[test]
    fn test_colors_cycle_after_palette_exhausted() {
        let mut styler = EntityStyler::new(true);
        let colors: Vec<AnsiColors> = (0..15).map(|i| styler.get_color(&format!("entity{}", i))).collect();

        // First 12 entities should have unique colors
        for i in 0..12 {
            for j in (i + 1)..12 {
                assert_ne!(colors[i], colors[j], "Colors {} and {} should differ", i, j);
            }
        }

        // 13th entity (index 12) should wrap to first color
        assert_eq!(colors[0], colors[12]);
        // 14th entity (index 13) should wrap to second color
        assert_eq!(colors[1], colors[13]);
        // 15th entity (index 14) should wrap to third color
        assert_eq!(colors[2], colors[14]);
    }

    // T013: render_content strips bracket markup from entities
    #[test]
    fn test_render_content_strips_brackets() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Meeting with [Sarah] about [project-alpha]");
        assert_eq!(output, "Meeting with Sarah about project-alpha");
        assert!(!output.contains('['));
        assert!(!output.contains(']'));
    }

    // T014: Plain text segments preserved unchanged
    #[test]
    fn test_plain_text_preserved() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Just plain text without entities");
        assert_eq!(output, "Just plain text without entities");
    }

    #[test]
    fn test_mixed_content_preserved() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Before [entity] middle [another] after");
        assert_eq!(output, "Before entity middle another after");
    }

    // T015: Styled output includes bold and color ANSI codes
    #[test]
    fn test_styled_output_includes_ansi_codes() {
        let mut styler = EntityStyler::new(true);
        let output = styler.render_content("Hello [Sarah]");

        // Should contain ANSI escape codes
        assert!(output.contains("\x1b["), "Should contain ANSI escape sequence");
        // Should contain the entity name
        assert!(output.contains("Sarah"));
        // Should NOT contain the original bracket markup (but ANSI codes use [ so we can't check for absence of all brackets)
        // Instead verify the original markup pattern is gone
        assert!(
            !output.contains("[Sarah]"),
            "Should not contain original bracket markup"
        );
    }

    #[test]
    fn test_styled_same_entity_consistent_color() {
        let mut styler = EntityStyler::new(true);
        let output = styler.render_content("[Sarah] and [sarah] again");

        // Count occurrences of "Sarah" - both should be styled the same
        let sarah_count = output.matches("Sarah").count() + output.matches("sarah").count();
        assert_eq!(sarah_count, 2);

        // The output should have ANSI codes
        assert!(output.contains("\x1b["));
    }

    #[test]
    fn test_no_entities_returns_original() {
        let mut styler = EntityStyler::new(true);
        let input = "Just plain text";
        let output = styler.render_content(input);
        assert_eq!(output, input);
    }

    #[test]
    fn test_empty_brackets_ignored() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Empty [] brackets");
        // Empty brackets are ignored by the pattern
        assert_eq!(output, "Empty [] brackets");
    }

    #[test]
    fn test_entity_with_spaces() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[multi word entity]");
        assert_eq!(output, "multi word entity");
    }

    #[test]
    fn test_entity_with_special_characters() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[project-alpha_v2]");
        assert_eq!(output, "project-alpha_v2");
    }

    #[test]
    fn test_whitespace_trimmed_from_entities() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[  Sarah  ]");
        assert_eq!(output, "Sarah");
    }
}
