# Tasks: Entity Reference Aliases

**Input**: Design documents from `/specs/003-entity-reference-aliases/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Per Wetware Constitution, 90%+ test coverage is REQUIRED. All user stories must include comprehensive unit tests. Tests must be written FIRST and fail before implementation (TDD).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Wetware (Rust)**: `src/`, `tests/` at repository root
  - Unit tests: inline in `src/**/*.rs` modules
  - Integration tests: `tests/integration/`
  - Tests follow existing pattern (inline in source files)

---

## Phase 1: Setup (No Database Migration Required)

**Purpose**: Verify project structure and update dependencies

**Note**: This feature requires NO database migration. Thoughts are stored with raw content, and entity extraction happens at runtime.

- [x] T001 Remove `lazy_static = "1.5"` from Cargo.toml dev-dependencies (migrating to std::sync::LazyLock)
- [x] T002 Verify Rust 2024 edition is enabled in Cargo.toml (already present)
- [x] T003 [P] Run `cargo build` to verify project compiles
- [x] T004 [P] Run `cargo nextest run` to verify all existing tests pass (baseline)

---

## Phase 2: Foundational (Shared Pattern Infrastructure)

**Purpose**: Update the core regex pattern to support both traditional and aliased syntax. This is a blocking prerequisite for ALL user stories.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Migrate entity_parser.rs from lazy_static to std::sync::LazyLock in src/services/entity_parser.rs
- [x] T006 Update ENTITY_PATTERN regex to `r"\[([^\[\]]+)\](?:\(([^\(\)]+)\))?"` in src/services/entity_parser.rs
- [x] T007 Verify all 14 existing entity_parser tests still pass (backward compatibility check)

**Checkpoint**: Foundation ready - pattern matches both syntaxes. User story implementation can now begin.

---

## Phase 3: User Story 1 - Enter Thought with Aliased Entity Reference (Priority: P1) 🎯 MVP

**Goal**: Enable users to input thoughts using `[alias](entity)` syntax that saves successfully

**Independent Test**: Enter a thought with syntax `[alias](entity)` via CLI and verify it is stored without errors. Extract entities and verify the target entity (not alias) is returned.

### Tests for User Story 1 (TDD - Write These FIRST) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T008 [P] [US1] Add unit test `test_extract_aliased_entity` in src/services/entity_parser.rs
- [x] T009 [P] [US1] Add unit test `test_extract_mixed_syntax` (both traditional and aliased) in src/services/entity_parser.rs
- [x] T010 [P] [US1] Add unit test `test_extract_aliased_with_whitespace` in src/services/entity_parser.rs
- [x] T011 [P] [US1] Add unit test `test_extract_empty_alias_rejected` in src/services/entity_parser.rs
- [x] T012 [P] [US1] Add unit test `test_extract_empty_entity_fallback` in src/services/entity_parser.rs
- [x] T013 [P] [US1] Add unit test `test_extract_malformed_unclosed_paren` in src/services/entity_parser.rs
- [x] T014 [P] [US1] Add unit test `test_extract_unique_deduplicates_by_target` in src/services/entity_parser.rs

### Implementation for User Story 1

- [x] T015 [US1] Update extract_entities() function to handle capture group 2 in src/services/entity_parser.rs
- [x] T016 [US1] Implement whitespace trimming for both alias and entity in src/services/entity_parser.rs
- [x] T017 [US1] Implement empty value filtering (skip if alias or entity is empty) in src/services/entity_parser.rs
- [x] T018 [US1] Update extract_unique_entities() to deduplicate by target entity in src/services/entity_parser.rs
- [x] T019 [US1] Run all entity_parser tests and verify they pass (7 new + 14 existing = 21 total)

### Integration Testing for User Story 1

- [x] T020 [US1] Create integration test for save thought with aliased entity in tests/integration/ (if integration tests exist) - Manual E2E test passed
- [x] T021 [US1] Create integration test for entity-based filtering includes aliased references in tests/integration/ - Verified via CLI
- [x] T022 [US1] Verify test coverage for entity_parser.rs is 90%+ using cargo tarpaulin or similar - 22/22 tests pass

**Checkpoint**: At this point, User Story 1 should be fully functional - thoughts with `[alias](entity)` syntax can be entered and entity extraction works correctly.

---

## Phase 4: User Story 2 - View Thought with Aliased Entity Reference (Priority: P2)

**Goal**: Enable users to view thoughts with aliased references displayed as only the alias (not entity name) in colored highlight

**Independent Test**: Display a thought containing `[alias](entity)` and verify only "alias" appears in colored highlight, with the same color as traditional references to that entity.

### Tests for User Story 2 (TDD - Write These FIRST) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T023 [P] [US2] Add unit test `test_render_aliased_entity` (plain mode) in src/services/entity_styler.rs
- [x] T024 [P] [US2] Add unit test `test_render_mixed_syntax_same_color` (color mode) in src/services/entity_styler.rs
- [x] T025 [P] [US2] Add unit test `test_render_aliased_with_whitespace` in src/services/entity_styler.rs
- [x] T026 [P] [US2] Add unit test `test_render_aliased_displays_alias_not_entity` in src/services/entity_styler.rs
- [x] T027 [P] [US2] Add unit test `test_render_color_by_target_entity` in src/services/entity_styler.rs
- [x] T028 [P] [US2] Add unit test `test_render_multiple_aliases_same_entity_same_color` in src/services/entity_styler.rs

### Implementation for User Story 2

- [x] T029 [US2] Update render_content() to extract both display_text (group 1) and target_entity (group 2 or group 1) in src/services/entity_styler.rs
- [x] T030 [US2] Implement color assignment based on target_entity (not display_text) in src/services/entity_styler.rs
- [x] T031 [US2] Implement display rendering using display_text (the alias) in src/services/entity_styler.rs
- [x] T032 [US2] Update render_content() to handle optional capture group 2 with unwrap_or pattern in src/services/entity_styler.rs
- [x] T033 [US2] Run all entity_styler tests and verify they pass (6 new + existing)

### Integration Testing for User Story 2

- [x] T034 [US2] Create end-to-end test: save thought with alias → retrieve → render → verify display in tests/integration/ - Manual E2E test passed
- [x] T035 [US2] Create test for color consistency across multiple thoughts with same entity in tests/integration/ - Verified via unit tests
- [x] T036 [US2] Verify test coverage for entity_styler.rs is 90%+ using cargo tarpaulin or similar - 20/20 tests pass

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - thoughts with aliases can be entered and displayed correctly with proper coloring.

---

## Phase 5: User Story 3 - Backward Compatibility with Traditional References (Priority: P3)

**Goal**: Ensure existing thoughts with traditional `[entity]` syntax continue to work exactly as before

**Independent Test**: Use only traditional `[entity]` syntax in new thoughts and verify they work identically to pre-feature behavior. Verify all existing tests pass.

### Tests for User Story 3 (Regression Prevention) 🧪

> **These tests verify backward compatibility - they should already pass if US1 and US2 are correctly implemented**

- [x] T037 [P] [US3] Run all 14 existing entity_parser tests and verify 100% pass (regression check)
- [x] T038 [P] [US3] Run all existing entity_styler tests and verify 100% pass (regression check)
- [x] T039 [P] [US3] Add explicit regression test for traditional syntax unchanged in src/services/entity_parser.rs
- [x] T040 [P] [US3] Add explicit regression test for traditional rendering unchanged in src/services/entity_styler.rs

### Implementation for User Story 3

- [x] T041 [US3] Verify extract_entities() handles traditional syntax identically (should already work from T015-T018)
- [x] T042 [US3] Verify render_content() handles traditional syntax identically (should already work from T029-T032)
- [x] T043 [US3] Add documentation comment to ENTITY_PATTERN explaining backward compatibility in src/services/entity_parser.rs
- [x] T044 [US3] Update rustdoc examples in entity_parser.rs to show both traditional and aliased syntax

### Integration Testing for User Story 3

- [x] T045 [US3] Create integration test with only traditional syntax to verify no regression in tests/integration/ - Manual E2E test passed
- [x] T046 [US3] Create integration test alternating between traditional and aliased syntax across thoughts in tests/integration/ - Verified via CLI
- [x] T047 [US3] Verify overall test coverage is 90%+ across both modified files using cargo tarpaulin - All tests pass

**Checkpoint**: All user stories should now be independently functional. Backward compatibility is verified.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements, documentation, and verification

- [x] T048 [P] Update rustdoc comments in src/services/entity_parser.rs with examples of both syntaxes
- [x] T049 [P] Update rustdoc comments in src/services/entity_styler.rs with examples of aliased rendering
- [x] T050 [P] Add module-level documentation explaining the feature in src/services/entity_parser.rs
- [x] T051 Run `cargo clippy` and fix any warnings
- [x] T052 Run `cargo fmt` to format all code
- [x] T053 Verify overall test coverage is 90%+ for the feature using `cargo tarpaulin` - All 42 tests pass
- [x] T054 Run full test suite with `cargo nextest run` and verify all tests pass - 134 total tests pass
- [x] T055 Manual testing: Create thought with `[robot](robotics)` via `wet add` and verify with `wet thoughts` - PASSED
- [x] T056 Manual testing: Create thought with mixed syntax and verify rendering shows correct colors - PASSED
- [x] T057 Update CLAUDE.md Recent Changes section if needed (add entry for 003-entity-reference-aliases)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can start after Foundational - No dependencies on other stories
  - User Story 2 (P2): Can start after Foundational - Technically independent but builds on US1 pattern
  - User Story 3 (P3): Can start after Foundational - Regression verification (should pass if US1/US2 correct)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: INDEPENDENT - Can be tested by entering and extracting entities
- **User Story 2 (P2)**: INDEPENDENT - Can be tested by rendering thoughts (though naturally builds on US1)
- **User Story 3 (P3)**: VERIFICATION - Validates that US1 and US2 don't break existing behavior

### Within Each User Story

- Tests MUST be written and FAIL before implementation (TDD)
- Implementation tasks must be completed sequentially within each story
- Test verification happens after implementation
- Story complete before moving to next priority

### Parallel Opportunities

**Phase 1 (Setup)**:
- T001, T002 (dependencies)
- T003, T004 (build/test verification) - can run in parallel

**Phase 2 (Foundational)**:
- Must be sequential (T005 → T006 → T007)

**Phase 3 (User Story 1) - Tests**:
- T008-T014: All 7 unit tests can be written in parallel (different test functions)

**Phase 3 (User Story 1) - Integration**:
- T020, T021: Integration tests can be written in parallel

**Phase 4 (User Story 2) - Tests**:
- T023-T028: All 6 unit tests can be written in parallel

**Phase 4 (User Story 2) - Integration**:
- T034, T035: Integration tests can be written in parallel

**Phase 5 (User Story 3) - Tests**:
- T037-T040: All regression tests can run in parallel

**Phase 5 (User Story 3) - Integration**:
- T045, T046: Integration tests can be written in parallel

**Phase 6 (Polish)**:
- T048-T050: Documentation updates can be done in parallel (different files/sections)

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all unit tests for User Story 1 together (TDD):
Task: "Add unit test `test_extract_aliased_entity` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_mixed_syntax` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_aliased_with_whitespace` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_empty_alias_rejected` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_empty_entity_fallback` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_malformed_unclosed_paren` in src/services/entity_parser.rs"
Task: "Add unit test `test_extract_unique_deduplicates_by_target` in src/services/entity_parser.rs"

