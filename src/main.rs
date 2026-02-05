use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process;
use wetware::cli::{Cli, Commands, EntityCommands};

fn main() {
    let cli = Cli::parse();

    // Get database path from environment variable or use default
    let db_path = env::var("WETWARE_DB").ok().map(PathBuf::from);

    let result = match cli.command {
        Commands::Add { content } => wetware::cli::add::execute(content, db_path.as_deref()),
        Commands::Thoughts { on } => wetware::cli::thoughts::execute(db_path.as_deref(), on.as_deref(), cli.color),
        Commands::Entities => wetware::cli::entities::execute(db_path.as_deref()),
        Commands::Entity { command } => match command {
            EntityCommands::Edit {
                entity_name,
                description,
                description_file,
            } => wetware::cli::entity_edit::execute(&entity_name, description, description_file, db_path.as_deref()),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
