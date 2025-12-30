/// Error types for note operations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoteError {
    #[error("Note content cannot be empty")]
    EmptyContent,

    #[error("Note exceeds maximum length of {max} characters (got {actual})")]
    ContentTooLong { max: usize, actual: usize },

    #[error("Failed to parse entity references: {0}")]
    ParseError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] rusqlite::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content_error_message() {
        let err = NoteError::EmptyContent;
        assert_eq!(err.to_string(), "Note content cannot be empty");
    }

    #[test]
    fn test_content_too_long_error_message() {
        let err = NoteError::ContentTooLong {
            max: 10000,
            actual: 10500,
        };
        assert_eq!(
            err.to_string(),
            "Note exceeds maximum length of 10000 characters (got 10500)"
        );
    }

    #[test]
    fn test_parse_error_message() {
        let err = NoteError::ParseError("malformed bracket".to_string());
        assert_eq!(
            err.to_string(),
            "Failed to parse entity references: malformed bracket"
        );
    }
}
