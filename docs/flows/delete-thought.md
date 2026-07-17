# Delete Thought

## Purpose

Remove a thought, from either the CLI (direct, no confirmation) or the TUI (confirmation step required).

## Trigger

`wet delete <id>` (CLI), or pressing `x` then `y`/`Y` in the TUI's `Normal`/`ConfirmDelete` modes.

## Participants

- `cli/delete.rs` (CLI path)
- `tui/state.rs` (`Mode::ConfirmDelete`), `tui/mod.rs` (`App::delete_selected_thought`), `tui/input.rs`
  (`handle_confirm_delete_mode`) (TUI path)
- `storage/thoughts_repository.rs` (both paths)

## Step-by-step flow

**CLI path**: fetch the thought first (to print its date/content in the confirmation message), then call
`ThoughtsRepository::delete` — errors `ThoughtNotFound` if the ID doesn't exist.

**TUI path**: pressing `x` in `Normal` mode enters `Mode::ConfirmDelete { thought_index }`. `y`/`Y` calls
`App::delete_selected_thought`, which opens its own DB connection, runs migrations, deletes the row via
`ThoughtsRepository::delete`, then removes the thought from the in-memory `thoughts` list (no re-query),
and resets `mode` to `Normal`. `n`/`N`/`Esc` cancels back to `Normal` without any DB call.

## Data and state changes

The `thoughts` row is removed; `ON DELETE CASCADE` on `thought_entities` automatically removes any link
rows for that thought — no explicit unlink step needed in either path.

## Success behavior

The thought no longer exists in the database. In the TUI, it also immediately disappears from the
in-memory `displayed_thoughts` view.

## Failure behavior

- CLI: thought ID doesn't exist → `ThoughtError::ThoughtNotFound`, printed and process exits 1.
- TUI: a delete error is swallowed — `handle_confirm_delete_mode` falls back to `Normal` mode silently on
  `Err`, with no error shown to the user. See [`../systems/tui.md`](../systems/tui.md#error-handling).

## External dependencies

None.

## Invariants and assumptions

The TUI's in-memory thought list is mutated directly rather than re-queried after a delete — if the
database were modified concurrently by another process, the TUI's view could drift from the database
state until the next full reload (session restart).

## Security and privacy notes

Not applicable beyond general local-data sensitivity noted in [`../systems/storage.md`](../systems/storage.md).

## Observability and debugging

If a TUI delete silently appears to fail (thought stays visible), it likely hit the swallowed-error path —
check the underlying `ThoughtsRepository::delete` call directly against the DB to confirm.

## Testing notes

Cover: CLI delete of an existing/nonexistent ID; TUI confirm-delete happy path; TUI cancel (`n`/`Esc`)
leaves the thought untouched.

## Source map

- [`src/cli/delete.rs`](../../src/cli/delete.rs)
- [`src/tui/state.rs`](../../src/tui/state.rs)
- [`src/tui/mod.rs`](../../src/tui/mod.rs)
- [`src/tui/input.rs`](../../src/tui/input.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)

## Related docs

- [`../systems/cli.md`](../systems/cli.md), [`../systems/tui.md`](../systems/tui.md),
  [`../systems/storage.md`](../systems/storage.md)
- [`../architecture/decisions/0008-delete-thoughts.md`](../architecture/decisions/0008-delete-thoughts.md)