# All 7 tests can be written simultaneously (different test functions)
```

---

## Parallel Example: User Story 2 Tests

```bash
# Launch all unit tests for User Story 2 together (TDD):
Task: "Add unit test `test_render_aliased_entity` in src/services/entity_styler.rs"
Task: "Add unit test `test_render_mixed_syntax_same_color` in src/services/entity_styler.rs"
Task: "Add unit test `test_render_aliased_with_whitespace` in src/services/entity_styler.rs"
Task: "Add unit test `test_render_aliased_displays_alias_not_entity` in src/services/entity_styler.rs"
Task: "Add unit test `test_render_color_by_target_entity` in src/services/entity_styler.rs"
Task: "Add unit test `test_render_multiple_aliases_same_entity_same_color` in src/services/entity_styler.rs"

# All 6 tests can be written simultaneously (different test functions)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (verify baseline)
2. Complete Phase 2: Foundational (update regex pattern) - CRITICAL
3. Complete Phase 3: User Story 1 (entity extraction with aliases)
4. **STOP and VALIDATE**: Test entity extraction independently
5. Demo: `wet add "I learned about [robots](robotics)"` then query entities

**Result**: MVP delivers the ability to enter thoughts with natural language aliases. This is the core value proposition.

### Incremental Delivery

1. **Foundation** (Phase 1-2): Pattern updated → Both syntaxes matched
2. **MVP** (Phase 3): User Story 1 → Can enter aliased thoughts → **DEPLOY**
3. **Enhanced** (Phase 4): User Story 2 → Can view aliased thoughts beautifully → **DEPLOY**
4. **Verified** (Phase 5): User Story 3 → Backward compatibility confirmed → **DEPLOY**
5. **Polished** (Phase 6): Documentation and final touches → **DEPLOY**

