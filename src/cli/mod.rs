/// CLI module for command-line interface
pub mod add;
pub mod entities;
pub mod notes;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wet")]
#[command(about = "Wetware - Personal networked notes", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new note
    Add {
        /// Note content
        content: String,
    },
    /// List all notes
    Notes {
        /// Filter notes by entity name
        #[arg(long)]
        on: Option<String>,
    },
    /// List all entities
    Entities,
}
