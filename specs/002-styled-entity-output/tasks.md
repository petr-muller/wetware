# Tasks: Styled Entity Output

**Input**: Design documents from `/specs/002-styled-entity-output/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Per Wetware Constitution, 90%+ test coverage is REQUIRED. All user stories must include unit, integration, and contract tests as appropriate. Tests must be written FIRST and fail before implementation (TDD).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Wetware (Rust)**: `src/`, `tests/` at repository root
  - Unit tests: inline in `src/**/*.rs` or `tests/unit/`
  - Integration tests: `tests/integration/`
  - Contract tests: `tests/contract/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add required dependency and create basic module structure

- [x] T001 Add owo-colors dependency to Cargo.toml with supports-colors feature
- [x] T002 [P] Create empty src/services/color_mode.rs module file
- [x] T003 [P] Create empty src/services/entity_styler.rs module file
- [x] T004 Export new modules in src/services/mod.rs

**Checkpoint**: Dependencies resolved, module structure ready

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Implement ColorMode enum with Always/Auto/Never variants in src/services/color_mode.rs
- [x] T006 Add clap ValueEnum derive to ColorMode for CLI argument parsing in src/services/color_mode.rs
- [x] T007 Implement ColorMode::should_use_colors() method using std::io::IsTerminal in src/services/color_mode.rs
- [x] T008 Add --color global flag to CLI struct in src/cli/mod.rs
- [x] T009 [P] Add unit tests for ColorMode::should_use_colors() in src/services/color_mode.rs

**Checkpoint**: Foundation ready - ColorMode can be parsed from CLI and determines styling behavior

---

## Phase 3: User Story 1+2 - Styled Entity Display (Priority: P1) 🎯 MVP

**Goal**: Display entities with bold text and distinct colors, without markup brackets. US1 (styling) and US2 (markup removal) are combined because styling inherently requires markup removal.

**Independent Test**: Run `wet thoughts` in terminal and verify entities appear bold, colored, without brackets, with consistent color per entity.

### Tests for User Story 1+2 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T010 [P] [US1] Unit test: same entity gets same color (case-insensitive) in src/services/entity_styler.rs
- [x] T011 [P] [US1] Unit test: different entities get different colors in src/services/entity_styler.rs
- [x] T012 [P] [US1] Unit test: colors cycle when entities exceed palette size (12+) in src/services/entity_styler.rs
- [x] T013 [P] [US2] Unit test: render_content strips bracket markup from entities in src/services/entity_styler.rs
- [x] T014 [P] [US2] Unit test: plain text segments preserved unchanged in src/services/entity_styler.rs
- [x] T015 [P] [US1] Unit test: styled output includes bold and color ANSI codes in src/services/entity_styler.rs
- [x] T016 [US1] Contract test: wet thoughts output shows styled entities in tests/contract/test_thoughts_command.rs

### Implementation for User Story 1+2

- [x] T017 [US1] Define ENTITY_COLORS constant array with 12 ANSI colors in src/services/entity_styler.rs
- [x] T018 [US1] Implement EntityStyler struct with color_map HashMap and next_color field in src/services/entity_styler.rs
- [x] T019 [US1] Implement EntityStyler::new(use_colors: bool) constructor in src/services/entity_styler.rs
- [x] T020 [US1] Implement EntityStyler::get_color() for consistent color assignment in src/services/entity_styler.rs
- [x] T021 [US2] Implement EntityStyler::render_content() to parse and render thought content in src/services/entity_styler.rs
- [x] T022 [US1] Apply bold and color styling in render_content() when use_colors is true in src/services/entity_styler.rs
- [x] T023 [US1] Update thoughts command to use EntityStyler for output in src/cli/thoughts.rs
- [x] T024 [US1] Pass ColorMode from CLI args to thoughts command execution in src/cli/thoughts.rs
- [x] T025 [US1] Add rustdoc comments to EntityStyler public API in src/services/entity_styler.rs
- [x] T026 [US1] Verify 90%+ test coverage for entity_styler module

**Checkpoint**: `wet thoughts` displays styled entities in terminal with consistent colors and no brackets

---

## Phase 4: User Story 3 - Automatic Plain Output (Priority: P2)

**Goal**: Automatically disable styling when output is piped or redirected (non-TTY)

**Independent Test**: Run `wet thoughts | cat` or `wet thoughts > file.txt` and verify no ANSI escape codes in output.

### Tests for User Story 3 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T027 [P] [US3] Contract test: piped output contains no ANSI escape codes in tests/contract/test_thoughts_command.rs
- [x] T028 [P] [US3] Contract test: output contains entity text without brackets when piped in tests/contract/test_thoughts_command.rs

### Implementation for User Story 3

