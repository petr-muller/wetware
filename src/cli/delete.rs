/// Delete command implementation
use crate::errors::ThoughtError;
use crate::storage::connection::get_connection;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Execute the delete command
///
/// Deletes a thought by its numeric ID. Prints the deleted thought's content
/// as confirmation.
pub fn execute(id: i64, db_path: &Path) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    // Verify thought exists before deleting (also shows what was deleted)
    let thought = ThoughtsRepository::get_by_id(&conn, id)?;
    ThoughtsRepository::delete(&conn, id)?;

    let date = thought.created_at.format("%Y-%m-%d");
    println!("Deleted thought {id} ({date}): {}", thought.content);

    Ok(())
}
