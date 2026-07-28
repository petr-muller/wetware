# Entity Merge

## Purpose

Fold a duplicate entity into another one: every reference to the merged-away entity — ID-keyed links,
literal text references in thought content and entity descriptions — ends up pointing at the survivor,
and the merged entity's description, aliases and relations are preserved on it.

## Trigger

`wet entity merge <entity_name> --into <target>`

## Participants

- `cli/entity_merge.rs`
- `services/entity_parser.rs` (`redirect_entity_references`)
- `storage/entities_repository.rs`
- `storage/thoughts_repository.rs`
- `storage/entity_aliases_repository.rs`
- `storage/entity_relations_repository.rs`

## Step-by-step flow

1. Resolve both names via `EntitiesRepository::resolve`, so either side may itself be an existing alias
   (see [`entity-alias-resolution.md`](entity-alias-resolution.md)) — errors `EntityNotFound` for
   whichever one is missing.
2. Reject a merge where both names resolve to the same entity — errors `SelfMerge`.
3. Reject a target whose name contains `(` or `)` — errors `InvalidInput`. The target name is
   interpolated into `[display](target)` markup, and `ENTITY_PATTERN`'s target group is `[^()]+`, so such
   a name would produce text that reads back as a *bare* reference to the display text. See
   "Invariants and assumptions" below.
4. In a single `conn.transaction()`, in this order:
   - Rewrite every entity's description via `entity_parser::redirect_entity_references`. Any entity's
     description can mention the source, not just the two being merged.
   - Rewrite the content of every thought reachable from the source the same way.
   - Append the source's (now-rewritten) description to the target's, separated by a blank line; if the
     target had none, the source's becomes the target's.
   - Register the source's aliases on the target, skipping any alias that names the target itself.
   - Re-attach the source's parent and child edges to the target, dropping any edge whose other end *is*
     the target (would be a self-relation) or that would close a cycle once collapsed.
   - Re-point `thought_entities` rows onto the target (`repoint_thought_links`).
   - Delete the source entity row.

## Data and state changes

- `thoughts.content` and `entities.description` are rewritten wherever they literally contained
  `[source]` or `[alias](source)`. The **display text is preserved**: `[Alice]` becomes `[Alice](Bob)`,
  so past thoughts still read as originally written while resolving to the survivor. A reference whose
  display text already names the target collapses to the bare form rather than `[Bob](Bob)`.
- `thought_entities` rows for the source are re-created against the target (`INSERT OR IGNORE`, so a
  thought that referenced both entities keeps a single link) and the source's own rows then disappear via
  `ON DELETE CASCADE`.
- The target's `description` may gain the source's description appended to it.
- `entity_aliases` and `entity_relations` rows are copied onto the target before the delete cascades the
  source's away.
- The source's `entities` row is deleted.

## Success behavior

The source entity no longer exists; every thought that referenced it is linked to the target and reads as
it originally did; the target carries the union of both entities' descriptions, aliases and relations.

## Failure behavior

- Either entity not found → `ThoughtError::EntityNotFound`, no changes made (plus the same stderr hint
  block `wet entity rename` prints).
- Both names resolve to the same entity → `ThoughtError::SelfMerge`, no changes made.
- Target name contains `(` or `)` → `ThoughtError::InvalidInput` naming the entity and suggesting
  `wet entity rename`, no changes made.
- Either name is an ambiguous alias → `ThoughtError::AmbiguousAlias`, no changes made.

Everything after resolution runs inside one transaction, so any storage error rolls the whole merge back.

## External dependencies

None.

## Invariants and assumptions

- Unlike [`entity-rename.md`](entity-rename.md), which leaves `thought_entities` alone because links are
  ID-keyed and the ID doesn't change, a merge **must** re-point those links — the source's ID is about to
  disappear.
- Text must be rewritten before the source row is deleted, while the source's name still resolves.
- The merged-away name is deliberately **not** registered as an alias of the survivor. Since `wet add`
  and `wet edit` re-extract entities by name, typing `[Alice]` in a *future* thought creates a fresh
  `Alice` entity. Run `wet entity alias <target> --alias <old name>` after the merge to have the old name
  keep resolving.
- An entity name containing `(` or `)` can be a merge *source* but never a merge *target*: group 1 of
  `ENTITY_PATTERN` permits parentheses (so `wet add "[Alice (HR)]"` creates such an entity through
  ordinary use) but the target group does not. Rewriting into such a target would desynchronize stored
  text from the ID-keyed link table, and because `wet edit` re-extracts entities by name, the next edit
  of an affected thought would silently recreate the merged-away entity and undo the merge for it.
- Merging is irreversible and, like `wet delete`, runs without confirmation (see
  [`../architecture/decisions/0008-delete-thoughts.md`](../architecture/decisions/0008-delete-thoughts.md)).

## Security and privacy notes

Not applicable beyond general local-data sensitivity noted in [`../systems/storage.md`](../systems/storage.md).

## Observability and debugging

The command reports how many thought links it moved, how many thoughts and descriptions it rewrote, and
how many relation edges it dropped. Links moved exceeds thoughts rewritten whenever a thought referenced
the source through a registered Known Alias — no stored text names the source in that case, so the link
moves without a rewrite. A rewrite count of zero where you expected more usually means the text used
bracket formatting that `ENTITY_PATTERN` doesn't match (see
[`../systems/services.md`](../systems/services.md)).

## Testing notes

Cover: bare and aliased references redirected with wording preserved; a thought referencing both entities
keeping one link and one listing; descriptions of unrelated entities rewritten; description append and
adopt-when-target-has-none; alias transfer, including skipping an alias that names the target; parent and
child relation transfer; relation that would become a self-relation or a cycle dropped *and counted*;
links moved counted separately from text rewrites when an alias-only reference is involved; both sides
resolved through aliases; self-merge rejected; parenthesized target rejected while a parenthesized source
still merges cleanly; missing entity leaving the database untouched.

## Source map

- [`src/cli/entity_merge.rs`](../../src/cli/entity_merge.rs)
- [`src/services/entity_parser.rs`](../../src/services/entity_parser.rs)
- [`src/storage/entities_repository.rs`](../../src/storage/entities_repository.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)

## Related docs

- [`../systems/cli.md`](../systems/cli.md), [`../systems/services.md`](../systems/services.md),
  [`../systems/storage.md`](../systems/storage.md)
- [`entity-rename.md`](entity-rename.md), [`entity-alias-resolution.md`](entity-alias-resolution.md),
  [`edit-thought.md`](edit-thought.md)
- [`../architecture/decisions/0014-entity-merge.md`](../architecture/decisions/0014-entity-merge.md)
