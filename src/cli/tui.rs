//! CLI handler for the TUI subcommand
//!
//! Initializes the terminal, loads data, and launches the interactive TUI viewer.
//! Ensures terminal state is restored on exit, including on panic.

use std::path::Path;

use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::thoughts_repository::ThoughtsRepository;
use crate::tui::App;

/// Launch the interactive TUI thought viewer.
///
/// Opens the database, loads all thoughts and entities, initializes a full-screen
/// terminal, and runs the TUI event loop. Terminal state is restored on exit.
pub fn execute(db_path: &Path) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    let thoughts = ThoughtsRepository::list_all(&conn)?;
    let entities = EntitiesRepository::list_all(&conn)?;

    // Initialize terminal
    let mut terminal = ratatui::init();

    // Run the app, capturing any error to ensure terminal cleanup
    let result = App::new(thoughts, entities)
        .with_db_path(db_path.to_path_buf())
        .run(&mut terminal);

    // Always restore terminal, even if app errored
    ratatui::restore();

    result
}
