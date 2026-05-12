//! CLI handler for the TUI subcommand
//!
//! Initializes the terminal, loads data, and launches the interactive TUI viewer.
//! Ensures terminal state is restored on exit, including on panic.

use std::path::Path;

use crate::errors::ThoughtError;
use crate::models::SortOrder;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::thoughts_repository::ThoughtsRepository;
use crate::tui::App;

/// Launch the interactive TUI thought viewer.
pub fn execute(db_path: &Path, sort_order: SortOrder) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    let thoughts = ThoughtsRepository::list_all(&conn)?;
    let entities = EntitiesRepository::list_all(&conn)?;

    let mut terminal = ratatui::init();

    let result = App::new(thoughts, entities, sort_order)
        .with_db_path(db_path.to_path_buf())
        .run(&mut terminal);

    ratatui::restore();

    result
}
