---
status: Accepted
date: "2026-05-12"
---

# Delete Thoughts

## Context

Neither the CLI nor the TUI could remove a thought once created — needed for correcting mistakes, without
adding the complexity of soft-delete or an undo system.

## Decision

Add `wet delete <ID>` (CLI), which deletes immediately with no confirmation prompt — deleting a single
thought by its explicit, already-known ID was judged low-risk enough not to need one. In the TUI, pressing
`x` (vim-style remove; `d` was already taken by entity detail) on a selected thought opens a confirmation
overlay (`y`/`Y` confirms, `n`/`N`/`Esc` cancels) — a destructive action in a browsing UI, where the target
isn't an explicit ID the user typed, warrants that extra step. Both paths rely on `ON DELETE CASCADE` on
`thought_entities` to clean up link rows automatically, rather than an explicit unlink step. The TUI stores
`db_path: PathBuf` on `App` and opens a connection on demand only for this mutation, consistent with the
rest of the TUI's read-only startup load.

## Consequences

- The CLI/TUI confirmation asymmetry is intentional, not an oversight — matched to how the target thought
  is selected in each context (typed ID vs. browsed selection).
- `thought_entities` cleanup is entirely delegated to the database's `ON DELETE CASCADE`, so the
  application code never needs to know which entities a thought referenced in order to delete it cleanly.
- This was the first TUI operation to write to the database, ending the TUI's purely read-only status from
  [`0006-tui-viewer.md`](0006-tui-viewer.md).

## Alternatives considered

- **CLI confirmation prompt for delete** — rejected: the user already had to know and type the exact ID,
  which is itself a deliberate act; an extra prompt would be friction without meaningfully preventing
  mistakes.
- **Soft delete (tombstone flag) instead of hard delete** — not pursued; no requirement for delete
  recovery was identified, and it would complicate every read query with a filter.

## Related code

- [`src/cli/delete.rs`](../../../src/cli/delete.rs)
- [`src/tui/state.rs`](../../../src/tui/state.rs)
- [`src/tui/mod.rs`](../../../src/tui/mod.rs)

## Related docs

- [`../../flows/delete-thought.md`](../../flows/delete-thought.md)
- [`../../systems/tui.md`](../../systems/tui.md)
