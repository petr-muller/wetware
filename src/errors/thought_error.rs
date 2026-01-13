/// Error types for thought operations
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThoughtError {
    #[error("Thought content cannot be empty")]
    EmptyContent,

    #[error("Thought exceeds maximum length of {max} characters (got {actual})")]
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
        let err = ThoughtError::EmptyContent;
        assert_eq!(err.to_string(), "Thought content cannot be empty");
    }

    #[test]
    fn test_content_too_long_error_message() {
        let err = ThoughtError::ContentTooLong {
            max: 10000,
            actual: 10500,
        };
        assert_eq!(
            err.to_string(),
            "Thought exceeds maximum length of 10000 characters (got 10500)"
        );
    }

    #[test]
    fn test_parse_error_message() {
        let err = ThoughtError::ParseError("malformed bracket".to_string());
        assert_eq!(err.to_string(), "Failed to parse entity references: malformed bracket");
    }
}
