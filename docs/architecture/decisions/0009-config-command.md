---
status: Accepted
date: "2026-05-13"
---

# Config Command

## Context

Users wanted to control application behavior (starting with default thought sort order) persistently,
without a dedicated flag on every command. A `git config`-style interface was a familiar model to follow.

## Decision

Add `wet config <key> [value]`, using dotted `section.item` key notation (e.g. `thoughts.order`), mimicking
`git config`. Config get/set uses explicit, hardcoded match arms per known key rather than dynamic TOML
path manipulation — the config schema is small and typed, and this keeps validation tight (unknown keys
error immediately; invalid values list the valid options) at the cost of needing a code change to add a
new key. The first (and currently only) supported key, `thoughts.order`, accepts `ascending`/`descending`
and defaults to `descending` (newest first); it maps to a `[thoughts]` TOML section with an `order` field,
with serde defaults ensuring old config files missing that section still parse. As part of this feature,
`SortOrder` moved from `src/tui/state.rs` to `src/models/sort_order.rs`, since it's a domain concept shared
by config, CLI, and TUI, not TUI-specific state. The `config` command operates purely on the config file —
it never touches the database.

## Consequences

- Adding a new config key requires editing `Config::get_value`/`set_value`'s match arms directly — a
  known, accepted limitation (see [`../../systems/config.md`](../../systems/config.md#common-pitfalls)),
  traded for tight validation and type safety over a fully generic key-path system.
- Moving `SortOrder` into `models/` unblocked config, CLI (`wet thoughts`), and the TUI from all sharing
  one type instead of duplicating or awkwardly cross-depending.
- `wet thoughts` and the TUI's initial launch order both now respect `thoughts.order`; the TUI's `s` key
  still allows interactive override within a session.

## Alternatives considered

- **Dynamic TOML key-path manipulation** (generic `set(path, value)` over the parsed document) — rejected:
  loses type safety and makes validating allowed values/keys harder for what is, so far, a very small
  config surface.
- **Per-setting CLI flags instead of a generic config command** (e.g. `wet thoughts --default-order`) —
  rejected: doesn't scale as more settings are added, and doesn't match the familiar `git config` mental
  model.

## Related code

- [`src/config.rs`](../../../src/config.rs)
- [`src/cli/config.rs`](../../../src/cli/config.rs)
- [`src/models/sort_order.rs`](../../../src/models/sort_order.rs)

## Related docs

- [`../../systems/config.md`](../../systems/config.md)
- [`../../systems/models.md`](../../systems/models.md)
