/// Note domain model
use crate::errors::NoteError;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub id: Option<i64>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Note {
    /// Create a new note with validation
    pub fn new(content: String) -> Result<Self, NoteError> {
        Self::validate_content(&content)?;
        Ok(Self {
            id: None,
            content,
            created_at: Utc::now(),
        })
    }

    /// Validate note content
    fn validate_content(content: &str) -> Result<(), NoteError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(NoteError::EmptyContent);
        }
        if content.len() > 10_000 {
            return Err(NoteError::ContentTooLong {
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
    fn test_new_note_valid() {
        let note = Note::new("Valid note content".to_string()).unwrap();
        assert_eq!(note.content, "Valid note content");
        assert!(note.id.is_none());
    }

    #[test]
    fn test_new_note_empty_content() {
        let result = Note::new("".to_string());
        assert!(matches!(result, Err(NoteError::EmptyContent)));
    }

    #[test]
    fn test_new_note_whitespace_only() {
        let result = Note::new("   \n\t  ".to_string());
        assert!(matches!(result, Err(NoteError::EmptyContent)));
    }

    #[test]
    fn test_new_note_too_long() {
        let long_content = "a".repeat(10_001);
        let result = Note::new(long_content);
        assert!(matches!(
            result,
            Err(NoteError::ContentTooLong {
                max: 10_000,
                actual: 10_001
            })
        ));
    }

    #[test]
    fn test_new_note_max_length() {
        let max_content = "a".repeat(10_000);
        let note = Note::new(max_content).unwrap();
        assert_eq!(note.content.len(), 10_000);
    }

    #[test]
    fn test_note_preserves_content() {
        let content = "Note with [entity] and special chars: 'quotes', \"double\"".to_string();
        let note = Note::new(content.clone()).unwrap();
        assert_eq!(note.content, content);
    }
}
