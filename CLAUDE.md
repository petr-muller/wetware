# CLAUDE.md

## Project Overview

Wetware is a Rust project that helps track "thoughts" - brief snippets of information associated with dates and entities. It provides a CLI binary called `wet` for interacting with these thoughts.

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
- tempfile 3.27 (editor temp files)
- ratatui 0.30 + crossterm (TUI framework), tui-input 0.15 (text input), nucleo-matcher 0.3 (fuzzy search)

## Database

- SQLite at `~/.local/share/wetware/thoughts.db` (or `WETWARE_DB` env var)
- Tables: `thoughts`, `entities` (with `description` column), `thought_entities` (junction)
- Migrations in `src/storage/migrations/`

## Feature Specs

Feature specifications live in `specs/` as single markdown files (one per feature).

**Workflow for new features:**
1. Use plan mode to create a new spec file in `specs/`
2. Implement the feature following the spec
3. Update existing specs if the new feature affects them

**Workflow for smaller behavior changes:**
1. Modify the relevant existing spec in `specs/`
2. Change the implementation to match

**Spec format:** Summary, Requirements, Decisions, CLI Interface, Edge Cases. Optional sections: Data Model, Status.

## Technology Preferences

When choosing new dependencies or approaches:
- Prefer lightweight, focused crates over heavyweight frameworks
- Prefer std library solutions when available (e.g. std::sync::LazyLock over lazy_static)
- Keep the single-binary CLI design; avoid runtime service dependencies
- SQLite for persistence; no plans to change storage backend
