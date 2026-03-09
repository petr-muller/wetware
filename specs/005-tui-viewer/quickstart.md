# Quickstart: Interactive TUI Thought Viewer

**Feature**: 005-tui-viewer
**Date**: 2026-03-09

## Prerequisites

- Rust 2024 edition toolchain
- Existing wetware database with some thoughts and entities (for manual testing)

## New Dependencies

Add to `Cargo.toml`:

```toml
ratatui = "0.29"
crossterm = "0.29"
tui-input = "0.11"
nucleo-matcher = "0.3"
```

Check crates.io for exact latest versions at implementation time.

## Build & Run

```bash
# Build
cargo build

# Run the TUI viewer
cargo run -- tui

# Run with a custom database
WETWARE_DB=/path/to/thoughts.db cargo run -- tui

# Run all tests
cargo nextest run

# Run only TUI-related tests
cargo nextest run tui

# Lint
cargo clippy

# Format
cargo fmt
```

## Development Workflow

### Recommended implementation order

1. **Scaffold**: Add `Tui` variant to CLI commands, create `src/tui/` module with empty `App::run()`
2. **Basic list**: Render thoughts in a scrollable list with `ListState`
3. **Entity highlighting**: Parse entity references and render with colored `Span`s
4. **Sort toggle**: Add `SortOrder` state and `s` key handler
5. **Status bar**: Show sort order, active filter, and key hints
6. **Entity picker**: Implement fuzzy picker overlay with `nucleo-matcher` and `tui-input`
7. **Entity description popup**: Implement modal popup for entity descriptions
8. **Polish**: Empty states, terminal resize handling, edge cases

### Testing approach

- **Unit tests**: Test `App` state transitions (sort toggle, filter application, mode changes) without rendering
- **Rendering tests**: Use ratatui's `TestBackend` to assert buffer contents for key UI states
- **Contract tests**: Run `wet tui` binary and verify it starts/exits cleanly (limited scope due to interactive nature)

### Key files to read first

1. `src/cli/mod.rs` — understand command dispatch pattern
2. `src/services/entity_parser.rs` — entity reference regex
3. `src/services/entity_styler.rs` — color assignment logic
4. `src/storage/thoughts_repository.rs` — `list_all()` and `list_by_entity()`
5. `src/storage/entities_repository.rs` — `list_all()` and `find_by_name()`
