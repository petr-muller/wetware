/// Entity show command implementation
use crate::errors::ThoughtError;
use crate::services::color_mode::ColorMode;
use crate::services::entity_styler::EntityStyler;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_relations_repository::EntityRelationsRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use std::path::Path;

/// Number of most recent thoughts to display for the entity
const LATEST_THOUGHTS_LIMIT: usize = 5;

/// Execute the entity show command
///
/// Displays an entity's full description (styled consistently with thought content,
/// with entity references colored and aliases rendered as their display text) followed
/// by the 5 most recent thoughts linked to the entity.
///
/// # Arguments
/// * `entity_name` - Name of the entity to show (case-insensitive)
/// * `db_path` - Database path
/// * `color_mode` - Whether to apply ANSI styling to entity references
///
/// # Returns
/// * `Ok(())` - Success
/// * `Err(ThoughtError::EntityNotFound)` - No entity with the given name exists
pub fn execute(entity_name: &str, db_path: &Path, color_mode: ColorMode) -> Result<(), ThoughtError> {
    let conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let entity = EntitiesRepository::find_by_name(&conn, entity_name)?
        .ok_or_else(|| ThoughtError::EntityNotFound(entity_name.to_string()))?;

    let mut styler = EntityStyler::new(color_mode.should_use_colors());

    println!("{}", entity.canonical_name);

    if let Some(description) = &entity.description {
        println!();
        println!("{}", styler.render_content(description));
    }

    let parents = EntityRelationsRepository::list_parents(&conn, entity.id.unwrap())?;
    let children = EntityRelationsRepository::list_children(&conn, entity.id.unwrap())?;

    if !parents.is_empty() {
        let names: Vec<_> = parents.iter().map(|e| e.canonical_name.as_str()).collect();
        println!();
        println!("Parents: {}", names.join(", "));
    }
    if !children.is_empty() {
        let names: Vec<_> = children.iter().map(|e| e.canonical_name.as_str()).collect();
        println!();
        println!("Children: {}", names.join(", "));
    }

    println!();
    println!("Latest thoughts:");

    let thoughts = ThoughtsRepository::list_latest_by_entity(&conn, entity_name, LATEST_THOUGHTS_LIMIT)?;

    if thoughts.is_empty() {
        println!("No thoughts found for entity: {}", entity.canonical_name);
    } else {
        for thought in thoughts {
            let styled_content = styler.render_content(thought.content.trim());
            println!(
                "[{}] {} - {}",
                thought.id.unwrap_or(0),
                thought.created_at.format("%Y-%m-%d"),
                styled_content
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entity::Entity;
    use crate::storage::connection::get_memory_connection;
    use rusqlite::Connection;

    fn setup_entity(conn: &Connection, name: &str, description: Option<&str>) {
        let entity = Entity::new(name.to_string());
        EntitiesRepository::find_or_create(conn, &entity).unwrap();
        if let Some(desc) = description {
            EntitiesRepository::update_description(conn, name, Some(desc.to_string())).unwrap();
        }
    }

    #[test]
    fn test_entity_not_found_returns_error() {
        let conn = get_memory_connection().unwrap();
        crate::storage::migrations::run_migrations(&conn).unwrap();

        let result = EntitiesRepository::find_by_name(&conn, "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_entity_without_description_has_none() {
        let conn = get_memory_connection().unwrap();
        crate::storage::migrations::run_migrations(&conn).unwrap();

        setup_entity(&conn, "rust", None);

        let entity = EntitiesRepository::find_by_name(&conn, "rust").unwrap().unwrap();
        assert!(entity.description.is_none());
    }

    #[test]
    fn test_show_with_parents_and_children_succeeds() {
        use crate::storage::entity_relations_repository::EntityRelationsRepository;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = get_connection(&db_path).unwrap();
        crate::storage::migrations::run_migrations(&conn).unwrap();

        setup_entity(&conn, "amazon", None);
        setup_entity(&conn, "aws", None);
        setup_entity(&conn, "big tech", None);

        let amazon = EntitiesRepository::find_by_name(&conn, "amazon").unwrap().unwrap();
        let aws = EntitiesRepository::find_by_name(&conn, "aws").unwrap().unwrap();
        let big_tech = EntitiesRepository::find_by_name(&conn, "big tech").unwrap().unwrap();
        EntityRelationsRepository::add_relation(&conn, aws.id.unwrap(), amazon.id.unwrap()).unwrap();
        EntityRelationsRepository::add_relation(&conn, amazon.id.unwrap(), big_tech.id.unwrap()).unwrap();
        drop(conn);

        let result = execute("amazon", &db_path, ColorMode::Never);
        assert!(result.is_ok());
    }

    #[test]
    fn test_show_without_relations_succeeds() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = get_connection(&db_path).unwrap();
        crate::storage::migrations::run_migrations(&conn).unwrap();

        setup_entity(&conn, "rust", None);
        drop(conn);

        let result = execute("rust", &db_path, ColorMode::Never);
        assert!(result.is_ok());
    }

    #[test]
    fn test_entity_with_description_is_returned() {
        let conn = get_memory_connection().unwrap();
        crate::storage::migrations::run_migrations(&conn).unwrap();

        setup_entity(&conn, "rust", Some("A systems programming language."));

        let entity = EntitiesRepository::find_by_name(&conn, "rust").unwrap().unwrap();
        assert_eq!(entity.description.as_deref(), Some("A systems programming language."));
    }
}
