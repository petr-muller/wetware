# 009: Config Command

## Summary

A `wet config` command for reading and setting configuration values using dotted `section.item` keys, mimicking `git config`. The first supported key is `thoughts.order`, which controls whether thoughts are listed newest-first or oldest-first in both CLI and TUI output.

## Requirements

- `wet config <key>` prints the current value of a configuration key
- `wet config <key> <value>` sets the value of a configuration key
- Keys use `section.item` dot notation (e.g. `thoughts.order`)
- Unknown keys produce an error
- Invalid values produce an error listing valid options
- `thoughts.order` accepts `ascending` (oldest first) or `descending` (newest first)
- `thoughts.order` defaults to `descending` when not explicitly set
- `wet thoughts` CLI output respects `thoughts.order`
- TUI launches with the sort order from `thoughts.order`; the `s` key still toggles interactively

## Decisions

- **Hardcoded key matching**: Config get/set uses explicit match arms per key rather than dynamic TOML manipulation. The config schema is small and typed; this keeps validation tight and avoids losing type safety.
- **Shared `SortOrder` type**: The `SortOrder` enum moves from `src/tui/state.rs` to `src/models/sort_order.rs` since it is a domain concept used by config, CLI, and TUI.
- **TOML nested sections**: `thoughts.order` maps to a `[thoughts]` TOML section with an `order` field. Serde defaults ensure backward compatibility with existing config files that lack this section.
- **No database needed**: The `config` command operates only on the config file in the data directory, not on the database.

## CLI Interface

```
wet config thoughts.order            # Prints: descending (or ascending)
wet config thoughts.order ascending  # Sets order to ascending
wet config thoughts.order descending # Sets order to descending
wet config unknown.key               # Error: Unknown config key: unknown.key
wet config thoughts.order invalid    # Error: Invalid value 'invalid' for thoughts.order. Valid values: ascending, descending
```

## Edge Cases

- Config file has only `version = 1` (no `[thoughts]` section) → `thoughts.order` defaults to `descending`
- Config file does not exist → created with defaults on first access (existing `ensure_config` behavior)
- Multiple `wet config` set calls → last write wins, entire config is rewritten each time
