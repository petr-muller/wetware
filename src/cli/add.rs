/// Add command implementation
use crate::errors::ThoughtError;
use crate::services::{date_parser, thought_writer};
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::tui::compose::ComposeApp;
use std::io::IsTerminal;
use std::path::Path;

/// Execute the add command
///
/// With `content`, saves the thought directly. Without it, opens the interactive
/// composer, which offers entity completion, relative-date help, and a live preview.
///
/// # Arguments
/// * `content` - Text of the thought, including any `[Entity]` mentions; `None` composes interactively
/// * `date` - Optional date string; see [`date_parser`] for the accepted forms
/// * `db_path` - Path to the SQLite database file
pub fn execute(content: Option<String>, date: Option<String>, db_path: &Path) -> Result<(), ThoughtError> {
    match content {
        Some(content) => add_directly(content, date, db_path),
        None => compose_interactively(date, db_path),
    }
}

/// Save a thought supplied on the command line.
fn add_directly(content: String, date: Option<String>, db_path: &Path) -> Result<(), ThoughtError> {
    let date = date.as_deref().map(date_parser::parse_date).transpose()?;

    let mut conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let created = thought_writer::create_thought(&mut conn, &content, date)?;

    // Ambiguous mentions are reported, not linked; warn but do not fail.
    for ambiguous in &created.ambiguous {
        eprintln!("Warning: {}", ambiguous.describe());
    }

    // Success message with entity count
    if created.entities.is_empty() {
        println!("Thought added successfully (ID: {})", created.id);
    } else {
        println!(
            "Thought added successfully (ID: {}, {} entity reference{})",
            created.id,
            created.entities.len(),
            if created.entities.len() == 1 { "" } else { "s" }
        );
    }

    Ok(())
}

/// Open the interactive composer, restoring the terminal however it exits.
fn compose_interactively(date: Option<String>, db_path: &Path) -> Result<(), ThoughtError> {
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(ThoughtError::InvalidInput(
            "The interactive composer needs a terminal. Pass the thought as an argument instead: wet add \"...\"."
                .to_string(),
        ));
    }

    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;
    let mut app = ComposeApp::new(conn, date)?;

    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();

    // Report the count before propagating: those saves are already committed, and
    // a terminal error must not leave the user unsure whether their work landed.
    match app.saved_count {
        0 => println!("No thoughts added."),
        1 => println!("1 thought added."),
        n => println!("{} thoughts added.", n),
    }

    result
}
