/// Thought domain model
use crate::errors::ThoughtError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Thought {
    pub id: Option<i64>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Thought {
    /// Create a new thought with validation
    pub fn new(content: String) -> Result<Self, ThoughtError> {
        Self::validate_content(&content)?;
        Ok(Self {
            id: None,
            content,
            created_at: Utc::now(),
        })
    }

    /// Create a new thought with a specific date
    pub fn new_with_date(content: String, created_at: DateTime<Utc>) -> Result<Self, ThoughtError> {
        Self::validate_content(&content)?;
        Ok(Self {
            id: None,
            content,
            created_at,
        })
    }

    /// Validate thought content
    fn validate_content(content: &str) -> Result<(), ThoughtError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(ThoughtError::EmptyContent);
        }
        if content.len() > 10_000 {
            return Err(ThoughtError::ContentTooLong {
                max: 10_000,
                actual: content.len(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_thought_valid() {
        let thought = Thought::new("Valid thought content".to_string()).unwrap();
        assert_eq!(thought.content, "Valid thought content");
        assert!(thought.id.is_none());
    }

    #[test]
    fn test_new_thought_empty_content() {
        let result = Thought::new("".to_string());
        assert!(matches!(result, Err(ThoughtError::EmptyContent)));
    }

    #[test]
    fn test_new_thought_whitespace_only() {
        let result = Thought::new("   \n\t  ".to_string());
        assert!(matches!(result, Err(ThoughtError::EmptyContent)));
    }

    #[test]
    fn test_new_thought_too_long() {
        let long_content = "a".repeat(10_001);
        let result = Thought::new(long_content);
        assert!(matches!(
            result,
            Err(ThoughtError::ContentTooLong {
                max: 10_000,
                actual: 10_001
            })
        ));
    }

    #[test]
    fn test_new_thought_max_length() {
        let max_content = "a".repeat(10_000);
        let thought = Thought::new(max_content).unwrap();
        assert_eq!(thought.content.len(), 10_000);
    }

    #[test]
    fn test_new_with_date() {
        let date = chrono::NaiveDate::from_ymd_opt(2024, 3, 15)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let thought = Thought::new_with_date("Backdated thought".to_string(), date).unwrap();
        assert_eq!(thought.content, "Backdated thought");
        assert_eq!(thought.created_at, date);
    }

    #[test]
    fn test_new_with_date_validates_content() {
        let date = Utc::now();
        let result = Thought::new_with_date("".to_string(), date);
        assert!(matches!(result, Err(ThoughtError::EmptyContent)));
    }

    #[test]
    fn test_thought_preserves_content() {
        let content = "Thought with [entity] and special chars: 'quotes', \"double\"".to_string();
        let thought = Thought::new(content.clone()).unwrap();
        assert_eq!(thought.content, content);
    }
}
