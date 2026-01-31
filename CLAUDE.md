# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview
Wetware is a Rust project that helps track "thoughts" - brief snippets of information associated with dates and entities. It provides a CLI binary called `wet` for interacting with these thoughts.

## Build Commands
- Build: `cargo build`
- Run: `cargo run -- <args>`
- Test all: `cargo nextest run`
- Test single: `cargo nextest run <test_name>`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style Guidelines
- Use Rust 2024 edition conventions
- Follow conventional commits (feat, fix, docs, etc.)
- Organize code into modules by responsibility (CLI, domain, persistence, input)
- Keep functions small and focused
- Use strong typing with appropriate error handling
- Prefer Result/Option over exceptions
- Target 90%+ test coverage
- Document public APIs with rustdoc

## Architecture
- CLI layer (with clap)
- Domain model (thoughts, entities)
- Persistence layer (using SQLite)
- User input handling layer

## Active Technologies
- Rust 2024 edition + clap 4.5, rusqlite 0.32, regex 1.11, chrono 0.4, thiserror 2.0 (002-styled-entity-output)
- SQLite (rusqlite) (002-styled-entity-output)
- Rust 2024 edition (matching Cargo.toml) (003-entity-reference-aliases)
- SQLite database (thoughts table stores content as-is, entity extraction happens at runtime) (003-entity-reference-aliases)

## Recent Changes
- 003-entity-reference-aliases: Added support for aliased entity references using markdown-like syntax `[alias](entity)`. Migrated from lazy_static to std::sync::LazyLock. Pattern: `r"\[([^\[\]]+)\](?:\(([^\(\)]+)\))?"`
- 002-styled-entity-output: Added Rust 2024 edition + clap 4.5, rusqlite 0.32, regex 1.11, chrono 0.4, thiserror 2.0
