# Implementation Plan: Edit Existing Thoughts

**Branch**: `004-edit-thoughts` | **Date**: 2026-02-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/004-edit-thoughts/spec.md`

## Summary

Add a `wet edit <ID>` subcommand that allows users to correct an existing thought's text (inline or via editor) and/or update its date, with all entity associations atomically recalculated. The implementation adds one new error variant, two new repository methods, one new CLI module, and introduces SQLite transactions for multi-step write safety. No schema changes are required.

## Technical Context

**Language/Version**: Rust 2024 edition
**Primary Dependencies**: clap 4.5 (CLI), rusqlite 0.38 (SQLite), chrono 0.4 (date parsing), tempfile 3.25 (editor temp file), regex 1.12 (entity parsing), thiserror 2.0 (error types)
**Storage**: SQLite at `~/.local/share/wetware/thoughts.db` (or `WETWARE_DB` env var). Schema unchanged — thoughts.id already exists and is surfaced in listing output.
**Testing**: cargo nextest
**Target Platform**: Linux CLI binary
**Project Type**: Single Rust project
**Performance Goals**: Edit operations complete instantaneously for any realistic thought size (<10,000 chars). No throughput requirements.
**Constraints**: Edit must be atomic (SC-006/FR-015); partial writes not acceptable. Editor behaviour must match existing `wet add --editor` and `wet entity edit` patterns exactly.
**Scale/Scope**: Single-user personal tool; no concurrency or multi-user concerns.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**I. Test-First Development (90%+ Coverage)**
- [x] Test strategy defined: unit tests for new error variant + repository methods; integration tests for the full edit command covering all acceptance scenarios and edge cases
- [x] 90%+ coverage target achievable: all new code paths are directly testable via the existing in-memory SQLite test harness
- [x] TDD approach planned: write failing tests for `ThoughtNotFound`, `update`, `unlink_all_from_thought` before implementing each

**II. Layer Separation**
- [x] Changes respect CLI / Domain / Persistence / Input layer boundaries: new `cli/edit.rs` delegates to repository methods; no domain logic in storage layer
- [x] No direct dependencies from Domain to CLI or Persistence implementation: `Thought` struct unchanged; error type extended only
- [x] Persistence abstraction maintained: new repository methods follow existing function-per-operation pattern

**III. Strong Typing & Error Handling**
- [x] Error types identified: `ThoughtNotFound(i64)` added to `ThoughtError`; `InvalidInput` reused for date parse errors and missing-argument errors
- [x] Result/Option types used: all new functions return `Result<_, ThoughtError>`; no panics in business logic
- [x] Error context propagation planned: `?` operator used throughout; user-facing errors are clear and include the thought ID

**IV. Observability & Documentation**
- [x] Public API documentation plan: rustdoc comments on all new public functions (`update`, `unlink_all_from_thought`, `execute`)
- [x] Logging strategy: success and error paths print human-readable messages to stdout/stderr consistent with existing commands
- [x] Architecture decision rationale documented: in `research.md` (transaction strategy, entity re-association strategy)

**V. Simplicity & YAGNI**
- [x] Solution is simplest that meets current requirements: reuses all existing editor, entity parser, and repository infrastructure; adds only the minimum new code
- [x] No premature optimization or abstraction: no new traits or generics introduced
- [x] Complexity justified if present: SQLite transaction is the minimum required to meet atomicity requirement (FR-015)

## Project Structure

### Documentation (this feature)

```text
specs/004-edit-thoughts/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 — implementation decisions
├── data-model.md        # Phase 1 — schema and domain model changes
├── quickstart.md        # Phase 1 — implementation guide
├── contracts/
│   └── cli-contract.md  # CLI interface contract (command signatures, scenarios, exit codes)
└── tasks.md             # Phase 2 output (/speckit.tasks — NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── cli/
│   ├── mod.rs              # MODIFIED: add Commands::Edit variant
│   ├── edit.rs             # NEW: wet edit execute function
│   ├── add.rs              # Reference (unchanged)
│   └── ...
├── errors/
│   └── thought_error.rs    # MODIFIED: add ThoughtNotFound(i64) variant
├── storage/
│   ├── thoughts_repository.rs    # MODIFIED: add update() method
│   └── entities_repository.rs   # MODIFIED: add unlink_all_from_thought() method
├── input/
│   └── editor.rs           # UNCHANGED: reused as-is
├── services/
│   └── entity_parser.rs    # UNCHANGED: reused as-is
└── main.rs                 # MODIFIED: route Commands::Edit

tests/
└── edit_thought.rs         # NEW: integration tests for edit command
```

**Structure Decision**: Single project (Option 1). No new modules, packages, or workspace members needed. The edit feature slots into the existing `src/cli/` structure following the established one-file-per-command pattern (`add.rs`, `entity_edit.rs`, etc.).

## Complexity Tracking

No constitution violations requiring justification.

The SQLite transaction in `cli/edit.rs` is the only non-trivial addition. It is justified by FR-015 (atomic edit requirement) and adds minimal complexity — rusqlite's `Transaction` API is a thin wrapper around `BEGIN/COMMIT/ROLLBACK`.
