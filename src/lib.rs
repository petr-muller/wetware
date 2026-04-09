/// Wetware - Personal networked note-taking system
pub mod cli;
pub mod config;
pub mod errors;
pub mod input;
pub mod models;
pub mod services;
pub mod storage;
pub mod tui;

pub use errors::ThoughtError;
pub use models::{Entity, Thought};
