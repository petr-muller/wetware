/// CLI module for command-line interface
pub mod add;
pub mod delete;
pub mod edit;
pub mod entities;
pub mod entity_edit;
pub mod thoughts;
pub mod tui;

use crate::services::color_mode::ColorMode;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wet")]
#[command(about = "Wetware - Personal networked notes", long_about = None)]
pub struct Cli {
    /// Control color output
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    pub color: ColorMode,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new thought
    Add {
        /// Thought content
        content: String,
        /// Date for the thought in YYYY-MM-DD format (defaults to today)
        #[arg(long)]
        date: Option<String>,
    },
    /// List all thoughts
    Thoughts {
        /// Filter thoughts by entity name
        #[arg(long)]
        on: Option<String>,
    },
    /// Edit an existing thought
    Edit {
        /// ID of the thought to edit (visible in `wet` listing output as [id])
        id: i64,
        /// New content for the thought (mutually exclusive with --editor)
        content: Option<String>,
        /// New date for the thought in YYYY-MM-DD format
        #[arg(long)]
        date: Option<String>,
        /// Open the thought in an interactive editor (mutually exclusive with CONTENT)
        #[arg(long, conflicts_with = "content")]
        editor: bool,
    },
    /// Delete a thought by ID
    Delete {
        /// ID of the thought to delete (visible in `wet` listing output as [id])
        id: i64,
    },
    /// Launch interactive TUI thought viewer
    Tui,
    /// List all entities
    Entities,
    /// Entity operations
    Entity {
        #[command(subcommand)]
        command: EntityCommands,
    },
}

#[derive(Subcommand)]
pub enum EntityCommands {
    /// Edit entity description
    Edit {
        /// Entity name (case-insensitive)
        entity_name: String,
        /// Inline description text (mutually exclusive with --description-file)
        #[arg(long)]
        description: Option<String>,
        /// Path to file containing description (mutually exclusive with --description)
        #[arg(long)]
        description_file: Option<std::path::PathBuf>,
    },
}
