/// Entity parser service - extracts entity references from note text
use regex::{Captures, Regex};
use std::sync::LazyLock;

/// Regex pattern for entity syntax: [entity] or [alias](entity)
///
/// Matches both traditional and aliased entity references:
/// - Traditional: `[entity]` - entity name in square brackets
/// - Aliased: `[alias](entity)` - display text in brackets, entity in parentheses
///
/// Capture groups:
/// - Group 1: Display text (alias or entity name for traditional syntax)
/// - Group 2: Target entity (optional, only present for aliased syntax)
///
/// This pattern maintains full backward compatibility with existing `[entity]` syntax
/// while enabling natural language aliases like `[robot](robotics)`.
pub static ENTITY_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[([^\[\]]+)](?:\(([^()]+)\))?").unwrap());

/// Extract entity names from text
///
/// Supports both traditional `[entity]` and aliased `[alias](entity)` syntax.
/// For aliased syntax, returns the target entity (not the alias).
///
/// # Examples
///
/// ```
/// use wetware::services::entity_parser::extract_entities;
///
/// // Traditional syntax
/// let entities = extract_entities("Meeting with [Sarah] about [project-alpha]");
/// assert_eq!(entities, vec!["Sarah", "project-alpha"]);
///
/// // Aliased syntax (returns target entity)
/// let entities = extract_entities("Started [ML](machine-learning) course");
/// assert_eq!(entities, vec!["machine-learning"]);
///
/// // Mixed syntax
/// let entities = extract_entities("[robotics] and [robot](robotics)");
/// assert_eq!(entities, vec!["robotics", "robotics"]);
/// ```
pub fn extract_entities(text: &str) -> Vec<String> {
    ENTITY_PATTERN
        .captures_iter(text)
        .filter_map(|cap| {
            // Capture group 1: alias (or entity for traditional syntax)
            // Capture group 2: entity reference (if aliased syntax)
            let alias = cap[1].trim();
            let entity = cap.get(2).map(|m| m.as_str().trim());

            // Return target entity, not alias
            match entity {
                Some(ent) if !ent.is_empty() => Some(ent.to_string()), // Aliased syntax
                None if !alias.is_empty() => Some(alias.to_string()),  // Traditional syntax
                _ => None,                                             // Invalid (empty alias or empty entity)
            }
        })
        .collect()
}

/// Extract unique entities from text (case-insensitive deduplication)
///
/// Returns entities in the order of first occurrence with original capitalization preserved.
pub fn extract_unique_entities(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut unique = Vec::new();

    for entity in extract_entities(text) {
        let lowercase = entity.to_lowercase();
        if !seen.contains(&lowercase) {
            seen.insert(lowercase);
            unique.push(entity);
        }
    }

    unique
}