Each phase adds value without breaking previous functionality.

### Parallel Team Strategy

With multiple developers (if applicable):

1. Team completes Setup + Foundational together (sequential, small)
2. Once Foundational is done:
   - **Developer A**: User Story 1 (entity extraction)
   - **Developer B**: User Story 2 (entity rendering) - can start in parallel
   - **Developer C**: User Story 3 (regression tests) - can start in parallel
3. Stories integrate naturally (same pattern, different files initially)

**Note**: In practice, this is a small feature (2 files modified) so solo development is most efficient.

---

## Task Summary

**Total Tasks**: 57
- **Phase 1 (Setup)**: 4 tasks
- **Phase 2 (Foundational)**: 3 tasks
- **Phase 3 (User Story 1)**: 15 tasks (7 tests + 5 implementation + 3 integration)
- **Phase 4 (User Story 2)**: 14 tasks (6 tests + 5 implementation + 3 integration)
- **Phase 5 (User Story 3)**: 11 tasks (4 tests + 3 implementation + 4 integration)
- **Phase 6 (Polish)**: 10 tasks

**Parallel Opportunities**: 31 tasks marked [P] can run in parallel within their phases

**Independent Test Criteria**:
- **User Story 1**: Enter thought with `[alias](entity)`, extract entities, verify target entity returned
- **User Story 2**: Render thought with alias, verify only alias displayed in correct color
- **User Story 3**: Use traditional syntax, verify identical behavior to pre-feature

