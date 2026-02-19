/// Edit command implementation
use crate::errors::ThoughtError;
use crate::input::editor;
use crate::models::entity::Entity;
use crate::services::entity_parser;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use chrono::NaiveDate;
use std::path::Path;

/// Execute the edit command
///
/// Edits an existing thought's content and/or date. At least one of `content`,
/// `date`, or `use_editor` must be provided. All changes are applied atomically:
/// if any step fails, the thought is left in its original state.
///
/// # Arguments
/// * `id` - Numeric ID of the thought to edit (shown as `[id]` in `wet` listing)
/// * `content` - Optional new text content for the thought
/// * `date` - Optional new date string in YYYY-MM-DD format
/// * `use_editor` - If true, open the thought's current content in an interactive editor
/// * `db_path` - Optional path to the SQLite database file
pub fn execute(
    id: i64,
    content: Option<String>,
    date: Option<String>,
    use_editor: bool,
    db_path: Option<&Path>,
) -> Result<(), ThoughtError> {
    // Validate at least one edit argument was provided
    if content.is_none() && date.is_none() && !use_editor {
        return Err(ThoughtError::InvalidInput(
            "At least one of CONTENT, --date, or --editor must be provided".to_string(),
        ));
    }

    let mut conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    // Fetch existing thought (needed to resolve editor content and fallback date/content)
    let existing = ThoughtsRepository::get_by_id(&conn, id).map_err(|e| match e {
        ThoughtError::StorageError(rusqlite::Error::QueryReturnedNoRows) => ThoughtError::ThoughtNotFound(id),
        other => other,
    })?;

    // Resolve new content
    let new_content: Option<String> = if use_editor {
        let edited = match editor::launch_editor(Some(&existing.content)) {
            Ok(c) => c,
            Err(ThoughtError::EditorLaunchFailed(ref editor_name)) => {
                eprintln!(
                    "Warning: Editor '{}' exited abnormally. No changes made to thought {}.",
                    editor_name, id
                );
                return Ok(());
            }
            Err(e) => return Err(e),
        };
        if edited == existing.content {
            None // No content change
        } else {
            Some(edited)
        }
    } else {
        content
    };

    // Validate content if provided
    if let Some(ref c) = new_content
        && c.trim().is_empty()
    {
        return Err(ThoughtError::EmptyContent);
    }

    // Parse new date if provided
    let new_date = if let Some(ref date_str) = date {
        let naive = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            ThoughtError::InvalidInput(format!("Invalid date format '{}'. Expected YYYY-MM-DD.", date_str))
        })?;
        Some(naive.and_hms_opt(0, 0, 0).unwrap().and_utc())
    } else {
        None
    };

    // If editor produced no change and no date was given, nothing to do
    if new_content.is_none() && new_date.is_none() {
        println!("No changes made to thought {}.", id);
        return Ok(());
    }

    // Resolve final values (fall back to existing when not provided)
    let final_content = new_content.as_deref().unwrap_or(&existing.content);
    let final_date = new_date.unwrap_or(existing.created_at);
    let content_changed = new_content.is_some();

    // Apply all changes atomically within a transaction
    let tx = conn.transaction()?;

    ThoughtsRepository::update(&tx, id, final_content, final_date)?;

    if content_changed {
        EntitiesRepository::unlink_all_from_thought(&tx, id)?;
        let entity_names = entity_parser::extract_unique_entities(final_content);
        for entity_name in &entity_names {
            let entity = Entity::new(entity_name.clone());
            let entity_id = EntitiesRepository::find_or_create(&tx, &entity)?;
            EntitiesRepository::link_to_thought(&tx, entity_id, id)?;
        }
    }

    tx.commit()?;

    println!("Thought {} updated.", id);
    Ok(())
}
