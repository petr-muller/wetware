# CLAUDE.md

## Project Overview

Wetware is a Rust project that helps track "thoughts" - brief snippets of information associated with dates and entities. It provides a CLI binary called `wet` for interacting with these thoughts.

## Documentation

Detailed system behavior, cross-system flows, and architectural rationale live in [`docs/`](docs/), routed
through [`AGENTS.md`](AGENTS.md). Read the relevant `docs/systems/`/`docs/flows/` doc before changing a
system's behavior, and update it in the same change if behavior, interfaces, data, or invariants change.
See [`docs/README.md`](docs/README.md) for the full map and [`docs/architecture/decisions/`](docs/architecture/decisions/)
for why past decisions were made.

## Build Commands

- Build: `cargo build`
- Run: `cargo run -- <args>`
- Test all: `cargo nextest run`
- Test single: `cargo nextest run <test_name>`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Architecture

Four-layer architecture with strict separation:

```
src/
├── cli/          CLI commands (clap subcommands: add, edit, thoughts, entities, entity_edit, tui)
├── models/       Domain types (Thought, Entity) — must not depend on CLI or persistence
├── services/     Business logic (entity_parser, entity_styler, description_formatter, color_mode)
├── storage/      SQLite persistence (thoughts_repository, entities_repository, migrations, connection)
├── input/        User input handling (editor integration)
├── tui/          Interactive TUI viewer (state, ui, input — built on ratatui)
├── errors/       Error types (ThoughtError)
├── lib.rs        Library root
└── main.rs       Binary entry point
```

Domain model must not depend on CLI or persistence. Persistence is behind repository interfaces. CLI delegates to services and repositories.

## Code Standards

- Rust 2024 edition
- Conventional commits: `type(scope): description` (feat, fix, docs, test, refactor, chore)
- 90%+ test coverage target; TDD for new features (tests fail before implementation)
- Result/Option over panics; domain-specific error types via thiserror
- Functions < 50 lines preferred; YAGNI — no speculative abstractions
- rustdoc on public APIs
- `cargo clippy` must be clean, `cargo fmt` must pass

## Dependencies

- clap 4.6 (CLI), rusqlite 0.39 (SQLite), chrono 0.4 (dates), regex 1.12 (entity parsing)
- thiserror 2.0 (error types), owo-colors 4 (terminal styling), terminal_size 0.4 (TTY detection)
- tempfile 3.27 (editor temp files), dirs 6.0 (XDG data directory), serde 1 + toml 0.8 (config)
- ratatui 0.30 + crossterm (TUI framework), tui-input 0.15 (text input), nucleo-matcher 0.3 (fuzzy search)

## Data Directory

See [`docs/systems/storage.md`](docs/systems/storage.md) for full detail on connection handling, migrations, and schema.

- Data directory at `~/.local/share/wetware/` (XDG data dir), overridable via `WETWARE_DATA_DIR` env var
- Contains `config.toml` (TOML config) and `default.db` (SQLite database)
- Database path overridable via `WETWARE_DB` env var (takes precedence over data dir for db)
- **Debug builds panic if `WETWARE_DATA_DIR` is not set** — prevents dev/test from touching production data
- Tables: `thoughts`, `entities` (with `description` column), `thought_entities` (junction)
- Migrations in `src/storage/migrations/`

## Documentation Workflow

**Workflow for new features:**
1. Read the relevant `docs/systems/*.md` and `docs/flows/*.md` before designing the change.
2. Use plan mode to design the implementation; for significant technical decisions, add an ADR under
   `docs/architecture/decisions/` (see [`docs/architecture/README.md`](docs/architecture/README.md)).
3. Implement the feature.
4. Update the affected `docs/systems/`/`docs/flows/` docs in the same change.

**Workflow for smaller behavior changes:**
1. Update the relevant `docs/systems/`/`docs/flows/` doc.
2. Change the implementation to match.

Use [`docs/templates/`](docs/templates/) for the expected structure of a new system, flow, or ADR doc.

## Technology Preferences

When choosing new dependencies or approaches:
- Prefer lightweight, focused crates over heavyweight frameworks
- Prefer std library solutions when available (e.g. std::sync::LazyLock over lazy_static)
- Keep the single-binary CLI design; avoid runtime service dependencies
- SQLite for persistence; no plans to change storage backend
