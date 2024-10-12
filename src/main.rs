#![allow(clippy::upper_case_acronyms)]

mod tui;
mod store;
mod model;

use std::io::IsTerminal;
use chrono::Local;
use clap::{Args, command, Parser, Subcommand};
use interim::{parse_date_string, Dialect};
use ratatui::{TerminalOptions, Viewport};
use crate::model::thoughts::Thought;
use crate::tui::app::Thoughts;

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
        #[arg(long)]
        date: Option<String>,
    },
    /// List thoughts
    #[command(name = "thoughts")]
    Thoughts {
        #[arg(long = "on")]
        entity: Option<String>,
    },
    /// List entities
    #[command(name = "entities")]
    Entities {},

    /// TUI
    #[command(name = "tui")]
    Tui {},
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
            let store = match store::sqlite::open(db) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("Failed to open thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };

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
        }
        Commands::Thoughts { entity } => {
            // TODO(muller): Do not create DB file on get when nonexistent
            // TODO(muller): Somehow eliminate the matches and use map_err?
            let store = match store::sqlite::open(db) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("Failed to open thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };

            // TODO(muller): implement entity filter as fluent api instead of a param
            let raw = match store.get_thoughts(entity) {
                Ok(thoughts) => thoughts,
                Err(e) => {
                    eprintln!("Failed to get thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };


            let mut thoughts = vec![];
            for item in raw {
                thoughts.push(item.as_thought().unwrap())
            }

            let tui_result;
            // Hypothetically can work without TTY after crossterm-rs/crossterm#919 is fixed?
            if std::io::stdout().is_terminal() {
                let output_size = match u16::try_from(thoughts.len()) {
                    Ok(x) => { x }
                    Err(_) => { u16::MAX }
                };

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

            let store = match store::sqlite::open(db) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("Failed to open thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };

            let raw = match store.get_thoughts(None) {
                Ok(thoughts) => thoughts,
                Err(e) => {
                    eprintln!("Failed to get thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };

            let mut thoughts = vec![];
            for item in raw {
                thoughts.push(item.as_thought().unwrap())
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
        Commands::Add { thought, date } => {
            // TODO(muller): Create DB file when nonexistent but warn about it / maybe ask about it
            let store = match store::sqlite::open(db) {
                Ok(store) => store,
                Err(e) => {
                    eprintln!("Failed to open thoughts: {}", e);
                    return Err(Box::new(e));
                }
            };


            let when = match date {
                None => { Local::now().date_naive() }
                Some(date) => {
                    match parse_date_string(date.as_str(), Local::now(), Dialect::Us) {
                        Ok(date) => { date.date_naive() }
                        Err(e) => {
                            eprintln!("Failed to parse --date: {}", e);
                            return Err(Box::new(e));
                        }
                    }
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
