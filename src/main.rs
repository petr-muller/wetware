use clap::{Parser, Subcommand};
use anyhow::{Result, Context};

mod model;
mod storage;

use storage::Storage;

#[derive(Parser)]
#[command(name = "wet")]
#[command(about = "Wetware - track, organize, and process thoughts")]
struct Cli {
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
    
    // Create and initialize storage
    let db_path = "wetware.db";
    let storage = storage::SqliteStorage::new(db_path);
    storage.init().context("Failed to initialize storage")?;

    match cli.command {
        Commands::Add { thought } => {
            println!("Adding thought: {}", thought);
            let thought = storage.save_thought(&thought)
                .context("Failed to save thought")?;
            println!("Thought saved with ID: {}", thought.id());
        }
        Commands::Thoughts => {
            println!("Listing all thoughts:");
            let thoughts = storage.get_thoughts()
                .context("Failed to retrieve thoughts")?;
            
            if thoughts.is_empty() {
                println!("No thoughts found.");
                return Ok(());
            }
            
            for thought in thoughts {
                println!("{}: {}", thought.id(), thought);
            }
        }
    }
    
    Ok(())
}