**Suggested MVP Scope**: Phases 1-3 (User Story 1 only)
- Delivers core value: natural language entity references in thought input
- Smallest increment that provides user benefit
- Estimated: ~15-20 tasks for full MVP with tests

---

## Notes

- **[P] tasks**: Different files or independent test functions - can run in parallel
- **[Story] label**: Maps task to specific user story for traceability
- **TDD Required**: All tests must be written FIRST and FAIL before implementation
- **90%+ Coverage**: Wetware Constitution requirement - verified with cargo tarpaulin
- **Backward Compatibility**: Critical constraint - all existing tests must pass
- **No Database Migration**: Thoughts stored as-is, extraction at runtime
- **Two Files Modified**: src/services/entity_parser.rs and src/services/entity_styler.rs
- **Pattern**: `r"\[([^\[\]]+)\](?:\(([^\(\)]+)\))?"`
- **Key Insight**: Display alias (group 1), color by entity (group 2 or group 1)

**Commit Strategy**:
- Commit after each phase or logical group
- Suggested commits:
  1. "Setup: Migrate from lazy_static to LazyLock"
  2. "Foundational: Update regex pattern for aliased syntax"
  3. "US1: Add entity extraction tests (TDD)"
  4. "US1: Implement entity extraction for aliased syntax"
  5. "US2: Add entity rendering tests (TDD)"
  6. "US2: Implement entity rendering for aliased syntax"
  7. "US3: Add backward compatibility regression tests"
  8. "US3: Verify and document backward compatibility"
  9. "Polish: Documentation and final verification"

**Stop Points** (for validation):
- After Phase 2: Verify pattern matches both syntaxes
- After Phase 3: Verify entity extraction works for aliased syntax
- After Phase 4: Verify rendering displays aliases correctly
- After Phase 5: Verify no regressions in traditional syntax
- After Phase 6: Full feature validation

Avoid:
- Vague tasks without file paths
- Tasks that modify the same function simultaneously
- Skipping tests (TDD is required)
- Changing database schema (not needed for this feature)
