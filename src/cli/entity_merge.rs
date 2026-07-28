/// Entity merge command implementation
use crate::errors::ThoughtError;
use crate::services::entity_parser::redirect_entity_references;
use crate::storage::connection::get_connection;
use crate::storage::entities_repository::EntitiesRepository;
use crate::storage::entity_aliases_repository::EntityAliasesRepository;
use crate::storage::entity_relations_repository::EntityRelationsRepository;
use crate::storage::migrations::run_migrations;
use crate::storage::thoughts_repository::ThoughtsRepository;
use rusqlite::{Connection, Transaction};
use std::path::Path;

/// What a merge changed, for reporting back to the user.
#[derive(Debug, PartialEq, Eq)]
pub struct MergeSummary {
    /// Canonical name of the entity that was merged away
    pub source: String,
    /// Canonical name of the surviving entity
    pub target: String,
    /// Thoughts whose stored content was rewritten
    pub thoughts_updated: usize,
    /// Entity descriptions whose stored text was rewritten
    pub descriptions_updated: usize,
    /// Thought links moved onto the target that it didn't already have. Larger than
    /// `thoughts_updated` when a thought referenced the source through a registered
    /// alias, since no text mentions the source's name in that case.
    pub links_moved: usize,
    /// Relation edges discarded because collapsing the two entities would have turned
    /// them into self-relations or cycles
    pub relations_dropped: usize,
}

/// Execute the entity merge command
///
/// Folds `entity_name` into `into`: every reference to the merged-away entity is
/// redirected at the surviving one - `thought_entities` links are re-pointed, and
/// literal references in thought content and entity descriptions are rewritten to
/// keep their original wording while targeting the survivor (`[Alice]` becomes
/// `[Alice](Bob)`). The merged entity's description, known aliases and parent/child
/// relations transfer to the survivor before its row is deleted. The whole operation
/// is atomic.
///
/// The merged-away name is deliberately *not* registered as an alias of the survivor;
/// run `wet entity alias <survivor> --alias <old name>` afterwards to have future
/// mentions of the old name keep resolving.
///
/// # Arguments
/// * `entity_name` - Entity to merge away (case-insensitive, may be an alias)
/// * `into` - Entity to merge into (case-insensitive, may be an alias)
/// * `db_path` - Database path
///
/// # Returns
/// * `Ok(())` - Entities successfully merged
/// * `Err(ThoughtError)` - Either entity not found, both names resolve to the same
///   entity, the target's name contains parentheses (it can't be a reference target),
///   or a storage error
pub fn execute(entity_name: &str, into: &str, db_path: &Path) -> Result<(), ThoughtError> {
    let mut conn = get_connection(db_path)?;
    run_migrations(&conn)?;

    let summary = match merge(&mut conn, entity_name, into) {
        Ok(summary) => summary,
        Err(ThoughtError::EntityNotFound(name)) => {
            eprintln!("Error: Entity '{}' not found", name);
            eprintln!();
            eprintln!("Hint: Create the entity first by referencing it in a thought:");
            eprintln!("  wet add \"Learning about [{}] today\"", name);
            return Err(ThoughtError::EntityNotFound(name));
        }
        Err(e) => return Err(e),
    };

    println!(
        "Merged entity '{}' into '{}'. Moved {} thought(s); rewrote {} thought(s) and {} description(s).",
        summary.source, summary.target, summary.links_moved, summary.thoughts_updated, summary.descriptions_updated
    );

    if summary.relations_dropped > 0 {
        println!(
            "Dropped {} relation(s) that would have become self-relations or cycles.",
            summary.relations_dropped
        );
    }

    Ok(())
}

