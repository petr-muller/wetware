use clap::{Parser, Subcommand};

mod model;

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
            let _thought = model::Thought::new(1, thought);
            // Persistence implementation will be added later
        }
        Commands::Thoughts => {
            println!("Listing all thoughts");
            // Sample thoughts (will be replaced with actual storage later)
            let thoughts = vec![
                model::Thought::new(1, "First thought".to_string()),
                model::Thought::new(2, "Second thought".to_string()),
            ];
            
            for thought in thoughts {
                println!("{}", thought);
            }
        }
    }
}