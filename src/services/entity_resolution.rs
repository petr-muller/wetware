/// Entity resolution service - resolves extracted entity-reference names to entity IDs,
/// consulting known aliases before falling back to creating a new literal entity.
use crate::errors::ThoughtError;
use crate::models::entity::Entity;
use crate::storage::entities_repository::EntitiesRepository;
use rusqlite::Connection;

/// Resolve `name` (as extracted from a `[bracket]` mention in thought/description text)
/// to an entity ID, creating a new entity if the name doesn't match any existing
/// canonical name or alias.
///
/// - Canonical name or unambiguous alias match: returns that entity's ID, without
///   creating anything.
/// - No match anywhere: creates a new entity literally named `name` (existing
///   behavior, unchanged).
/// - Ambiguous alias match (the same alias registered to more than one entity):
///   prints a warning to stderr naming the alias and its candidate entities, and
///   returns `Ok(None)` - the mention is not linked to any entity, and no new entity
///   is created. This is a soft, non-fatal condition: it never fails the caller's
///   overall add/edit operation, mirroring how a failed editor launch is a warning
///   rather than a hard error elsewhere in the CLI.
pub fn resolve_or_create_entity(conn: &Connection, name: &str) -> Result<Option<i64>, ThoughtError> {
    match resolve_entity(conn, name)? {
        Resolution::Entity(id) => Ok(Some(id)),
        Resolution::Ambiguous(ambiguous) => {
            eprintln!("Warning: {}", ambiguous.describe());
            Ok(None)
        }
    }
}

/// A mention that could not be linked because its alias is registered to more
/// than one entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousMention {
    /// The alias as written in the mention
    pub alias: String,
    /// Canonical names of every entity the alias could mean
    pub candidates: Vec<String>,
}

impl AmbiguousMention {
    /// One-line explanation, so every caller words this identically.
    pub fn describe(&self) -> String {
        format!(
            "'{}' matches multiple entities ({}); skipping link for this mention.",
            self.alias,
            self.candidates.join(", ")
        )
    }
}

/// Outcome of resolving a single `[bracket]` mention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution {
    /// Resolved (or created) to this entity ID
    Entity(i64),
    /// Left unlinked because the alias is ambiguous
    Ambiguous(AmbiguousMention),
}

/// Resolve `name` to an entity ID, creating one if nothing matches, and *report*
/// an ambiguous alias rather than printing about it.
///
/// [`resolve_or_create_entity`] is the plain-CLI wrapper that prints the warning
/// to stderr. Callers that own the terminal — the interactive composer holds it in
/// raw mode on the alternate screen — must use this instead: writing to stderr
/// there lands on the same tty as the rendered frame and smears it, and because
/// ratatui only redraws diffs the smear persists for the rest of the session.
pub fn resolve_entity(conn: &Connection, name: &str) -> Result<Resolution, ThoughtError> {
    match EntitiesRepository::resolve(conn, name) {
        Ok(Some(entity)) => {
            let id = match entity.id {
                Some(id) => id,
                // A resolved entity always has an ID; recreate rather than panic.
                None => EntitiesRepository::find_or_create(conn, &entity)?,
            };
            Ok(Resolution::Entity(id))
        }
        Ok(None) => {
            let entity = Entity::new(name.to_string());
            Ok(Resolution::Entity(EntitiesRepository::find_or_create(conn, &entity)?))
        }
        Err(ThoughtError::AmbiguousAlias { alias, entities }) => Ok(Resolution::Ambiguous(AmbiguousMention {
            alias,
            candidates: entities,
        })),
        Err(other) => Err(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::connection::get_memory_connection;
    use crate::storage::entity_aliases_repository::EntityAliasesRepository;
    use crate::storage::migrations::run_migrations;

    #[test]
    fn test_resolve_or_create_creates_new_entity_when_unmatched() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let id = resolve_or_create_entity(&conn, "NewEntity").unwrap();
        assert!(id.is_some());

        let entity = EntitiesRepository::find_by_name(&conn, "newentity").unwrap();
        assert!(entity.is_some());
    }

    #[test]
    fn test_resolve_or_create_resolves_existing_canonical_name() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah_id = EntitiesRepository::find_or_create(&conn, &Entity::new("Sarah".to_string())).unwrap();

        let resolved_id = resolve_or_create_entity(&conn, "sarah").unwrap().unwrap();
        assert_eq!(resolved_id, sarah_id);
    }

    #[test]
    fn test_resolve_or_create_resolves_alias_without_creating_new_entity() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah_id = EntitiesRepository::find_or_create(&conn, &Entity::new("Sarah".to_string())).unwrap();
        EntityAliasesRepository::add_alias(&conn, sarah_id, "sar").unwrap();

        let resolved_id = resolve_or_create_entity(&conn, "sar").unwrap().unwrap();
        assert_eq!(resolved_id, sarah_id);

        // No new literal "sar" entity should have been created.
        assert!(EntitiesRepository::find_by_name(&conn, "sar").unwrap().is_none());
    }

    #[test]
    fn test_resolve_or_create_ambiguous_alias_skips_link() {
        let conn = get_memory_connection().unwrap();
        run_migrations(&conn).unwrap();

        let sarah_id = EntitiesRepository::find_or_create(&conn, &Entity::new("Sarah".to_string())).unwrap();
        let john_id = EntitiesRepository::find_or_create(&conn, &Entity::new("John".to_string())).unwrap();
        EntityAliasesRepository::add_alias(&conn, sarah_id, "boss").unwrap();
        EntityAliasesRepository::add_alias(&conn, john_id, "boss").unwrap();

        let result = resolve_or_create_entity(&conn, "boss").unwrap();
        assert!(result.is_none());

        // No new literal "boss" entity should have been created either.
        assert!(EntitiesRepository::find_by_name(&conn, "boss").unwrap().is_none());
    }
}
