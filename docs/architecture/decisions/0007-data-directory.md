---
status: Accepted
date: "2026-04-10"
---

# Data Directory

## Context

wetware's database and config needed a standard, predictable location, and development/test runs needed a
hard guarantee they could never accidentally read or write a real user's production data.

## Decision

Use the OS-conventional data directory (`dirs::data_dir()/wetware`, typically
`~/.local/share/wetware/` on Linux/XDG) as the default location for `config.toml` and `default.db`,
auto-created on first run. `WETWARE_DATA_DIR` overrides the entire directory; `WETWARE_DB` overrides just
the database path and takes precedence over the data-directory-derived default. Critically, **debug
builds** (`#[cfg(debug_assertions)]`) never call `dirs::data_dir()` at all — they panic immediately if no
explicit `WETWARE_DATA_DIR` override is set. Only release builds resolve the real XDG path. Config uses
TOML via `serde` + `toml`, created with defaults on first run if missing.

## Consequences

- `cargo run`/`cargo test` can never silently touch a real user's data directory — any accidental omission
  of `WETWARE_DATA_DIR` in a dev/test context fails loudly (a panic) rather than quietly writing to
  `~/.local/share/wetware/`.
- The directory structure (`config.toml` + a single `default.db`) leaves room for future multi-database
  support without a breaking change, though none is planned.
- Every debug/test invocation must set `WETWARE_DATA_DIR` explicitly — a small ergonomic cost, accepted
  deliberately as a safety-over-convenience tradeoff (see [`CLAUDE.md`](../../../CLAUDE.md)).

## Alternatives considered

- **Same resolution logic in debug and release builds, relying on developer discipline** — rejected: a
  forgotten override in a debug/test run would silently read/write real user data, which is the exact
  failure this decision exists to prevent.
- **A dedicated `--data-dir` CLI flag instead of/alongside the env var** — not pursued; the env var
  approach was judged sufficient and keeps every command's argument surface uncluttered.

## Related code

- [`src/storage/data_dir.rs`](../../../src/storage/data_dir.rs)
- [`src/config.rs`](../../../src/config.rs)

## Related docs

- [`../../systems/storage.md`](../../systems/storage.md)
- [`../../systems/config.md`](../../systems/config.md)
- [`../../systems/entry-points.md`](../../systems/entry-points.md)
