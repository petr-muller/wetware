#![allow(clippy::upper_case_acronyms)]

mod model;
mod store;
mod tui;

use crate::model::thoughts::Thought;
use crate::store::sqlite::SqliteStoreError;
use crate::tui::app::Thoughts;
use chrono::Local;
use clap::{command, Args, Parser, Subcommand};
use indexmap::IndexMap;
use interim::{parse_date_string, Dialect};
use ratatui::{TerminalOptions, Viewport};
use std::io::IsTerminal;

#[derive(Debug, Parser)]
#[clap(name = "wet", version)]
struct Wet {
    #[clap(flatten)]
    globals: GlobalFlags,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new thought
    #[command(name = "add", arg_required_else_help = true)]
    Add {
        /// The thought to add
        thought: String,
        #[arg(long, default_value = "today")]
        date: String,
    },
    /// Edit a thought identified by ID
    #[command(name = "edit", arg_required_else_help = true)]
    Edit {
        /// The ID of the thought to edit
        thought_id: u32,
        /// Edited thought
        new_thought: Option<String>,
        #[arg(long = "date")]
        new_date: Option<String>,
    },
    /// List thoughts
    #[command(name = "thoughts")]
    Thoughts {
        #[arg(long = "on")]
        entity: Option<String>,
    },

    /// Entity subcommand
    #[clap(subcommand)]
    #[command(name = "entity")]
    Entity(EntityCommands),

    /// List entities
    #[command(name = "entities")]
    Entities {},

    /// TUI
    #[command(name = "tui")]
    Tui {},
}

#[derive(Debug, Subcommand)]
enum EntityCommands {
    #[command(name = "list")]
    List {},
    #[command(name = "describe")]
    Describe {
        /// Entity name
        entity: String,
        /// Entity description
        description: String,
    },
}

#[derive(Debug, Args)]
struct GlobalFlags {
    /// The path to the database
    #[arg(long, env = "WETWARE_DB_PATH", required(false))]
    db: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Wet::parse();

    let db = args.globals.db.unwrap_or_else(|| {
        eprintln!("No database path provided");
        std::process::exit(1);
    });

    match args.command {
        Commands::Entities {} => {
            list_entities(&db)?;
        }
        Commands::Entity(command) => match command {
            EntityCommands::List {} => {
                list_entities(&db)?;
            }
            EntityCommands::Describe {
                entity,
                description,
            } => {
                let store = store::sqlite::open(&db)?;
                let mut old = store.get_entity(&entity)?;

                old.description = description;

                match store.edit_entity(old) {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!("Failed to edit entity: {}", e);
                        return Err(Box::new(e));
                    }
                }
            }
        },
        Commands::Thoughts { entity } => {
            // TODO(muller): Do not create DB file on get when nonexistent
            // TODO(muller): Somehow eliminate the matches and use map_err?
            let store = store::sqlite::open(&db)?;
            let raw = store.get_thoughts(entity)?;

            let mut thoughts = IndexMap::new();
            for (id, item) in raw {
                thoughts.insert(id, item.as_thought()?);
            }

            let tui_result;
            // Hypothetically can work without TTY after crossterm-rs/crossterm#919 is fixed?
            if std::io::stdout().is_terminal() {
                let output_size = u16::try_from(thoughts.len()).unwrap_or(u16::MAX);

                // Does not work without TTY because of the following issue:
                //
                // cursor::position() fails when piping stdout:
                // https://github.com/crossterm-rs/crossterm/issues/919
                let mut terminal = ratatui::init_with_options(TerminalOptions {
                    viewport: Viewport::Inline(output_size),
                });

                tui_result = Thoughts::populated(thoughts).noninteractive(&mut terminal);
                ratatui::restore();
            } else {
                tui_result = Thoughts::populated(thoughts).raw();
            }
            return match tui_result {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("TUI failed: {}", e);
                    Err(Box::new(e))
                }
            };
        }
        Commands::Tui {} => {
            let mut terminal = ratatui::init_with_options(TerminalOptions {
                viewport: Viewport::Inline(12),
            });

            let store = store::sqlite::open(&db)?;

            let raw = store.get_thoughts(None)?;

            let mut thoughts = IndexMap::new();
            for (id, item) in raw {
                thoughts.insert(id, item.as_thought()?);
            }

            let tui_result = Thoughts::populated(thoughts).interactive(&mut terminal);
            ratatui::restore();
            return match tui_result {
                Ok(()) => Ok(()),
                Err(e) => {
                    eprintln!("TUI failed: {}", e);
                    Err(Box::new(e))
                }
            };
        }
        Commands::Edit {
            thought_id,
            new_thought,
            new_date,
        } => {
            let store = store::sqlite::open(&db)?;
            let old = store.get_thought(thought_id)?.as_thought()?;

            let date = if let Some(date) = new_date {
                match parse_date_string(date.as_str(), Local::now(), Dialect::Us) {
                    Ok(date) => date.date_naive(),
                    Err(e) => {
                        eprintln!("Failed to parse --date: {}", e);
                        return Err(Box::new(e));
                    }
                }
            } else {
                old.added
            };

            let raw = if let Some(raw) = new_thought {
                raw
            } else {
                old.raw
            };

            let thought = match Thought::from_input(raw, date) {
                Ok(thought) => thought,
                Err(e) => {
                    eprintln!("Failed to read thought: {}", e);
                    return Err(Box::new(e));
                }
            };

            match store.edit_thought(thought_id, thought) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Failed to edit thought: {}", e);
                    return Err(Box::new(e));
                }
            }
        }
        Commands::Add { thought, date } => {
            // TODO(muller): Create DB file when nonexistent but warn about it / maybe ask about it
            let store = store::sqlite::open(&db)?;

            let when = match parse_date_string(date.as_str(), Local::now(), Dialect::Us) {
                Ok(date) => date.date_naive(),
                Err(e) => {
                    eprintln!("Failed to parse --date: {}", e);
                    return Err(Box::new(e));
                }
            };

            let thought = match Thought::from_input(thought, when) {
                Ok(thought) => thought,
                Err(e) => {
                    eprintln!("Failed to read thought: {}", e);
                    return Err(Box::new(e));
                }
            };

            match store.add_thought(thought) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("Failed to add thought: {}", e);
                    return Err(Box::new(e));
                }
            }
        }
    }
    Ok(())
}

fn list_entities(db: &String) -> Result<(), Box<SqliteStoreError>> {
    let store = store::sqlite::open(db)?;

    let entities = match store.get_entities() {
        Ok(entities) => entities,
        Err(e) => {
            eprintln!("Failed to get thoughts: {}", e);
            return Err(Box::new(e));
        }
    };

    if entities.is_empty() {
        println!("No entities in the database");
    } else {
        for entity in entities {
            println!("{}", entity);
        }
    }
    Ok(())
}
