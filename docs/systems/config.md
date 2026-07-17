# Config

## Purpose

TOML-backed user configuration, stored at `<data_dir>/config.toml`, currently controlling one setting:
default Thought sort order.

## Questions this doc answers

- What config keys exist?
- How is the config file created and loaded?
- How do I add a new config key?

## Scope

`src/config.rs` — `Config`/`ThoughtsConfig` structs, load/save/get/set logic.

## Non-scope

The `wet config` CLI command itself (thin wrapper, see [`cli.md`](cli.md)); data directory resolution
(see [`storage.md`](storage.md)).

## Key concepts

- **Config** — `{ version: u32, thoughts: ThoughtsConfig }`. `version` defaults to `1` and is currently
  unused (reserved for future migrations).
- **ThoughtsConfig** — `{ order: SortOrder }`, defaults to `SortOrder::Descending`.

## How the system works

- `load_config(data_dir)` reads and parses `config.toml`; returns `Config::default()` if the file doesn't
  exist yet. A malformed file is a parse error.
- `save_config(data_dir, config)` serializes and writes the file.
- `ensure_config(data_dir)` loads the config, writing a default file if none exists, and returns the
  resulting config. Called once at CLI startup (`main.rs`).
- `Config::get_value(key)` / `Config::set_value(key, value)` are the only supported read/write paths for
  individual keys. Today the only recognized key is `"thoughts.order"`; any other key returns
  `ThoughtError::InvalidInput`.

Fields use `#[serde(default = ...)]`, so a config file written before a new field existed still parses —
missing fields fall back to their defaults rather than erroring.

## Important flows

Not applicable — config reads/writes are single-step, not cross-system.

## Data and state

`config.toml` in the data directory (see [`storage.md`](storage.md) for data directory resolution).

## Interfaces and entry points

`load_config`, `save_config`, `ensure_config`, `Config::get_value`, `Config::set_value`. Exposed to users
via `wet config <key> [value]` ([`cli.md`](cli.md)).

## Dependencies

`errors` (`ThoughtError`), `models` (`SortOrder`), `serde`, `toml`.

## Downstream effects

`cli/thoughts.rs` reads `thoughts.order` (via the loaded `Config`) to decide default display order.

## Invariants and assumptions

`Config::get_value`/`set_value` must be extended together whenever a new key is added — there's no
generic reflection-based key access.

## Error handling

Unknown keys → `ThoughtError::InvalidInput`. Malformed TOML → parse error surfaced through
`ThoughtError`.

## Security and privacy notes

Config is a local, unencrypted TOML file with no secrets expected in it.

## Observability and debugging

Inspect `<data_dir>/config.toml` directly; `wet config <key>` prints the current value.

## Testing notes

Round-trip tests cover `load_config`/`save_config`, default-file creation via `ensure_config`, and
`get_value`/`set_value` for the supported key plus the unknown-key error path.

## Common pitfalls

- The config surface is single-key today (`thoughts.order`). Adding a second key means updating
  `get_value`/`set_value`'s match arms, not just the struct — it's easy to add a struct field and forget
  the CLI-facing accessor.

## Source map

- [`src/config.rs`](../../src/config.rs)
- [`src/cli/config.rs`](../../src/cli/config.rs)

## Related docs

- [`cli.md`](cli.md) — `wet config` command.
- [`storage.md`](storage.md) — data directory resolution.
- [Glossary: Sort Order](../glossary.md#sort-order)
