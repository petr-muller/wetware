use clap::{Parser, Subcommand};

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { thought } => {
            println!("Adding thought: {}", thought);
            // Implementation will be added later
        }
        Commands::Thoughts => {
            println!("Listing all thoughts");
            // Implementation will be added later
        }
    }
}