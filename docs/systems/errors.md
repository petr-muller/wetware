# Errors

## Purpose

A single error type, `ThoughtError`, used as the `Result` error across the entire crate.

## Questions this doc answers

- What error variants exist and what do they mean?
- How do underlying errors (SQLite, I/O) become a `ThoughtError`?

## Scope

The `ThoughtError` enum and its `#[from]` conversions.

## Non-scope

Where specific variants are raised or handled (see the owning system's doc — `storage.md`, `cli.md`,
`input.md`, `tui.md`).

## Key concepts

`ThoughtError` (via `thiserror`) is the universal error type: essentially every fallible function in the
crate returns `Result<T, ThoughtError>`.

## How the system works

Variants, in `src/errors/thought_error.rs`:

| Variant | Meaning |
|---|---|
| `EmptyContent` | Thought content was empty or whitespace-only. |
| `ContentTooLong { max, actual }` | Thought content exceeded the 10,000-character limit. |
| `ParseError(String)` | A value (e.g. a date argument) failed to parse. |
| `StorageError(#[from] rusqlite::Error)` | Any underlying SQLite error, auto-converted. |
| `InvalidInput(String)` | A CLI argument or config value was invalid. |
| `EditorLaunchFailed(String)` | The external `$EDITOR` process exited non-zero or failed to launch. |
| `EntityNotFound(String)` | An operation referenced an entity that doesn't exist. |
| `EntityAlreadyExists(String)` | A rename target collides with a different existing entity. |
| `ThoughtNotFound(i64)` | An operation referenced a thought ID that doesn't exist. |
| `FileError(#[from] std::io::Error)` | Any underlying filesystem error, auto-converted. |
| `TuiError(String)` | A TUI-specific failure. |
| `RelationCycle { child, parent }` | Adding an entity relation would create a cycle in the parent/child graph. |
| `SelfRelation(String)` | An entity relation was attempted between an entity and itself. |

`#[from]` on `StorageError` and `FileError` means `rusqlite::Error`/`std::io::Error` convert automatically
via `?` — code that queries SQLite or touches the filesystem doesn't need explicit error mapping unless it
wants a more specific variant. `cli/edit.rs` shows that idiom: it matches on
`ThoughtError::StorageError(rusqlite::Error::QueryReturnedNoRows)` and remaps it to the more specific
`ThoughtError::ThoughtNotFound(id)`.

## Important flows

Not applicable.

## Data and state

None — `ThoughtError` is a stateless enum.

## Interfaces and entry points

The `ThoughtError` type itself, re-exported at the crate root (`wetware::ThoughtError`).

## Dependencies

`thiserror`, `rusqlite`, `std::io`.

## Downstream effects

Every system's public functions return `Result<_, ThoughtError>`. Adding a variant is low-risk; removing
or renaming one is a breaking change to every caller that matches on it.

## Invariants and assumptions

Fallible functions in this crate return `Result`, not panics, except for the deliberate debug-build panic
guard in [`storage.md`](storage.md#invariants-and-assumptions) around data-directory resolution.

## Error handling

`main.rs` is the top-level handler: on `Err`, it prints `Error: {e}` to stderr and exits with status 1.

## Security and privacy notes

Not applicable.

## Observability and debugging

Error `Display` messages (from `thiserror`) are what users see printed by `main.rs` — keep them
user-readable, not just diagnostic.

## Testing notes

Individual systems test that the correct variant is returned for their own failure paths (e.g.
`storage.md`'s repositories testing `ThoughtNotFound`/`EntityNotFound`).

## Common pitfalls

- Matching on `ThoughtError::StorageError(rusqlite::Error::...)` to special-case a SQLite error (as
  `cli/edit.rs` does) is a bit fragile — it couples CLI code to `rusqlite`'s error shape. This is the
  existing pattern; be aware of it rather than silently deviating from it in new code without reason.

## Source map

- [`src/errors/thought_error.rs`](../../src/errors/thought_error.rs)
- [`src/errors/mod.rs`](../../src/errors/mod.rs)

## Related docs

- [`storage.md`](storage.md), [`cli.md`](cli.md), [`input.md`](input.md), [`tui.md`](tui.md) — where
  specific variants originate.