- [x] T029 [US3] Ensure ColorMode::Auto correctly returns false for non-TTY in src/services/color_mode.rs
- [x] T030 [US3] Integration test: verify end-to-end behavior with pipe simulation in tests/integration/test_styled_output.rs

**Checkpoint**: `wet thoughts | cat` produces clean, ANSI-free output with entities visible as plain text

---

## Phase 5: User Story 4 - Explicit Styling Control (Priority: P3)

**Goal**: Allow users to force colors on/off regardless of terminal detection via `--color` flag

**Independent Test**: Run `wet thoughts --color=never` in terminal (no styling) and `wet thoughts --color=always | cat` (has styling).

### Tests for User Story 4 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T031 [P] [US4] Contract test: --color=never disables styling in terminal in tests/contract/test_thoughts_command.rs
- [x] T032 [P] [US4] Contract test: --color=always enables styling when piped in tests/contract/test_thoughts_command.rs
- [x] T033 [P] [US4] Unit test: ColorMode::Always always returns true from should_use_colors() in src/services/color_mode.rs
- [x] T034 [P] [US4] Unit test: ColorMode::Never always returns false from should_use_colors() in src/services/color_mode.rs

### Implementation for User Story 4

- [x] T035 [US4] Verify --color flag correctly overrides auto-detection in src/cli/thoughts.rs
- [x] T036 [US4] Test edge case: --color=always with redirect to file produces ANSI codes in tests/contract/test_thoughts_command.rs

**Checkpoint**: Users have full control over styling with explicit --color flag

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and verification across all user stories

- [x] T037 Run cargo clippy and fix any warnings
- [x] T038 Run cargo fmt to ensure consistent formatting
- [x] T039 Verify overall 90%+ test coverage with cargo tarpaulin or cargo llvm-cov
- [x] T040 [P] Add edge case test: 0 entities in output produces normal output in tests/contract/test_thoughts_command.rs
- [x] T041 [P] Add edge case test: entity with special characters renders correctly in tests/contract/test_thoughts_command.rs
- [x] T042 Run full test suite: cargo nextest run
- [x] T043 Manual validation per quickstart.md verification checklist

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1+2 (P1): Core styling - should complete first
  - US3 (P2): TTY detection - builds on US1+2
  - US4 (P3): Explicit control - builds on US3
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

```
Phase 2: Foundational (ColorMode + CLI flag)
    │
    ▼
Phase 3: US1+2 (Styled Entity Display) ← MVP
    │
    ▼
Phase 4: US3 (Automatic Plain Output)
    │
    ▼
Phase 5: US4 (Explicit Styling Control)
    │
    ▼
Phase 6: Polish
```

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Core types/structs before methods
- Methods before CLI integration
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 1 (Setup)**:
- T002, T003 can run in parallel (different files)

**Phase 2 (Foundational)**:
- T009 can run in parallel with T005-T008 (test file vs implementation)

**Phase 3 (US1+2)**:
- T010, T011, T012, T013, T014, T015 can all run in parallel (unit tests)
- T017, T018, T019, T020 can run in sequence (building on each other)

**Phase 4-5**:
- Contract tests (T027, T028, T031, T032) can run in parallel
- Unit tests (T033, T034) can run in parallel

---

## Parallel Example: User Story 1+2 Tests

```bash
# Launch all unit tests for US1+2 in parallel:
Task: "Unit test: same entity gets same color in src/services/entity_styler.rs"
Task: "Unit test: different entities get different colors in src/services/entity_styler.rs"
Task: "Unit test: colors cycle when entities exceed palette in src/services/entity_styler.rs"
Task: "Unit test: render_content strips bracket markup in src/services/entity_styler.rs"
Task: "Unit test: plain text segments preserved unchanged in src/services/entity_styler.rs"
Task: "Unit test: styled output includes bold and color ANSI in src/services/entity_styler.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1+2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1+2
4. **STOP and VALIDATE**: Test styled output in terminal
5. Deploy/demo if ready - basic styled entities work!

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1+2 → Test independently → Demo (MVP with styled entities!)
3. Add US3 → Test independently → Demo (now works in pipes)
4. Add US4 → Test independently → Demo (full user control)
5. Each story adds value without breaking previous stories

### Single Developer Strategy (Recommended)

Work sequentially through phases:
1. Phase 1: Setup (~15 min)
2. Phase 2: Foundational (~30 min)
3. Phase 3: US1+2 MVP (~2-3 hours)
4. **Validate and potentially ship MVP**
5. Phase 4: US3 (~30 min)
6. Phase 5: US4 (~30 min)
7. Phase 6: Polish (~30 min)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US1 and US2 combined because markup removal is prerequisite for styling
- All tests use TDD: write test → verify failure → implement → verify pass
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
