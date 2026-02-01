# Implementation Plan: Entity Descriptions

**Branch**: `001-entity-descriptions` | **Date**: 2026-02-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-entity-descriptions/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Add multi-paragraph description support to entities in the wetware CLI tool. Users will be able to add/edit descriptions via three methods (inline flag, interactive editor, file input) using the `wet entity edit` command. Descriptions support plain text and entity references (both `@entity` and `[alias](@entity)` syntax). The `entities` command will display ellipsized previews of descriptions alongside entity names, showing only the first paragraph to fit on single terminal lines. Terminal width is detected, with previews suppressed below 60 characters.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: clap 4.5 (CLI), rusqlite 0.32 (SQLite), regex 1.11 (entity parsing), owo-colors 4 (styling)
**Storage**: SQLite database (currently at `~/.local/share/wetware/thoughts.db` or `WETWARE_DB` env var)
**Testing**: cargo nextest (unit, integration, contract tests)
**Target Platform**: Linux CLI (with cross-platform Rust support)
**Project Type**: Single Rust binary CLI application
**Performance Goals**: Interactive CLI response times (<100ms for entity operations)
**Constraints**: Single-line output per entity (80-char minimum terminal width assumed), max 60-char threshold for preview suppression
**Scale/Scope**: Small-scale personal knowledge management (100s-1000s of entities typical)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with Wetware Constitution (`.specify/memory/constitution.md`):

**I. Test-First Development (90%+ Coverage)**
- [x] Test strategy defined (unit, integration, contract)
  - Unit tests: Entity parsing logic, description truncation, whitespace handling
  - Integration tests: Database operations, entity reference auto-creation
  - Contract tests: CLI command behavior (`wet entity edit`, updated `entities` output)
- [x] 90%+ coverage target achievable for this feature
  - All new code paths testable (description storage, retrieval, display, input methods)
- [x] TDD approach planned for new functionality
  - Write failing tests for each acceptance scenario before implementation

**II. Layer Separation**
- [x] Changes respect CLI / Domain / Persistence / Input layer boundaries
  - CLI: `wet entity edit` command in `src/cli/` (new file or extend existing)
  - Domain: Extend `Entity` struct in `src/models/entity.rs` with description field
  - Persistence: Update `EntitiesRepository` in `src/storage/entities_repository.rs`
  - Input: Interactive editor and file input handling in `src/input/`
- [x] No direct dependencies from Domain to CLI or Persistence implementation
  - Entity model remains agnostic to storage and CLI concerns
- [x] Persistence abstraction maintained (interface-based)
  - Repository pattern continues with new description methods

**III. Strong Typing & Error Handling**
- [x] Error types identified for new functionality
  - File I/O errors (reading description files)
  - Editor launch errors (interactive mode)
  - Database errors (description storage)
  - Validation errors (empty/whitespace-only descriptions)
- [x] Result/Option types used (no panics in business logic)
  - `Option<String>` for optional descriptions
  - `Result<(), ThoughtError>` for operations
- [x] Error context propagation planned
  - Use `context()` from anyhow/thiserror for informative error messages

**IV. Observability & Documentation**
- [x] Public API documentation plan (rustdoc)
  - Document new `wet entity edit` command
  - Document Entity struct changes
  - Document repository methods
- [x] Logging strategy for significant operations
  - Log description updates, file reads, editor launches
- [x] Architecture decision rationale documented
  - Decision to extend existing entity table vs. separate descriptions table
  - Choice of first-paragraph-only preview strategy
  - Terminal width threshold rationale

**V. Simplicity & YAGNI**
- [x] Solution is simplest that meets current requirements
  - Extend existing entity table with single TEXT column
  - Reuse existing entity reference parsing logic
  - Leverage existing terminal width detection if available
- [x] No premature optimization or abstraction
  - Direct string operations for preview generation
  - Simple regex split for paragraph detection
- [x] Complexity justified if present (see Complexity Tracking below)
  - No violations identified

## Project Structure

### Documentation (this feature)

```text
specs/001-entity-descriptions/
├── plan.md              # This file
├── research.md          # Phase 0 output (technical decisions)
├── data-model.md        # Phase 1 output (Entity schema extension)
├── quickstart.md        # Phase 1 output (usage examples)
├── checklists/
│   └── requirements.md  # Quality validation checklist
└── tasks.md             # Phase 2 output (NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── models/
│   ├── entity.rs        # MODIFY: Add description: Option<String> field
│   ├── thought.rs
│   └── mod.rs
├── services/
│   ├── entity_parser.rs # REUSE: Existing entity reference parsing
│   ├── entity_styler.rs # REUSE: For rendering references in descriptions
│   ├── description_formatter.rs  # NEW: Preview generation, paragraph extraction
│   └── mod.rs
├── storage/
│   ├── entities_repository.rs  # MODIFY: Add description CRUD methods
│   ├── migrations/
│   │   └── add_entity_descriptions_migration.rs  # NEW: Add description column
│   └── mod.rs
├── cli/
│   ├── entity_edit.rs   # NEW: `wet entity edit` command
│   ├── entities.rs      # MODIFY: Display description previews
│   └── mod.rs           # MODIFY: Register new command
├── input/
│   ├── editor.rs        # NEW or MODIFY: Interactive editor support
│   └── mod.rs
└── lib.rs

tests/
├── contract/
│   ├── test_entity_edit_command.rs  # NEW: CLI contract tests
│   └── test_entities_command.rs     # MODIFY: Preview display tests
├── integration/
│   ├── test_entity_descriptions.rs  # NEW: Description storage/retrieval
│   └── test_description_formatter.rs # NEW: Preview logic tests
└── unit/
    └── (inline module tests for description utilities)
```

**Structure Decision**: Single Rust project (default structure). All changes extend existing architecture layers without requiring new projects or major restructuring. The CLI, domain, persistence, and input layers remain cleanly separated.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations identified. All constitution principles are followed:
- Test-first approach with 90%+ coverage
- Clean layer separation maintained
- Strong typing with Result/Option
- Documentation and logging planned
- Simple, focused implementation without premature optimization
