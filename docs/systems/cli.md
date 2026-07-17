# CLI

## Purpose

The `wet` command-line surface: argument parsing (clap) and the per-command implementations that tie
together `models`, `services`, and `storage`.

## Questions this doc answers

- What subcommands exist and what do they do?
- What's the common pattern every command follows?
- Which commands touch more than one repository/table?

## Scope

`src/cli/mod.rs` (clap definitions) and each `src/cli/*.rs` command implementation.

## Non-scope

The TUI, launched by `wet tui` but documented separately (see [`tui.md`](tui.md)); the underlying
repository/service logic each command calls (see [`storage.md`](storage.md), [`services.md`](services.md)).

## Key concepts

Global `--color` flag (`ColorMode`, see [`services.md`](services.md)). Subcommand list below.

## How the system works

`Cli { color: ColorMode, command: Commands }`, parsed via `Cli::parse()` in `main.rs`
([`entry-points.md`](entry-points.md)).

| Subcommand | Args | Purpose | Source |
|---|---|---|---|
| `add` | `content`, `--date` | Add a new thought | `cli/add.rs` |
| `thoughts` | `--on <entity>` | List thoughts, optionally filtered | `cli/thoughts.rs` |
| `edit` | `id`, `content?`, `--date`, `--editor` (conflicts w/ content) | Edit a thought | `cli/edit.rs` |
| `delete` | `id` | Delete a thought | `cli/delete.rs` |
| `config` | `key`, `value?` | Get/set config values | `cli/config.rs` |
| `tui` | — | Launch the interactive TUI | `cli/tui.rs` |
| `entities` | — | List all entities | `cli/entities.rs` |
| `entity edit` | `entity_name`, `--description` \| `--description-file` \| interactive | Set/remove a description | `cli/entity_edit.rs` |
| `entity rename` | `entity_name`, `new_name` | Rename an entity, rewriting references | `cli/entity_rename.rs` |
| `entity show` | `entity_name` | Show description + 5 latest linked thoughts | `cli/entity_show.rs` |

**Common pattern**: every command's `execute(...)` opens its own `Connection`, calls
`storage::run_migrations`, performs its repository/service calls, and prints output — usually through
`EntityStyler` for entity-aware rendering. There is no shared session or long-lived connection across
commands (see [`storage.md`](storage.md)).

**Notable per-command detail:**

- `add.rs` — parses `--date` (`NaiveDate` "%Y-%m-%d" → midnight UTC) if given, saves the thought, then
  extracts entities via `entity_parser::extract_unique_entities` and `find_or_create`s + links each.
- `thoughts.rs` — repository always returns ascending order; the command reverses the list if
  `SortOrder::Descending`.
- `edit.rs` — see [`flows/edit-thought.md`](../flows/edit-thought.md). If `--editor` is used and the
  editor process exits abnormally, this prints a warning and returns `Ok(())` — **no error is propagated
  and no changes are made** (see Common Pitfalls).
- `delete.rs` — fetches the thought first (to print a confirmation with its date/content) before
  deleting.
- `entities.rs` — if terminal width ≥ 60 chars, shows a description preview per entity via
  `description_formatter::generate_preview` alongside the name.
- `entity_edit.rs` — three mutually exclusive input modes: inline `--description`, `--description-file`,
  or interactive editor (none of the flags given). Trimmed-empty input means "remove the description".
  Verifies the entity exists first, with a hint to create it via `wet add` if not. Auto-creates any
  entities newly referenced *within* the description text itself.
- `entity_rename.rs` — see [`flows/entity-rename.md`](../flows/entity-rename.md). Validates `new_name` is
  non-empty and contains none of `[`, `]`, `(`, `)` (these would break entity-reference parsing, see
  [`services.md`](services.md)).
- `entity_show.rs` — prints canonical name, styled description (if any), and up to 5 most recent linked
  thoughts (`LATEST_THOUGHTS_LIMIT = 5`).
- `tui.rs` — loads all thoughts+entities, calls `ratatui::init()`, builds `tui::App`, runs the event loop,
  then **always** calls `ratatui::restore()` after, even if the loop returned an error (terminal state is
  restored before the error propagates further).

## Important flows

- [`flows/edit-thought.md`](../flows/edit-thought.md)
- [`flows/entity-rename.md`](../flows/entity-rename.md)
- [`flows/delete-thought.md`](../flows/delete-thought.md)

## Data and state

No persistent CLI-layer state — every invocation is stateless beyond what it reads from/writes to
storage.

## Interfaces and entry points

`main.rs`'s dispatch match on `Commands`, and each `cli::<name>::execute(...)` function.

## Dependencies

`errors`, `models`, `services::{color_mode, description_formatter, entity_parser, entity_styler}`,
`storage::*`, `input::editor`, `config`, `tui::App` (only `cli/tui.rs`).

## Downstream effects

Every command writes to SQLite via `storage/`; several also mutate the `entities` table implicitly
through `find_or_create` even when the user only intended to add a thought.

## Invariants and assumptions

Commands that mutate more than one table (`edit`, `entity rename`) do so inside a single
`conn.transaction()` for atomicity.

## Error handling

Each command surfaces `ThoughtError` variants specific to its operation (see [`errors.md`](errors.md));
`main.rs` is the single top-level handler.

## Security and privacy notes

Not applicable beyond what's noted in [`storage.md`](storage.md) about local data sensitivity.

## Observability and debugging

Run any command with `WETWARE_DATA_DIR`/`WETWARE_DB` pointed at a scratch location to inspect behavior
without touching real data (required in debug builds regardless, see [`storage.md`](storage.md)).

## Testing notes

Command `execute()` functions are typically tested directly against an in-memory or temp-file database,
bypassing actual CLI argument parsing.

## Common pitfalls

- `edit --editor` swallowing an abnormal editor exit as a silent `Ok(())` (no changes made, no error)
  differs from every other failure path in this system, which propagates a `ThoughtError`. Don't assume
  `edit`'s success return means content changed.
- `entity_edit.rs`'s three input modes are mutually exclusive at the clap level (`--description` and
  `--description-file` conflict) — passing neither is what triggers the interactive editor, which is easy
  to miss when reading the command signature alone.
- `entity_rename.rs`'s new-name character restriction (`[`, `]`, `(`, `)` disallowed) exists specifically
  because those characters are entity-reference syntax; an unrestricted name could produce unparseable
  references.

## Source map

- [`src/main.rs`](../../src/main.rs)
- [`src/cli/mod.rs`](../../src/cli/mod.rs)
- [`src/cli/add.rs`](../../src/cli/add.rs)
- [`src/cli/thoughts.rs`](../../src/cli/thoughts.rs)
- [`src/cli/edit.rs`](../../src/cli/edit.rs)
- [`src/cli/delete.rs`](../../src/cli/delete.rs)
- [`src/cli/config.rs`](../../src/cli/config.rs)
- [`src/cli/tui.rs`](../../src/cli/tui.rs)
- [`src/cli/entities.rs`](../../src/cli/entities.rs)
- [`src/cli/entity_edit.rs`](../../src/cli/entity_edit.rs)
- [`src/cli/entity_rename.rs`](../../src/cli/entity_rename.rs)
- [`src/cli/entity_show.rs`](../../src/cli/entity_show.rs)

## Related docs

- [`storage.md`](storage.md), [`services.md`](services.md), [`tui.md`](tui.md), [`input.md`](input.md)
- [`flows/edit-thought.md`](../flows/edit-thought.md), [`flows/entity-rename.md`](../flows/entity-rename.md),
  [`flows/delete-thought.md`](../flows/delete-thought.md)
