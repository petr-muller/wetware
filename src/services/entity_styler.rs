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
    /// Supports both traditional `[entity]` and aliased `[alias](entity)` syntax.
    /// For aliased syntax, displays the alias but colors by the target entity.
    ///
    /// This method:
    /// - Finds all entity references (traditional and aliased)
    /// - Removes the markup (brackets and parentheses)
    /// - Displays the alias for aliased syntax, entity name for traditional
    /// - Colors based on target entity for consistent coloring
    /// - Applies bold and color styling when `use_colors` is true
    /// - Preserves plain text segments unchanged
    ///
    /// # Arguments
    ///
    /// * `content` - The thought content with entity markup
    ///
    /// # Returns
    ///
    /// The rendered string with entities styled or plain (without markup)
    ///
    /// # Examples
    ///
    /// ```
    /// use wetware::services::entity_styler::EntityStyler;
    ///
    /// let mut styler = EntityStyler::new(false);
    ///
    /// // Traditional syntax
    /// let output = styler.render_content("[Sarah] called about [project-alpha]");
    /// assert_eq!(output, "Sarah called about project-alpha");
    ///
    /// // Aliased syntax (displays alias)
    /// let output = styler.render_content("[ML](machine-learning) course");
    /// assert_eq!(output, "ML course");
    /// ```
    pub fn render_content(&mut self, content: &str) -> String {
        let mut result = String::new();
        let mut last_end = 0;

        for cap in ENTITY_PATTERN.captures_iter(content) {
            let full_match = cap.get(0).unwrap();

            // Group 1: display text (alias or entity name for traditional)
            // Group 2: target entity (optional, only for aliased syntax)
            let display_text = cap[1].trim();
            let target_entity = cap.get(2).map(|m| m.as_str().trim()).unwrap_or(display_text); // If no group 2, use group 1

            // Add text before this entity
            result.push_str(&content[last_end..full_match.start()]);

            // Add styled or plain entity
            if self.use_colors {
                // Color by TARGET entity, display ALIAS
                let color = self.get_color(target_entity);
                let styled = display_text.bold().color(color).to_string();
                result.push_str(&styled);
            } else {
                result.push_str(display_text);
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

    // ========== User Story 2: Aliased Entity Rendering Tests ==========

    #[test]
    fn test_render_aliased_entity() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Started [ML](machine-learning) course");
        assert_eq!(output, "Started ML course"); // Shows alias, not entity
    }

    #[test]
    fn test_render_mixed_syntax_same_color() {
        let mut styler = EntityStyler::new(true);
        // First render traditional syntax to assign color
        styler.render_content("[robotics]");

        // Now render aliased reference to same entity
        let output = styler.render_content("[robot](robotics)");

        // Both should use same color mapping (one entity in color_map)
        assert_eq!(styler.color_map.len(), 1); // Only one entity color assigned
        assert!(output.contains("robot")); // Displays alias
    }

    #[test]
    fn test_render_aliased_with_whitespace() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[ ML ]( machine-learning )");
        assert_eq!(output, "ML"); // Whitespace trimmed, alias displayed
    }

    #[test]
    fn test_render_aliased_displays_alias_not_entity() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[robot](robotics)");
        assert_eq!(output, "robot"); // Shows "robot" not "robotics"
        assert!(!output.contains("robotics"));
    }

    #[test]
    fn test_render_color_by_target_entity() {
        let mut styler = EntityStyler::new(true);
        let output = styler.render_content("[robot](robotics) and [machine](robotics)");

        // Both aliases reference same entity, should have same color
        assert_eq!(styler.color_map.len(), 1); // Only one entity color
        assert!(output.contains("robot"));
        assert!(output.contains("machine"));
    }

    #[test]
    fn test_render_multiple_aliases_same_entity_same_color() {
        let mut styler = EntityStyler::new(true);
        styler.render_content("[robotics]"); // Traditional
        let color1 = *styler.color_map.get("robotics").unwrap();

        styler.render_content("[robot](robotics)"); // Aliased to same entity
        let color2 = *styler.color_map.get("robotics").unwrap();

        assert_eq!(color1, color2); // Same entity = same color
    }

    // ========== User Story 3: Backward Compatibility Tests ==========

    #[test]
    fn test_traditional_rendering_unchanged() {
        // Verify traditional [entity] rendering works exactly as before
        let mut styler = EntityStyler::new(false);

        assert_eq!(styler.render_content("[Sarah]"), "Sarah");
        assert_eq!(styler.render_content("[Sarah] and [John]"), "Sarah and John");
        assert_eq!(
            styler.render_content("Meeting with [Sarah] about [project-alpha]"),
            "Meeting with Sarah about project-alpha"
        );

        // Brackets should be stripped
        let output = styler.render_content("[entity]");
        assert!(!output.contains('['));
        assert!(!output.contains(']'));
    }
}