/// Merge `entity_name` into `into` within a single transaction.
///
/// The storage-level half of [`execute`], separated so it can be driven directly
/// against a connection (integration tests) without CLI output or a database file.
/// Both names are resolved alias-aware; either one missing yields `EntityNotFound`.
pub fn merge(conn: &mut Connection, entity_name: &str, into: &str) -> Result<MergeSummary, ThoughtError> {
    let Some(source) = EntitiesRepository::resolve(conn, entity_name)? else {
        return Err(ThoughtError::EntityNotFound(entity_name.to_string()));
    };

    let Some(target) = EntitiesRepository::resolve(conn, into)? else {
        return Err(ThoughtError::EntityNotFound(into.to_string()));
    };

    if source.id == target.id {
        return Err(ThoughtError::SelfMerge(source.canonical_name));
    }

    // The target's name is interpolated into `[display](target)` markup, and
    // `ENTITY_PATTERN`'s target group is `[^()]+` - a target name containing parentheses
    // would produce text the parser reads back as a *bare* reference to the display text,
    // silently desynchronizing stored content from the link table. Such names exist:
    // group 1 permits parentheses, so `wet add "[Alice (HR)]"` creates one. Only the
    // target side is affected; parentheses in the source's name rewrite fine.
    // `entity_rename.rs` guards the same character class for the same reason.
    if target.canonical_name.contains(['(', ')']) {
        return Err(ThoughtError::InvalidInput(format!(
            "Cannot merge into '{}': entity names containing '(' or ')' cannot be used as reference \
             targets. Rename it first: wet entity rename \"{}\" \"<new name>\"",
            target.canonical_name, target.canonical_name
        )));
    }

    let source_id = source.id.unwrap();
    let target_id = target.id.unwrap();

    let tx = conn.transaction()?;

    // Rewrite stored text before touching the entity rows, while the source's name
    // still resolves. Descriptions of *every* entity can mention the source, not just
    // the two being merged.
    let mut descriptions_updated = 0;
    for other in EntitiesRepository::list_all(&tx)? {
        if let Some(desc) = &other.description {
            let redirected = redirect_entity_references(desc, &source.name, &target.canonical_name);
            if redirected != *desc {
                EntitiesRepository::update_description(&tx, &other.name, Some(redirected))?;
                descriptions_updated += 1;
            }
        }
    }

    // `list_by_entity` walks relations, so this is a superset of the thoughts actually
    // linked to the source - harmless, since the redirect is a no-op on text that
    // doesn't mention it.
    let mut thoughts_updated = 0;
    for thought in ThoughtsRepository::list_by_entity(&tx, &source.name)? {
        let redirected = redirect_entity_references(&thought.content, &source.name, &target.canonical_name);
        if redirected != thought.content {
            ThoughtsRepository::update(&tx, thought.id.unwrap(), &redirected, thought.created_at)?;
            thoughts_updated += 1;
        }
    }

    merge_description(&tx, &source.name, &target.name)?;
    transfer_aliases(&tx, &source, &target)?;
    let relations_dropped = transfer_relations(&tx, source_id, target_id)?;

    let links_moved = EntitiesRepository::repoint_thought_links(&tx, source_id, target_id)?;
    EntitiesRepository::delete(&tx, source_id)?;

    tx.commit()?;

    Ok(MergeSummary {
        source: source.canonical_name,
        target: target.canonical_name,
        thoughts_updated,
        descriptions_updated,
        links_moved,
        relations_dropped,
    })
}

/// Append the source's description to the target's, keeping both.
///
/// Re-reads both rows so the already-redirected description text is what gets merged.
fn merge_description(tx: &Transaction, source_name: &str, target_name: &str) -> Result<(), ThoughtError> {
    let (Some(source), Some(target)) = (
        EntitiesRepository::find_by_name(tx, source_name)?,
        EntitiesRepository::find_by_name(tx, target_name)?,
    ) else {
        return Ok(());
    };

    let Some(source_desc) = source.description.as_ref().filter(|d| !d.trim().is_empty()) else {
        return Ok(());
    };

    let merged = match target.description.as_ref().filter(|d| !d.trim().is_empty()) {
        Some(target_desc) => format!("{}\n\n{}", target_desc.trim_end(), source_desc.trim_start()),
        None => source_desc.clone(),
    };

    EntitiesRepository::update_description(tx, &target.name, Some(merged))
}

/// Register the source's aliases on the target.
///
/// Skips any alias that names the target itself - an entity's own name registered as
/// its own alias is noise, and `resolve` would never reach it anyway (canonical names
/// always win). `add_alias` is idempotent, so aliases both entities already share are
/// a no-op.
fn transfer_aliases(
    tx: &Transaction,
    source: &crate::models::entity::Entity,
    target: &crate::models::entity::Entity,
) -> Result<(), ThoughtError> {
    for alias in EntityAliasesRepository::list_for_entity(tx, source.id.unwrap())? {
        if alias.to_lowercase() == target.name {
            continue;
        }
        EntityAliasesRepository::add_alias(tx, target.id.unwrap(), &alias)?;
    }

    Ok(())
}

/// Re-attach the source's parent and child edges to the target.
///
/// Edges whose other end *is* the target would become self-relations, and edges that
/// would close a cycle once collapsed onto the target are dropped rather than erroring:
/// a merge shouldn't fail because of a graph shape the user never asked for. Returns how
/// many edges were dropped, so the caller can report the loss rather than making it
/// silent as well as irreversible.
fn transfer_relations(tx: &Transaction, source_id: i64, target_id: i64) -> Result<usize, ThoughtError> {
    let mut dropped = 0;

    for parent in EntityRelationsRepository::list_parents(tx, source_id)? {
        let parent_id = parent.id.unwrap();
        if parent_id == target_id || EntityRelationsRepository::would_create_cycle(tx, target_id, parent_id)? {
            dropped += 1;
            continue;
        }
        EntityRelationsRepository::add_relation(tx, target_id, parent_id)?;
    }

    for child in EntityRelationsRepository::list_children(tx, source_id)? {
        let child_id = child.id.unwrap();
        if child_id == target_id || EntityRelationsRepository::would_create_cycle(tx, child_id, target_id)? {
            dropped += 1;
            continue;
        }
        EntityRelationsRepository::add_relation(tx, child_id, target_id)?;
    }

    Ok(dropped)
}
