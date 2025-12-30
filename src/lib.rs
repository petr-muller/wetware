/// Wetware - Personal networked note-taking system
pub mod cli;
pub mod errors;
pub mod input;
pub mod models;
pub mod services;
pub mod storage;

pub use errors::ThoughtError;
pub use models::{Entity, Thought};
