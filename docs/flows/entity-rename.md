# Entity Rename

## Purpose

Rename an entity while keeping every existing literal text reference to it (in thought content and other
entities' descriptions) pointing at the new name, without touching the underlying ID-keyed links.

## Trigger

`wet entity rename <entity_name> <new_name>`

## Participants

- `cli/entity_rename.rs`
- `services/entity_parser.rs` (`rewrite_entity_references`)
- `storage/entities_repository.rs`
- `storage/thoughts_repository.rs`

## Step-by-step flow

1. Validate `new_name`: non-empty, and contains none of `[`, `]`, `(`, `)` (these would break entity
   reference syntax — see [`../systems/services.md`](../systems/services.md)).
2. Confirm the entity exists (`EntitiesRepository::resolve`, so `entity_name` may itself be an existing
   alias — see [`entity-alias-resolution.md`](entity-alias-resolution.md)) — errors `EntityNotFound` if
   not.
3. Confirm `new_name` doesn't collide with a *different* existing entity's canonical name — errors
   `EntityAlreadyExists` if so. The collision check compares entity IDs, so renaming an entity to itself,
   or only changing its casing, is allowed.
4. Confirm `new_name` isn't already registered as an alias of a *different* entity — errors
   `RenameCollidesWithAlias` if so (renaming to the same entity's own alias is fine). Without this guard,
   `resolve()`'s canonical-wins-first rule would let the rename silently and permanently shadow that other
   entity's alias.
5. In a single `conn.transaction()`:
   - Rewrite every other entity's description text via `entity_parser::rewrite_entity_references`.
   - Rewrite every linked thought's content the same way.
   - Rename the entity row itself (`name` + `canonical_name`).

## Data and state changes

Text content of `entities.description` and `thoughts.content` may be rewritten wherever they literally
contained `[old_name]` or `[alias](old_name)`. `thought_entities` **link rows are untouched** — they're
keyed by entity ID, which doesn't change on rename.

## Success behavior

Every literal reference to the old name (bare or aliased-target) across all thought content and entity
descriptions now points at the new name; the entity's own name/canonical_name are updated; all existing
thought↔entity links remain intact.

## Failure behavior

- Entity not found → `ThoughtError::EntityNotFound`, no changes made.
- New name collides with a different entity's canonical name → `ThoughtError::EntityAlreadyExists`, no
  changes made.
- New name is already registered as a different entity's alias → `ThoughtError::RenameCollidesWithAlias`,
  no changes made.
- New name contains reserved characters → `ThoughtError::InvalidInput`, no changes made, no transaction
  opened.

## External dependencies

None.

## Invariants and assumptions

- `thought_entities` is ID-keyed, so renaming never requires touching the link table — only the text and
  the entity's own name columns change.
- The rewrite is purely textual (regex-based substring rewrite via `rewrite_entity_references`), not a
  re-parse-and-relink — it assumes the stored text's entity references were well-formed to begin with.

## Security and privacy notes

Not applicable beyond general local-data sensitivity noted in [`../systems/storage.md`](../systems/storage.md).

## Observability and debugging

If a reference doesn't get rewritten after a rename, check whether it used non-standard bracket
formatting that the `ENTITY_PATTERN` regex wouldn't have matched in the first place (see
[`../systems/services.md`](../systems/services.md)).

## Testing notes

Cover: rename with existing bare references, rename with aliased references (alias text preserved, target
rewritten), rename to an existing different entity's name (collision error), rename to a different
entity's registered alias (`RenameCollidesWithAlias`), rename to the entity's own alias (allowed),
case-only rename (allowed), rename looked up by alias, rename of a nonexistent entity.

## Source map

- [`src/cli/entity_rename.rs`](../../src/cli/entity_rename.rs)
- [`src/services/entity_parser.rs`](../../src/services/entity_parser.rs)
- [`src/storage/entities_repository.rs`](../../src/storage/entities_repository.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)

## Related docs

- [`../systems/cli.md`](../systems/cli.md), [`../systems/services.md`](../systems/services.md),
  [`../systems/storage.md`](../systems/storage.md)
- [`entity-alias-resolution.md`](entity-alias-resolution.md)
- [`../architecture/decisions/0010-entity-rename.md`](../architecture/decisions/0010-entity-rename.md)
- [`../architecture/decisions/0013-entity-aliases.md`](../architecture/decisions/0013-entity-aliases.md)
