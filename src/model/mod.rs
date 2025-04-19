//! Domain model for the wetware application
//!
//! This module contains the core domain entities and logic for the wetware application.

use std::fmt;

/// A representation of a single thought
///
/// A thought is a brief snippet of information associated with a unique identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Thought {
    /// The unique identifier for the thought
    id: usize,

    /// The content of the thought
    content: String,
}

impl Thought {
    /// Create a new thought with the given id and content
    pub fn new(id: usize, content: String) -> Self {
        Self { id, content }
    }

    /// Get the unique identifier for this thought
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the content of this thought
    pub fn content(&self) -> &str {
        &self.content
    }
}

impl fmt::Display for Thought {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thought_display_shows_content() {
        let thought = Thought::new(1, "Test thought".to_string());
        assert_eq!(format!("{}", thought), "Test thought");
    }

    #[test]
    fn thought_accessors_return_correct_values() {
        let thought = Thought::new(42, "The meaning".to_string());
        assert_eq!(thought.id(), 42);
        assert_eq!(thought.content(), "The meaning");
    }
}
