//! Persistence layer for the wetware application
//!
//! This module provides abstraction for data storage and retrieval.

mod sqlite;

use crate::model::Thought;
use anyhow::Result;

pub use self::sqlite::SqliteStorage;

/// Storage trait that defines the interface for persisting and retrieving thoughts
pub trait Storage {
    /// Initialize the storage, creating necessary structures if needed
    fn init(&self) -> Result<()>;
    
    /// Save a thought to the storage
    fn save_thought(&self, content: &str) -> Result<Thought>;
    
    /// Get all thoughts from the storage
    fn get_thoughts(&self) -> Result<Vec<Thought>>;
    
    /// Get a thought by its ID
    fn get_thought(&self, id: usize) -> Result<Option<Thought>>;
}