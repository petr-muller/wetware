/// Entity edit command implementation
use crate::errors::ThoughtError;
use crate::input::editor;
use crate::models::entity::Entity;
use crate::services::entity_parser;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use std::fs;
use std::path::{Path, PathBuf};

/// Execute the entity edit command
///
/// # Arguments
/// * `entity_name` - Name of the entity to edit (case-insensitive)
/// * `description` - Inline description text (mutually exclusive with other methods)
/// * `description_file` - Path to file containing description (mutually exclusive)
/// * `db_path` - Optional database path
///
/// # Input Methods
/// 1. Inline: `--description "text"` - Provide description directly
/// 2. File: `--description-file path.txt` - Read description from file
/// 3. Interactive: No flags - Launch editor with current description
///
/// # Returns
/// * `Ok(())` - Description successfully updated or removed
/// * `Err(ThoughtError)` - Entity not found, file errors, editor errors, etc.
pub fn execute(
    entity_name: &str,
    description: Option<String>,
    description_file: Option<PathBuf>,
    db_path: Option<&Path>,
) -> Result<(), ThoughtError> {
    // T031: Check mutual exclusivity of input methods (both flags cannot be used together)
    if description.is_some() && description_file.is_some() {
        eprintln!("Error: Cannot use multiple input methods simultaneously");
        eprintln!();
        eprintln!("Usage:");
        eprintln!("  wet entity edit <name> --description \"text\"       # Inline");
        eprintln!("  wet entity edit <name> --description-file file    # From file");
        eprintln!("  wet entity edit <name>                            # Interactive editor");
        return Err(ThoughtError::InvalidInput(
            "Cannot use --description and --description-file together".to_string(),
        ));
    }

    // Get database connection
    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    // T030: Verify entity exists
    let entity_opt = EntitiesRepository::find_by_name(&conn, entity_name)?;
    if entity_opt.is_none() {
        eprintln!("Error: Entity '{}' not found", entity_name);
        eprintln!();
        eprintln!("Hint: Create the entity first by referencing it in a thought:");
        eprintln!("  wet add \"Learning about [{}] today\"", entity_name);
        return Err(ThoughtError::EntityNotFound(entity_name.to_string()));
    }

    // Get description text based on input method
    let description_text = if let Some(inline_desc) = description {
        // T025: Inline description
        inline_desc
    } else if let Some(file_path) = description_file {
        // T026: File input
        match fs::read_to_string(&file_path) {
            Ok(content) => content,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    eprintln!("Error: Description file '{}' not found", file_path.display());
                } else {
                    eprintln!(
                        "Error: Failed to read description file '{}': {}",
                        file_path.display(),
                        e
                    );
                }
                return Err(ThoughtError::FileError(e));
            }
        }
    } else {
        // T027: Interactive editor
        let current_description = entity_opt.as_ref().and_then(|e| e.description.as_deref());
        editor::launch_editor(current_description)?
    };

    // T028: Trim and check if empty (whitespace-only = removal)
    let trimmed = description_text.trim();
    let final_description = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    };

    // T029: Extract entity references and auto-create entities
    if let Some(ref desc) = final_description {
        let referenced_entities = entity_parser::extract_unique_entities(desc);
        for ref_entity_name in referenced_entities {
            let ref_entity = Entity::new(ref_entity_name);
            EntitiesRepository::find_or_create(&conn, &ref_entity)?;
        }
    }

    // Update description in database
    EntitiesRepository::update_description(&conn, entity_name, final_description.clone())?;

    // Print success message
    if final_description.is_some() {
        println!("Description updated for entity '{}'", entity_name);
    } else {
        println!("Description removed for entity '{}'", entity_name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // T022: Unit tests for file reading logic
    #[test]
    fn test_file_reading_success() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test description").unwrap();
        temp_file.flush().unwrap();

        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(content.trim(), "Test description");
    }

    #[test]
    fn test_file_reading_not_found() {
        let result = fs::read_to_string("/nonexistent/file.txt");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn test_whitespace_trimming() {
        let text = "  content  \n  ";
        assert_eq!(text.trim(), "content");
    }

    #[test]
    fn test_whitespace_only_empty_after_trim() {
        let text = "   \n  \t  ";
        assert_eq!(text.trim(), "");
        assert!(text.trim().is_empty());
    }
}
