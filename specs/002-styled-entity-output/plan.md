# Implementation Plan: Styled Entity Output

**Branch**: `002-styled-entity-output` | **Date**: 2026-01-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-styled-entity-output/spec.md`

## Summary

Add styled (bold + colored) entity rendering to `wet thoughts` output. Entities should be displayed without their markup syntax (`[entity]` → `entity`), with consistent color assignment per entity throughout a single execution. Styling should auto-detect TTY and be controllable via CLI flags.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: clap 4.5, rusqlite 0.32, regex 1.11, chrono 0.4, thiserror 2.0
**Storage**: SQLite (rusqlite)
**Testing**: cargo nextest
**Target Platform**: Linux/macOS/Windows CLI
**Project Type**: Single CLI application
**Performance Goals**: Negligible overhead for styling (microseconds per output line)
**Constraints**: Must handle 100+ entities gracefully with color reuse
**Scale/Scope**: Single command output enhancement

**New Dependencies Required**:
- Terminal styling library (NEEDS RESEARCH: `colored`, `owo-colors`, or `crossterm`)
- TTY detection (NEEDS RESEARCH: built-in std, `atty`, or `is-terminal`)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with Wetware Constitution (`.specify/memory/constitution.md`):

**I. Test-First Development (90%+ Coverage)**
- [x] Test strategy defined (unit, integration, contract)
  - Unit tests: Color assignment logic, markup stripping function
  - Integration tests: Entity extraction with styling
  - Contract tests: CLI output format verification, `--color` flag behavior
- [x] 90%+ coverage target achievable for this feature
- [x] TDD approach planned for new functionality

**II. Layer Separation**
- [x] Changes respect CLI / Domain / Persistence / Input layer boundaries
  - Styling logic: New service module (`src/services/output_style.rs` or similar)
  - CLI changes: Add `--color` flag, call styling service
  - Domain unchanged: Entity/Thought models not modified
- [x] No direct dependencies from Domain to CLI or Persistence implementation
- [x] Persistence abstraction maintained (interface-based)

**III. Strong Typing & Error Handling**
- [x] Error types identified for new functionality
  - Styling failures are non-critical (fallback to plain text)
  - No new error types needed (styling is best-effort)
- [x] Result/Option types used (no panics in business logic)
- [x] Error context propagation planned

**IV. Observability & Documentation**
- [x] Public API documentation plan (rustdoc for styling module)
- [x] Logging strategy for significant operations (log color mode selection)
- [x] Architecture decision rationale documented (in research.md)

**V. Simplicity & YAGNI**
- [x] Solution is simplest that meets current requirements
- [x] No premature optimization or abstraction
- [x] Complexity justified if present - N/A (simple feature)

## Project Structure

### Documentation (this feature)

```text
specs/002-styled-entity-output/
├── plan.md              # This file
├── research.md          # Phase 0 output - dependency selection
├── data-model.md        # Phase 1 output - color assignment model
├── quickstart.md        # Phase 1 output - implementation guide
├── contracts/           # Phase 1 output - CLI interface contracts
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs           # CLI structure (add ColorChoice to global args)
│   └── thoughts.rs      # wet thoughts command (MODIFY: styled output)
├── services/
│   ├── mod.rs           # Services module (MODIFY: export new module)
│   ├── entity_parser.rs # Entity extraction (existing, may extend)
│   └── output_style.rs  # NEW: Styling service (color assignment, TTY detection)
└── models/              # Unchanged

tests/
├── contract/
│   └── test_thoughts_command.rs  # MODIFY: Add styled output tests
├── integration/
│   └── test_styled_output.rs     # NEW: Color consistency tests
└── unit/
    └── test_output_style.rs      # NEW: Unit tests for styling logic
```

**Structure Decision**: Single project structure maintained. New styling logic added as a service module to maintain layer separation. CLI layer delegates to service for styled output rendering.

## Complexity Tracking

> No constitution violations identified. Feature is straightforward addition.
