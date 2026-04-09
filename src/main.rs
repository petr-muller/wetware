use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::process;
use wetware::cli::{Cli, Commands, EntityCommands};
use wetware::config;
use wetware::storage::{default_db_path_in, ensure_data_dir, resolve_data_dir};

fn main() {
    let cli = Cli::parse();

    // Resolve data directory: WETWARE_DATA_DIR env var > XDG default (release only)
    let data_dir_override = env::var("WETWARE_DATA_DIR").ok().map(PathBuf::from);
    let data_dir = match resolve_data_dir(data_dir_override.as_deref()) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    };

    // Ensure data directory exists and config is initialized
    if let Err(e) = ensure_data_dir(&data_dir) {
        eprintln!("Error creating data directory: {e}");
        process::exit(1);
    }
    if let Err(e) = config::ensure_config(&data_dir) {
        eprintln!("Error initializing config: {e}");
        process::exit(1);
    }

    // Database path: WETWARE_DB env var > <data_dir>/default.db
    let db_path = env::var("WETWARE_DB")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_db_path_in(&data_dir));
    let db_path = Some(db_path);

    let result = match cli.command {
        Commands::Tui => wetware::cli::tui::execute(db_path.as_deref()),
        Commands::Add { content, date } => wetware::cli::add::execute(content, date, db_path.as_deref()),
        Commands::Edit {
            id,
            content,
            date,
            editor,
        } => wetware::cli::edit::execute(id, content, date, editor, db_path.as_deref()),
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
