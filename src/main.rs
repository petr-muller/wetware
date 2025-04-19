use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod model;
mod storage;

use storage::Storage;

#[derive(Parser)]
#[command(name = "wet")]
#[command(about = "Wetware - track, organize, and process thoughts")]
struct Cli {
    /// Path to the database file
    #[arg(short, long, default_value = "wetware.db")]
    database: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new thought
    Add {
        /// The thought text
        thought: String,
    },
    /// List all thoughts
    Thoughts,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Create and initialize storage with the provided database path
    let storage = storage::SqliteStorage::new(&cli.database);
    storage.init().context("Failed to initialize storage")?;

    match cli.command {
        Commands::Add { thought } => {
            println!("Adding thought: {}", thought);
            let thought = storage
                .save_thought(&thought)
                .context("Failed to save thought")?;
            println!("Thought saved with ID: {}", thought.id());
        }
        Commands::Thoughts => {
            println!("Listing all thoughts:");
            let thoughts = storage
                .get_thoughts()
                .context("Failed to retrieve thoughts")?;

            if thoughts.is_empty() {
                println!("No thoughts found.");
                return Ok(());
            }

            for thought in thoughts {
                println!("{}: {}", thought.id(), thought.content());
            }
        }
    }

    Ok(())
}
