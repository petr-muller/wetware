---
status: Accepted
date: "2026-07-12"
---

# Entity Rename

## Context

Entity references are stored as literal bracket text embedded directly in thought content and entity
descriptions (see [`0001-networked-notes-schema.md`](0001-networked-notes-schema.md) and
[`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md)) — not as pure foreign-key
placeholders resolved at render time. Renaming an entity therefore raises a real question: what happens to
every stored occurrence of its old name?

## Decision

`wet entity rename <old-name> <new-name>` (case-insensitive match on `old-name`) rewrites every stored
literal occurrence of the old name to the new name, in addition to updating the entity's own row:
bare `[OldName]` becomes `[NewName]`; aliased `[Alias](OldName)` has only the target rewritten, the alias
display text is left untouched. Every entity's description that references the old name is rewritten
using the same rules (scanned across *all* entities, since any entity's description may reference any
other entity) — but thought-content rewriting is scoped to only the thoughts already linked to the entity
via `thought_entities`, not every thought in the database. The entire operation (row update + all content
rewrites) runs in one transaction. Renaming onto a name already used by a *different* entity fails with an
error — no merge behavior. The collision check compares entity IDs rather than the `name` column directly,
so a same-name or casing-only rename (`Sarah` → `SARAH`) succeeds despite the column's
`UNIQUE COLLATE NOCASE` constraint. `new-name` is rejected up front if it contains `[`, `]`, `(`, or `)`,
since those characters are interpolated directly into rewritten bracket syntax and would corrupt it.

`thought_entities` link rows are never touched by rename — they're keyed by `entity_id`, not name, so they
remain valid automatically. The content rewrite exists purely to keep displayed/stored text in sync with
the new name, and to prevent duplicate-entity creation the next time an affected thought is edited (`wet
edit` re-extracts entities by name from content on every content change, so stale name text would
otherwise silently re-create the old entity).

## Consequences

- Text always reflects the current entity name everywhere it's stored — a simple, consistent mental model
  for the user.
- No persisted alias table was introduced; renaming doesn't turn old occurrences into an implicit
  `[OldName](NewName)` alias, which would leave stale-looking display text around indefinitely.
- Rename is a potentially large write (every linked thought + every entity's description scanned/rewritten
  in one transaction) — acceptable at current data scale, but a heavier operation than most other single-row
  updates in the system.
- No entity-merge functionality exists — renaming onto an existing different entity is a hard error, by
  design; merging two entities' thought/description history is out of scope.

## Alternatives considered

- **Alias-preserving rewrite** (`[Sarah]` → `[Sarah](Sarah Smith)` instead of `[Sarah Smith]`) — considered
  and rejected: achieves the same "links stay valid" goal as the literal rewrite (since links are already
  ID-keyed and unaffected either way), but leaves stale-looking display text everywhere and adds
  complexity for no additional benefit.
- **Re-deriving `thought_entities` links from a fresh parse after rename** — not needed: links never
  become invalid in the first place, since they're ID-keyed, not name-keyed.
- **Entity merge on rename collision** — rejected as out of scope; a merge has different, more complex
  semantics (whose description wins? do both entities' thought histories combine?) that weren't required
  by this feature.

## Related code

- [`src/cli/entity_rename.rs`](../../../src/cli/entity_rename.rs)
- [`src/services/entity_parser.rs`](../../../src/services/entity_parser.rs) (`rewrite_entity_references`)
- [`src/storage/entities_repository.rs`](../../../src/storage/entities_repository.rs)

## Related docs

- [`../../flows/entity-rename.md`](../../flows/entity-rename.md)
- [`0001-networked-notes-schema.md`](0001-networked-notes-schema.md)
- [`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md)
