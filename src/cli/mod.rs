/// CLI module for command-line interface
pub mod add;
pub mod entities;
pub mod thoughts;

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
    },
    /// List all thoughts
    Thoughts {
        /// Filter thoughts by entity name
        #[arg(long)]
        on: Option<String>,
    },
    /// List all entities
    Entities,
}
