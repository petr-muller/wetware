/// Entity rename command implementation
use crate::errors::ThoughtError;
use crate::services::entity_parser::rewrite_entity_references;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Execute the entity rename command
///
/// Renames an entity and rewrites every literal reference to its old name
/// (in linked thoughts and in any entity's description) to the new name.
/// `thought_entities` links are keyed by entity ID and are left untouched -
/// only stored text is rewritten. The whole operation is atomic.
///
/// # Arguments
/// * `entity_name` - Current name of the entity to rename (case-insensitive)
/// * `new_name` - New name for the entity
/// * `db_path` - Database path
///
/// # Returns
/// * `Ok(())` - Entity successfully renamed
/// * `Err(ThoughtError)` - Entity not found, new name already in use, or storage error
pub fn execute(entity_name: &str, new_name: &str, db_path: &Path) -> Result<(), ThoughtError> {
    if new_name.trim().is_empty() {
        return Err(ThoughtError::InvalidInput(
            "New entity name cannot be empty".to_string(),
        ));
    }

    let mut conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let entity = EntitiesRepository::find_by_name(&conn, entity_name)?;
    let Some(entity) = entity else {
        eprintln!("Error: Entity '{}' not found", entity_name);
        eprintln!();
        eprintln!("Hint: Create the entity first by referencing it in a thought:");
        eprintln!("  wet add \"Learning about [{}] today\"", entity_name);
        return Err(ThoughtError::EntityNotFound(entity_name.to_string()));
    };

    if let Some(existing) = EntitiesRepository::find_by_name(&conn, new_name)?
        && existing.id != entity.id
    {
        eprintln!("Error: Entity '{}' already exists", new_name);
        return Err(ThoughtError::EntityAlreadyExists(new_name.to_string()));
    }

    let tx = conn.transaction()?;

    // Rewrite descriptions of every entity (including this one's own) before renaming
    // the row, since lookups here are still by the pre-rename name.
    for other in EntitiesRepository::list_all(&tx)? {
        if let Some(desc) = &other.description {
            let rewritten = rewrite_entity_references(desc, &entity.name, new_name);
            if rewritten != *desc {
                EntitiesRepository::update_description(&tx, &other.name, Some(rewritten))?;
            }
        }
    }

    // Rewrite content of every thought currently linked to this entity.
    for thought in ThoughtsRepository::list_by_entity(&tx, &entity.name)? {
        let rewritten = rewrite_entity_references(&thought.content, &entity.name, new_name);
        if rewritten != thought.content {
            ThoughtsRepository::update(&tx, thought.id.unwrap(), &rewritten, thought.created_at)?;
        }
    }

    EntitiesRepository::rename(&tx, &entity.name, new_name)?;

    tx.commit()?;

    println!("Entity '{}' renamed to '{}'.", entity_name, new_name);

    Ok(())
}