/// Rewrite literal references to `old_name` (case-insensitive) into `new_name`.
///
/// Used when renaming an entity, to keep stored thought/description text in sync
/// with the entity's new name.
///
/// - Bare `[OldName]` -> `[NewName]` (no group 2, display text matches `old_name`)
/// - Aliased `[Alias](OldName)` -> `[Alias](NewName)` (group 2 matches `old_name`;
///   the alias/display text in group 1 is left untouched)
/// - `[Sarah](other-target)` where group 1 happens to equal `old_name` but group 2
///   is a different target: left untouched (coincidental alias text, not a reference
///   to this entity)
/// - Everything else: left untouched
///
/// # Examples
///
/// ```
/// use wetware::services::entity_parser::rewrite_entity_references;
///
/// let text = rewrite_entity_references("Meeting with [Sarah]", "sarah", "Sarah Smith");
/// assert_eq!(text, "Meeting with [Sarah Smith]");
///
/// let text = rewrite_entity_references("Called [Sarah](sarah) again", "sarah", "Sarah Smith");
/// assert_eq!(text, "Called [Sarah](Sarah Smith) again");
/// ```
pub fn rewrite_entity_references(text: &str, old_name: &str, new_name: &str) -> String {
    let old_lower = old_name.trim().to_lowercase();

    ENTITY_PATTERN
        .replace_all(text, |caps: &Captures| {
            let alias = caps[1].trim();
            let target = caps.get(2).map(|m| m.as_str().trim());

            match target {
                Some(t) if !t.is_empty() => {
                    if t.to_lowercase() == old_lower {
                        format!("[{}]({})", alias, new_name)
                    } else {
                        caps[0].to_string()
                    }
                }
                None if alias.to_lowercase() == old_lower => format!("[{}]", new_name),
                _ => caps[0].to_string(),
            }
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_entity() {
        let entities = extract_entities("Meeting with [Sarah]");
        assert_eq!(entities, vec!["Sarah"]);
    }

    #[test]
    fn test_extract_multiple_entities() {
        let entities = extract_entities("Discussion about [project-alpha] with [Sarah] and [John]");
        assert_eq!(entities, vec!["project-alpha", "Sarah", "John"]);
    }

    #[test]
    fn test_extract_no_entities() {
        let entities = extract_entities("Regular note without entities");
        assert!(entities.is_empty());
    }

    #[test]
    fn test_extract_empty_brackets_ignored() {
        let entities = extract_entities("Empty [] brackets ignored");
        assert!(entities.is_empty());
    }

    #[test]
    fn test_extract_unclosed_bracket_ignored() {
        let entities = extract_entities("Unclosed [bracket ignored");
        assert!(entities.is_empty());
    }

    #[test]
    fn test_extract_nested_brackets() {
        // With nested brackets, inner content is extracted (outer brackets match first)
        let entities = extract_entities("[[inner]]");
        assert_eq!(entities, vec!["inner"]);
    }

    #[test]
    fn test_extract_entity_with_spaces() {
        let entities = extract_entities("Reference [multi word entity]");
        assert_eq!(entities, vec!["multi word entity"]);
    }

    #[test]
    fn test_extract_entity_with_hyphens() {
        let entities = extract_entities("Project [project-alpha-v2]");
        assert_eq!(entities, vec!["project-alpha-v2"]);
    }

    #[test]
    fn test_extract_entity_with_underscores() {
        let entities = extract_entities("Task [bug_123]");
        assert_eq!(entities, vec!["bug_123"]);
    }

    #[test]
    fn test_extract_entity_with_numbers() {
        let entities = extract_entities("Issue [issue-42]");
        assert_eq!(entities, vec!["issue-42"]);
    }

    #[test]
    fn test_extract_duplicate_entities() {
        let entities = extract_entities("[Sarah] and [sarah] and [SARAH]");
        assert_eq!(entities, vec!["Sarah", "sarah", "SARAH"]);
    }

    #[test]
    fn test_extract_unique_entities_case_insensitive() {
        let unique = extract_unique_entities("[Sarah] and [sarah] and [SARAH]");
        assert_eq!(unique, vec!["Sarah"]); // First occurrence preserved
    }

    #[test]
    fn test_extract_unique_entities_multiple() {
        let unique = extract_unique_entities("[Sarah] and [John] and [sarah]");
        assert_eq!(unique, vec!["Sarah", "John"]);
    }

    #[test]
    fn test_extract_unique_entities_order_preserved() {
        let unique = extract_unique_entities("[Z] [A] [M] [a]");
        assert_eq!(unique, vec!["Z", "A", "M"]); // First occurrence order and caps preserved
    }

    #[test]
    fn test_extract_whitespace_trimmed() {
        let entities = extract_entities("[  Sarah  ]");
        assert_eq!(entities, vec!["Sarah"]);
    }

    // ========== User Story 1: Aliased Entity Reference Tests ==========

    #[test]
    fn test_extract_aliased_entity() {
        let entities = extract_entities("Started [ML](machine-learning) course");
        assert_eq!(entities, vec!["machine-learning"]); // Returns target entity, not alias
    }

    #[test]
    fn test_extract_mixed_syntax() {
        let entities = extract_entities("[robotics] and [robot](robotics)");
        assert_eq!(entities, vec!["robotics", "robotics"]); // Both extract same target entity
    }

    #[test]
    fn test_extract_aliased_with_whitespace() {
        let entities = extract_entities("[ ML ]( machine-learning )");
        assert_eq!(entities, vec!["machine-learning"]); // Whitespace trimmed from both
    }

    #[test]
    fn test_extract_empty_alias_rejected() {
        let entities = extract_entities("[](entity)");
        assert!(entities.is_empty()); // Empty alias → no match
    }

    #[test]
    fn test_extract_empty_entity_fallback() {
        let entities = extract_entities("[alias]()");
        assert_eq!(entities, vec!["alias"]); // Empty entity → treated as traditional
    }

    #[test]
    fn test_extract_malformed_unclosed_paren() {
        let entities = extract_entities("[alias](unclosed");
        assert_eq!(entities, vec!["alias"]); // Unclosed paren → treated as traditional
    }

    #[test]
    fn test_extract_unique_deduplicates_by_target() {
        let unique = extract_unique_entities("[robot](robotics) and [robotics]");
        assert_eq!(unique, vec!["robotics"]); // Both reference same entity → deduplicated
    }

    // ========== User Story 3: Backward Compatibility Tests ==========

    #[test]
    fn test_traditional_syntax_unchanged() {
        // Verify traditional [entity] syntax works exactly as before
        let entities = extract_entities("[Sarah] met [John] about [project-alpha]");
        assert_eq!(entities, vec!["Sarah", "John", "project-alpha"]);

        // All original test cases should still work
        assert_eq!(extract_entities("Meeting with [Sarah]"), vec!["Sarah"]);
        assert_eq!(extract_entities("[entity1] [entity2]"), vec!["entity1", "entity2"]);
        assert!(extract_entities("No entities here").is_empty());
        assert!(extract_entities("Empty [] ignored").is_empty());
    }

    // ========== Entity Rename: rewrite_entity_references Tests ==========

    #[test]
    fn test_rewrite_bare_single_occurrence() {
        let text = rewrite_entity_references("Meeting with [Sarah]", "sarah", "Sarah Smith");
        assert_eq!(text, "Meeting with [Sarah Smith]");
    }

    #[test]
    fn test_rewrite_bare_multiple_occurrences() {
        let text = rewrite_entity_references("[Sarah] met [Sarah] again", "sarah", "Sarah Smith");
        assert_eq!(text, "[Sarah Smith] met [Sarah Smith] again");
    }

    #[test]
    fn test_rewrite_aliased_target_preserves_alias_text() {
        let text = rewrite_entity_references("Called [Sarah](sarah) again", "sarah", "Sarah Smith");
        assert_eq!(text, "Called [Sarah](Sarah Smith) again");
    }

    #[test]
    fn test_rewrite_aliased_target_different_alias_text() {
        let text = rewrite_entity_references("Called [my friend](sarah) again", "sarah", "Sarah Smith");
        assert_eq!(text, "Called [my friend](Sarah Smith) again");
    }

    #[test]
    fn test_rewrite_untouched_coincidental_alias_display_text() {
        // "Sarah" is used here as alias display text for a *different* target entity.
        let text = rewrite_entity_references("[Sarah](project-alpha)", "sarah", "Sarah Smith");
        assert_eq!(text, "[Sarah](project-alpha)");
    }

    #[test]
    fn test_rewrite_case_insensitive_match() {
        let text = rewrite_entity_references("[SARAH] and [sarah]", "Sarah", "Sarah Smith");
        assert_eq!(text, "[Sarah Smith] and [Sarah Smith]");
    }

    #[test]
    fn test_rewrite_mixed_bare_and_aliased() {
        let text = rewrite_entity_references("[Sarah] called [S](sarah)", "sarah", "Sarah Smith");
        assert_eq!(text, "[Sarah Smith] called [S](Sarah Smith)");
    }

    #[test]
    fn test_rewrite_no_match_passthrough() {
        let text = rewrite_entity_references("Meeting with [John] about [project]", "sarah", "Sarah Smith");
        assert_eq!(text, "Meeting with [John] about [project]");
    }

    #[test]
    fn test_rewrite_unrelated_text_unchanged() {
        let text = rewrite_entity_references("No entities here at all", "sarah", "Sarah Smith");
        assert_eq!(text, "No entities here at all");
    }
}
