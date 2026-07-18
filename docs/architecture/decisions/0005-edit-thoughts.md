---
status: Accepted
date: "2026-04-10"
---

# Edit Thoughts

## Context

Thoughts were immutable after creation. Users needed a way to correct a thought's text or date without
deleting and re-adding it (which would lose the original ID and reset entity-link history).

## Decision

Add `wet edit <ID> ["new text"] [--date YYYY-MM-DD] [--editor]`, parallel in structure to `wet add`. At
least one of text, `--date`, or `--editor` must be given; inline text and `--editor` are mutually
exclusive. When text changes, entity associations are fully recalculated: existing links are removed and
the new content is re-parsed and re-linked, reusing the same entity-parsing and find-or-create
infrastructure as `wet add`. All steps (content/date update, unlink, relink) run inside one SQLite
transaction, so a failure at any point leaves the thought completely unchanged. No schema change was
needed — existing `id`/`content`/`created_at` columns suffice. A new `ThoughtError::ThoughtNotFound(i64)`
variant was added for edits (and later deletes) targeting a nonexistent ID.

## Consequences

- Editing is atomic: there's no observable partial state where content changed but entity links didn't
  (or vice versa).
- Reusing `wet add`'s entity infrastructure (parser, find-or-create) kept the implementation small and
  behaviorally consistent between add and edit.
- An editor crash or abnormal exit during `--editor` makes no changes and only prints a warning, rather
  than surfacing as an error — a deliberate "don't lose data on editor failure" choice, at the cost of
  editor failures being less visible than other error paths. See
  [`../../systems/cli.md`](../../systems/cli.md#common-pitfalls).

## Alternatives considered

- **Delete + re-add for corrections** — rejected as the existing workaround: loses the original ID and
  requires re-linking from scratch, with no atomicity guarantee across the two operations.
- **Diffing old vs. new entity references instead of unlink-all + relink-all** — not pursued: the
  unlink/relink approach is simpler and the extra churn (re-creating links that didn't actually change) is
  cheap at this data scale.

## Related code

- [`src/cli/edit.rs`](../../../src/cli/edit.rs)

## Related docs

- [`../../flows/edit-thought.md`](../../flows/edit-thought.md)
- [`../../systems/cli.md`](../../systems/cli.md)
