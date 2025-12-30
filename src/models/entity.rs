/// Entity domain model
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    pub id: Option<i64>,
    pub name: String,           // Lowercase for case-insensitive lookups
    pub canonical_name: String, // Original capitalization for display
}

impl Entity {
    /// Create a new entity with case normalization
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name: name.to_lowercase(), // Normalize for case-insensitive matching
            canonical_name: name,      // Preserve original capitalization
        }
    }

    /// Get the display name (canonical capitalization)
    pub fn display_name(&self) -> &str {
        &self.canonical_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entity() {
        let entity = Entity::new("Sarah".to_string());
        assert_eq!(entity.name, "sarah");
        assert_eq!(entity.canonical_name, "Sarah");
        assert!(entity.id.is_none());
    }

    #[test]
    fn test_entity_case_normalization() {
        let entity1 = Entity::new("SARAH".to_string());
        let entity2 = Entity::new("sarah".to_string());
        let entity3 = Entity::new("Sarah".to_string());

        // All should have same normalized name
        assert_eq!(entity1.name, "sarah");
        assert_eq!(entity2.name, "sarah");
        assert_eq!(entity3.name, "sarah");

        // But preserve original capitalization
        assert_eq!(entity1.canonical_name, "SARAH");
        assert_eq!(entity2.canonical_name, "sarah");
        assert_eq!(entity3.canonical_name, "Sarah");
    }

    #[test]
    fn test_entity_display_name() {
        let entity = Entity::new("Project-Alpha".to_string());
        assert_eq!(entity.display_name(), "Project-Alpha");
    }

    #[test]
    fn test_entity_with_spaces() {
        let entity = Entity::new("multi word entity".to_string());
        assert_eq!(entity.name, "multi word entity");
        assert_eq!(entity.canonical_name, "multi word entity");
    }

    #[test]
    fn test_entity_with_special_chars() {
        let entity = Entity::new("task-123_test".to_string());
        assert_eq!(entity.name, "task-123_test");
        assert_eq!(entity.canonical_name, "task-123_test");
    }
}
