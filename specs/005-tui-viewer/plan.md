# Implementation Plan: Interactive TUI Thought Viewer

**Branch**: `005-tui-viewer` | **Date**: 2026-03-09 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/005-tui-viewer/spec.md`

## Summary

Add a read-only interactive TUI viewer for thoughts, launched via a new `wet tui` subcommand. The TUI displays thoughts in a scrollable list with entity references highlighted in color, supports toggling sort order (ascending/descending by date), provides a fuzzy entity picker overlay for filtering, and shows entity descriptions in centered modal popups. Built with ratatui + crossterm using The Elm Architecture (TEA) pattern for state management.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: ratatui (TUI framework), crossterm (terminal backend/events), tui-input (text input widget), nucleo-matcher (fuzzy matching)
**Storage**: SQLite via rusqlite (existing, read-only access)
**Testing**: cargo nextest run (unit + integration + contract tests)
**Target Platform**: Linux/macOS terminals (crossterm cross-platform)
**Project Type**: Single project (existing Rust binary)
**Performance Goals**: Responsive UI with databases of 1,000+ thoughts, instant sort/filter updates
**Constraints**: Read-only viewer, no database modifications, single-threaded synchronous event loop
**Scale/Scope**: 4 new source files (tui module), ~1,000-1,500 lines of new code

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with Wetware Constitution (`.specify/memory/constitution.md`):

**I. Test-First Development (90%+ Coverage)**
- [x] Test strategy defined (unit, integration, contract)
  - Unit tests: App state transitions, fuzzy matching logic, sort toggling, filter application
  - Integration tests: TUI rendering output verification with test data
  - Contract tests: `wet tui` subcommand launches and exits cleanly
  - Note: Full TUI rendering tests are limited (ratatui's `TestBackend` enables buffer assertions)
- [x] 90%+ coverage target achievable for this feature
  - State management and data transformation logic is fully testable
  - Rendering logic testable via ratatui's `TestBackend`
- [x] TDD approach planned for new functionality

**II. Layer Separation**
- [x] Changes respect CLI / Domain / Persistence / Input layer boundaries
  - New `src/tui/` module handles TUI concerns (rendering, input, state)
  - TUI reads from persistence layer via existing repository interfaces
  - Domain models (Thought, Entity) used as-is, no modifications
  - Entity parsing/styling reused from existing `services/` module
- [x] No direct dependencies from Domain to CLI or Persistence implementation
- [x] Persistence abstraction maintained (interface-based)

**III. Strong Typing & Error Handling**
- [x] Error types identified for new functionality
  - New `ThoughtError::TuiError(String)` variant for terminal initialization/restoration failures
  - All ratatui/crossterm errors wrapped in existing error type
- [x] Result/Option types used (no panics in business logic)
- [x] Error context propagation planned

**IV. Observability & Documentation**
- [x] Public API documentation plan (rustdoc)
  - All public TUI module types and functions documented
- [x] Logging strategy for significant operations
  - TUI startup/shutdown logged
- [x] Architecture decision rationale documented
  - TEA pattern choice documented in research.md

**V. Simplicity & YAGNI**
- [x] Solution is simplest that meets current requirements
  - Synchronous event loop (no async runtime needed)
  - Single App struct holds all state
  - Reuses existing entity parsing and color assignment
- [x] No premature optimization or abstraction
- [x] Complexity justified if present

## Project Structure

### Documentation (this feature)

```text
specs/005-tui-viewer/
├── plan.md              # This file
├── research.md          # Phase 0: Technology decisions
├── data-model.md        # Phase 1: TUI state model
├── quickstart.md        # Phase 1: Development setup
├── contracts/           # Phase 1: Module interfaces
│   └── tui-module.md    # TUI module public API contract
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs           # Updated: add Tui variant to Commands enum
│   └── tui.rs           # NEW: TUI subcommand handler (launches TUI)
├── tui/
│   ├── mod.rs           # NEW: TUI module root, App struct, TEA loop
│   ├── state.rs         # NEW: Application state types (Mode, SortOrder, etc.)
│   ├── ui.rs            # NEW: Rendering functions (draw thought list, overlays)
│   └── input.rs         # NEW: Key event handling, message dispatch
├── models/              # Unchanged
├── services/
│   └── entity_styler.rs # Minor: may need to expose color assignment for ratatui Styles
├── storage/             # Unchanged (read-only access)
└── errors/
    └── thought_error.rs # Updated: add TuiError variant

tests/
├── contract/
│   └── test_tui_command.rs    # NEW: TUI subcommand contract tests
└── integration/
    └── test_tui_state.rs      # NEW: TUI state management tests
```

**Structure Decision**: The TUI gets its own top-level module (`src/tui/`) separate from `src/cli/` because it has distinct concerns (rendering, event loop, UI state) that don't fit the existing CLI handler pattern. The CLI layer gets a thin `tui.rs` handler that initializes the terminal and delegates to the TUI module. This maintains layer separation: CLI dispatches, TUI manages the interactive session, persistence provides data.
