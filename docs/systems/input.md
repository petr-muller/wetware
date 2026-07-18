# Input

## Purpose

Launches the user's external text editor (`$EDITOR`) for interactive content entry, used by `wet edit
--editor` and interactive entity description editing.

## Questions this doc answers

- How is the editor chosen?
- What happens if the editor exits abnormally?

## Scope

`src/input/editor.rs` — `launch_editor`.

## Non-scope

What the caller does with the returned content (see [`cli.md`](cli.md) for `edit`/`entity edit`).

## Key concepts

None beyond the editor-resolution fallback chain below.

## How the system works

`launch_editor(initial_content: Option<&str>) -> Result<String, ThoughtError>`:

1. Writes `initial_content` (or empty) to a `tempfile::NamedTempFile`.
2. Resolves which editor to run: `$EDITOR` env var if set, else a fallback chain tried via `which`:
   `vim` → `nano` → `vi`. If none of those are found on `PATH`, falls back to the hardcoded string `"vi"`
   regardless.
3. Runs the editor as a child process against the temp file path and waits for it to exit.
4. A non-zero exit status returns `ThoughtError::EditorLaunchFailed(editor)`.
5. On success, reads back the (possibly edited) file content and returns it.

## Important flows

Used by [`flows/edit-thought.md`](../flows/edit-thought.md) (`wet edit --editor`) and by
`cli/entity_edit.rs`'s interactive description-editing path (no dedicated flow doc — single-step, see
[`cli.md`](cli.md)).

## Data and state

A temporary file, cleaned up automatically when `NamedTempFile` drops.

## Interfaces and entry points

`launch_editor(initial_content: Option<&str>) -> Result<String, ThoughtError>`.

## Dependencies

`errors` (`ThoughtError`), `tempfile`, `which` (implicitly, for fallback resolution), external editor
binaries on `PATH`.

## Downstream effects

Callers (`cli/edit.rs`, `cli/entity_edit.rs`) treat an `EditorLaunchFailed` as a non-fatal warning in some
paths — see their "Common pitfalls" notes in `cli.md`.

## Invariants and assumptions

The temp file always exists and is readable after the editor process exits successfully — no defensive
re-check beyond the exit status.

## Error handling

Editor launch/exit failure → `ThoughtError::EditorLaunchFailed`. Temp file I/O failure →
`ThoughtError::FileError`.

## Security and privacy notes

The temp file briefly holds thought/entity-description content in a world-readable-by-default temp
directory (subject to OS temp dir permissions); no additional protection is applied.

## Observability and debugging

If the editor never appears, check `$EDITOR` and confirm `vim`/`nano`/`vi` are on `PATH`.

## Testing notes

Hard to unit test the actual editor launch in CI; behavior is typically verified manually or by testing
the fallback-resolution logic in isolation.

## Common pitfalls

- The final fallback is a hardcoded `"vi"` even if `which` didn't find `vi` on `PATH` either — in that
  case the process spawn itself will fail with an OS-level error rather than a friendlier message.

## Source map

- [`src/input/editor.rs`](../../src/input/editor.rs)

## Related docs

- [`flows/edit-thought.md`](../flows/edit-thought.md)
- [`cli.md`](cli.md)
