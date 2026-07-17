# Entry Points

## Purpose

The binary's startup sequence (`main.rs`) and the library root's module wiring (`lib.rs`).

## Questions this doc answers

- What happens between process start and a command actually running?
- Where do `WETWARE_DATA_DIR`/`WETWARE_DB` get resolved?
- What does the crate re-export at its root?

## Scope

`src/main.rs`, `src/lib.rs`.

## Non-scope

Individual command behavior (see [`cli.md`](cli.md)); data directory/db path resolution logic itself
(see [`storage.md`](storage.md)).

## Key concepts

None beyond the startup sequence below.

## How the system works

`lib.rs` (12 lines) declares the crate's top-level modules (`cli`, `config`, `errors`, `input`, `models`,
`services`, `storage`, `tui`) and re-exports `ThoughtError`, `Entity`, `SortOrder`, `Thought` at the crate
root for convenient use in tests and downstream code.

`main.rs` (79 lines) is the startup sequence, in order:

1. Parse CLI arguments into `Cli` via `Cli::parse()` ([`cli.md`](cli.md)).
2. Resolve the data directory: `WETWARE_DATA_DIR` env var if non-empty, else
   `storage::resolve_data_dir(None)` — which panics in debug builds without an override (see
   [`storage.md`](storage.md#invariants-and-assumptions)).
3. `storage::ensure_data_dir` creates the directory if missing; `config::ensure_config` loads or creates
   `config.toml`.
4. Resolve the database path: `WETWARE_DB` env var if set, else `storage::default_db_path_in(data_dir)`
   (`<data_dir>/default.db`).
5. Dispatch `cli.command` to the matching `cli::<name>::execute(...)` function.
6. On `Err`, print `Error: {e}` to stderr and exit with status 1.

## Important flows

This sequence runs before every command; see [`cli.md`](cli.md) for what happens after dispatch.

## Data and state

No persistent state of its own — this module wires together environment variables, the data directory,
and the database path for every other system.

## Interfaces and entry points

The `main()` function itself — the process entry point.

## Dependencies

`cli`, `config`, `storage` (`default_db_path_in`, `ensure_data_dir`, `resolve_data_dir`).

## Downstream effects

Every command depends on the data directory and db path resolved here being correct before it opens a
connection.

## Invariants and assumptions

`WETWARE_DATA_DIR` and `WETWARE_DB` are checked once, at startup, not re-read mid-command.

## Error handling

Any error from data-directory/config setup or command dispatch propagates up to the single top-level
`Err` handler in `main()`, which prints and exits 1 — there's no per-command custom error handling at
this layer.

## Security and privacy notes

Not applicable.

## Observability and debugging

Set `WETWARE_DATA_DIR`/`WETWARE_DB` explicitly when debugging to avoid touching real user data — required
outright in debug builds (see [`storage.md`](storage.md)).

## Testing notes

Integration-style tests typically set `WETWARE_DATA_DIR`/`WETWARE_DB` to a temp location and invoke
command `execute()` functions directly rather than spawning the compiled binary.

## Common pitfalls

- Debug builds panic without `WETWARE_DATA_DIR` set — this is deliberate (see
  [`storage.md`](storage.md#invariants-and-assumptions)), not a bug, but it surprises anyone running
  `cargo run` without reading `CLAUDE.md` first.

## Source map

- [`src/main.rs`](../../src/main.rs)
- [`src/lib.rs`](../../src/lib.rs)

## Related docs

- [`cli.md`](cli.md)
- [`storage.md`](storage.md)
- [`config.md`](config.md)
