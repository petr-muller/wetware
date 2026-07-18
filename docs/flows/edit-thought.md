# Edit Thought

## Purpose

Change an existing thought's content and/or date, atomically recalculating which entities it references.

## Trigger

`wet edit <id> [content] [--date YYYY-MM-DD] [--editor]` — at least one of `content`, `--date`, or
`--editor` must be given; `--editor` conflicts with inline `content`.

## Participants

- `cli/edit.rs`
- `input/editor.rs` (only if `--editor`)
- `services/entity_parser.rs` (`extract_unique_entities`)
- `storage/thoughts_repository.rs`
- `storage/entities_repository.rs`

## Step-by-step flow

1. Fetch the existing thought by ID (`ThoughtsRepository::get_by_id`) — errors `ThoughtNotFound` if
   missing.
2. Resolve the new content: the inline `content` argument, or the result of `input::launch_editor`
   pre-filled with the existing content if `--editor` was passed, or unchanged if only `--date` was given.
3. If content changed, re-extract entity references via `entity_parser::extract_unique_entities`.
4. In a single `conn.transaction()`:
   - Update the thought row (content and/or `created_at`) via `ThoughtsRepository::update`.
   - If content changed: `EntitiesRepository::unlink_all_from_thought`, then `find_or_create` +
     `link_to_thought` for each newly-extracted entity.

## Data and state changes

`thoughts.content`/`thoughts.created_at` updated; if content changed, `thought_entities` rows for this
thought are fully rebuilt (not diffed) inside the same transaction.

## Success behavior

The thought's stored content/date match the new values, and its entity links reflect exactly the entities
referenced in the new content (old links not present in the new content are removed; new ones are added).

## Failure behavior

- Thought ID doesn't exist → `ThoughtError::ThoughtNotFound(id)`, no changes made.
- `--editor` process exits abnormally → `cli/edit.rs` prints a warning and returns `Ok(())`, making
  **no changes at all** — this does not surface as an error to the caller, unlike every other failure path
  in this flow. See [`../systems/cli.md`](../systems/cli.md#common-pitfalls).

## External dependencies

The user's `$EDITOR` (or `vim`/`nano`/`vi` fallback), only when `--editor` is used.

## Invariants and assumptions

The content update and entity relink happen in one transaction — a partial state (updated content, stale
entity links, or vice versa) is never observable.

## Security and privacy notes

Not applicable beyond general local-data sensitivity noted in [`../systems/storage.md`](../systems/storage.md).

## Observability and debugging

If entity links look stale after an edit, check whether the edit actually returned `Ok` with real changes
vs. the silent no-op path (abnormal editor exit).

## Testing notes

Cover: content-only edit, date-only edit, both, `--editor` success, `--editor` abnormal exit
(verify no changes persisted), edit of a nonexistent ID.

## Source map

- [`src/cli/edit.rs`](../../src/cli/edit.rs)
- [`src/input/editor.rs`](../../src/input/editor.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)
- [`src/storage/entities_repository.rs`](../../src/storage/entities_repository.rs)
- [`src/services/entity_parser.rs`](../../src/services/entity_parser.rs)

## Related docs

- [`../systems/cli.md`](../systems/cli.md), [`../systems/storage.md`](../systems/storage.md),
  [`../systems/input.md`](../systems/input.md)
- [`../architecture/decisions/0005-edit-thoughts.md`](../architecture/decisions/0005-edit-thoughts.md)
