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