# 007: Data Directory

## Summary

Wetware uses a dedicated data directory following XDG conventions (`~/.local/share/wetware/`) to store its database and configuration. The directory is auto-created on first run. Debug builds require an explicit override, ensuring development and testing never touch the user's production data.

## Requirements

- Default data directory is `$XDG_DATA_HOME/wetware/` (typically `~/.local/share/wetware/`)
- On first run, the directory is created automatically if it doesn't exist
- Directory contains a TOML config file (`config.toml`) and a default database (`default.db`)
- `WETWARE_DATA_DIR` env var overrides the entire data directory location
- `WETWARE_DB` env var overrides just the database path (takes precedence over data dir for db)
- Debug builds (`cargo build`/`cargo run`/`cargo test`) panic if no explicit data dir override is set — they never resolve the real XDG path
- Only release builds resolve the real XDG data directory
- Config file uses TOML format with serde for (de)serialization
- Config file is created with sensible defaults on first run if missing

## Decisions

- **XDG via `dirs` crate**: `dirs::data_dir()` for cross-platform data directory resolution. Only called in release builds.
- **Compile-time guard**: `#[cfg(debug_assertions)]` prevents debug builds from ever computing the real data dir. Developers must set `WETWARE_DATA_DIR` explicitly.
- **TOML config**: idiomatic for Rust, human-readable, uses `toml` + `serde` crates.
- **Single default database**: `default.db` in the data directory. Structure allows future multi-database support.
- **Env var precedence**: `WETWARE_DB` > `<data_dir>/default.db`; `WETWARE_DATA_DIR` > XDG default.

## Directory Structure

```
~/.local/share/wetware/
├── config.toml          # User preferences and settings
└── default.db           # Default SQLite database
```

## CLI Interface

No new commands. Existing commands transparently use the data directory:

```
wet add "thought"            # Uses ~/.local/share/wetware/default.db
WETWARE_DATA_DIR=/tmp/w wet add "thought"   # Uses /tmp/w/default.db
WETWARE_DB=/tmp/t.db wet add "thought"      # Uses /tmp/t.db directly
```

## Edge Cases

- `dirs::data_dir()` returns `None` (e.g., `$HOME` not set) → clear error message
- Data directory exists but is not writable → standard IO error propagation
- Config file exists but is malformed → error with details, not silent fallback
- `WETWARE_DB` set alongside `WETWARE_DATA_DIR` → `WETWARE_DB` wins for database path
- Debug build without `WETWARE_DATA_DIR` → panic with message explaining the requirement
