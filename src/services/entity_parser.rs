/// Entity parser service - extracts entity references from note text
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Regex pattern for entity syntax: [entity-name]
    /// Matches content within square brackets, excluding nested brackets
    static ref ENTITY_PATTERN: Regex = Regex::new(r"\[([^\[\]]+)\]").unwrap();
}

/// Extract entity names from text
///
/// # Examples
///
/// ```
/// use wetware::services::entity_parser::extract_entities;
///
/// let entities = extract_entities("Meeting with [Sarah] about [project-alpha]");
/// assert_eq!(entities, vec!["Sarah", "project-alpha"]);
/// ```
pub fn extract_entities(text: &str) -> Vec<String> {
    ENTITY_PATTERN
        .captures_iter(text)
        .map(|cap| cap[1].trim().to_string())
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
}
