# Tasks: Entity Descriptions

**Input**: Design documents from `/specs/001-entity-descriptions/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Per Wetware Constitution, 90%+ test coverage is REQUIRED. All user stories must include unit, integration, and contract tests as appropriate. Tests must be written FIRST and fail before implementation (TDD).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Wetware (Rust)**: `src/`, `tests/` at repository root
  - Unit tests: inline in `src/**/*.rs` or `tests/unit/`
  - Integration tests: `tests/integration/`
  - Contract tests: `tests/contract/`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependency and prepare for entity descriptions implementation

- [x] T001 Add `terminal_size = "0.3"` dependency to Cargo.toml
- [x] T002 Create migration placeholder in src/storage/migrations/add_entity_descriptions_migration.rs
- [x] T003 [P] Create description formatter module in src/services/description_formatter.rs
- [x] T004 [P] Create editor support module in src/input/editor.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data model and database migration that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Extend Entity struct in src/models/entity.rs to add description: Option<String> field
- [x] T006 Update Entity constructor methods (new, with_description, has_description, description_or_empty) in src/models/entity.rs
- [x] T007 Implement database migration in src/storage/migrations/add_entity_descriptions_migration.rs (ALTER TABLE entities ADD COLUMN description TEXT)
- [x] T008 Register migration in migration runner (src/storage/migrations/mod.rs or equivalent)
- [x] T009 Update EntitiesRepository::find_by_name in src/storage/entities_repository.rs to include description column in SELECT
- [x] T010 Update EntitiesRepository::list_all in src/storage/entities_repository.rs to include description column in SELECT
- [x] T011 Add EntitiesRepository::update_description method in src/storage/entities_repository.rs

**Checkpoint**: Foundation ready - Entity model extended, database migrated, repositories updated

---

## Phase 3: User Story 1 - Add Description to Entity (Priority: P1) 🎯 MVP

**Goal**: Users can add/edit multi-paragraph descriptions to existing entities using three input methods (inline, interactive editor, file input)

**Independent Test**: Can be fully tested by editing an existing entity with a description using `wet entity edit` and verifying the description is stored and can be retrieved

### Tests for User Story 1 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T012 [P] [US1] Contract test for `wet entity edit --description` in tests/contract/test_entity_edit_command.rs
- [x] T013 [P] [US1] Contract test for `wet entity edit --description-file` in tests/contract/test_entity_edit_command.rs
- [x] T014 [P] [US1] Contract test for `wet entity edit` interactive editor in tests/contract/test_entity_edit_command.rs
- [x] T015 [P] [US1] Contract test for whitespace-only description (removal) in tests/contract/test_entity_edit_command.rs
- [x] T016 [P] [US1] Contract test for entity not found error in tests/contract/test_entity_edit_command.rs
- [x] T017 [P] [US1] Contract test for file not found error in tests/contract/test_entity_edit_command.rs
- [x] T018 [P] [US1] Integration test for description storage/retrieval in tests/integration/test_entity_descriptions.rs
- [x] T019 [P] [US1] Integration test for entity reference auto-creation in descriptions in tests/integration/test_entity_descriptions.rs
- [x] T020 [P] [US1] Integration test for whitespace validation in tests/integration/test_entity_descriptions.rs
- [x] T021 [P] [US1] Unit tests for editor launch logic in src/input/editor.rs (inline #[cfg(test)])
- [x] T022 [P] [US1] Unit tests for file reading logic in src/cli/entity_edit.rs (inline #[cfg(test)])

### Implementation for User Story 1

- [x] T023 [US1] Implement editor launcher in src/input/editor.rs (get EDITOR env var, fallback chain, launch with temp file)
- [x] T024 [US1] Implement EntityEdit command struct in src/cli/entity_edit.rs (with clap derive)
- [x] T025 [US1] Implement inline description handler (--description flag) in src/cli/entity_edit.rs
- [x] T026 [US1] Implement file input handler (--description-file flag) in src/cli/entity_edit.rs
- [x] T027 [US1] Implement interactive editor handler (no flags) in src/cli/entity_edit.rs
- [x] T028 [US1] Implement whitespace validation (trim and check is_empty) in src/cli/entity_edit.rs
- [x] T029 [US1] Implement entity reference extraction and auto-creation in src/cli/entity_edit.rs (reuse entity_parser::extract_unique_entities)
- [x] T030 [US1] Implement entity existence validation (return error if not found) in src/cli/entity_edit.rs
- [x] T031 [US1] Implement mutual exclusivity check for input methods in src/cli/entity_edit.rs
- [x] T032 [US1] Add error types for editor launch, file I/O in src/errors/thought_error.rs or new error module
- [x] T033 [US1] Add logging for description updates, file reads, editor launches in src/cli/entity_edit.rs
- [x] T034 [US1] Register EntityEdit command in src/cli/mod.rs
- [x] T035 [US1] Add rustdoc comments to EntityEdit command and all public functions
- [ ] T036 [US1] Verify 90%+ test coverage for this story (cargo tarpaulin --lib --bins)

**Checkpoint**: At this point, `wet entity edit` command should be fully functional with all three input methods

---

## Phase 4: User Story 2 - View Entities with Description Previews (Priority: P2)

**Goal**: Users can see ellipsized description previews when listing entities, with terminal width detection and smart formatting

**Independent Test**: Can be fully tested by running `wet entities` command and verifying that entities with descriptions show ellipsized previews on a single terminal line

### Tests for User Story 2 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T037 [P] [US2] Contract test for `wet entities` with description previews in tests/contract/test_entities_command.rs
- [x] T038 [P] [US2] Contract test for `wet entities` with mixed entities (some with/without descriptions) in tests/contract/test_entities_command.rs
- [x] T039 [P] [US2] Contract test for `wet entities` on narrow terminal (<60 chars) in tests/contract/test_entities_command.rs
- [x] T040 [P] [US2] Contract test for entity references rendered as plain text in previews in tests/contract/test_entities_command.rs
- [x] T041 [P] [US2] Integration test for preview generation (first paragraph extraction) in tests/integration/test_description_formatter.rs
- [x] T042 [P] [US2] Integration test for ellipsization with word boundaries in tests/integration/test_description_formatter.rs
- [x] T043 [P] [US2] Integration test for entity reference markup stripping in tests/integration/test_description_formatter.rs
- [x] T044 [P] [US2] Integration test for newline collapsing in tests/integration/test_description_formatter.rs
- [x] T045 [P] [US2] Unit tests for terminal width detection in src/services/description_formatter.rs (inline #[cfg(test)])
- [x] T046 [P] [US2] Unit tests for preview length calculation in src/services/description_formatter.rs (inline #[cfg(test)])

### Implementation for User Story 2

- [x] T047 [P] [US2] Implement terminal width detection function in src/services/description_formatter.rs (use terminal_size crate)
- [x] T048 [P] [US2] Implement first paragraph extraction function in src/services/description_formatter.rs (split on \n\n)
- [x] T049 [US2] Implement entity reference markup stripping function in src/services/description_formatter.rs (reuse entity_parser regex)
- [x] T050 [US2] Implement newline collapsing function in src/services/description_formatter.rs (replace \n with space)
- [x] T051 [US2] Implement word-boundary ellipsization function in src/services/description_formatter.rs (truncate at last space, append …)
- [x] T052 [US2] Implement full preview generation function in src/services/description_formatter.rs (compose all helpers)
- [x] T053 [US2] Update entities command in src/cli/entities.rs to detect terminal width
- [x] T054 [US2] Update entities command in src/cli/entities.rs to check if width < 60 (suppress preview)
- [x] T055 [US2] Update entities command in src/cli/entities.rs to generate and display previews for entities with descriptions
- [x] T056 [US2] Update entities command in src/cli/entities.rs to display entity name only for entities without descriptions
- [x] T057 [US2] Add rustdoc comments to DescriptionFormatter module and all public functions
- [x] T058 [US2] Verify 90%+ test coverage for this story (cargo tarpaulin --lib --bins)

**Checkpoint**: At this point, `wet entities` command should display previews correctly with terminal width adaptation

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories and final validation

- [x] T059 [P] Update CLAUDE.md with entity descriptions feature in Recent Changes section
- [x] T060 [P] Add usage examples to README.md based on quickstart.md
- [x] T061 Code cleanup and refactoring (remove dead code, improve naming)
- [x] T062 Run `cargo clippy` and fix all warnings
- [x] T063 Run `cargo fmt` to ensure consistent formatting
- [x] T064 Verify overall 90%+ test coverage across all stories (cargo tarpaulin --lib --bins)
- [x] T065 Run all tests (cargo nextest run) and ensure 100% pass rate
- [x] T066 Manually validate quickstart.md examples against running CLI
- [x] T067 Test edge cases from spec.md (whitespace, long descriptions, narrow terminals, entity references)
- [x] T068 Performance validation (description operations complete in <100ms)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on T001-T004 completion - BLOCKS all user stories
- **User Stories (Phase 3-4)**: Both depend on Foundational phase (T005-T011) completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2)
- **Polish (Phase 5)**: Depends on both user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on US2
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on US1 (but logically follows for UX)

### Within Each User Story

- Tests (T012-T022 for US1, T037-T046 for US2) MUST be written and FAIL before implementation
- All tests within a story can run in parallel (marked [P])
- Implementation tasks follow dependencies:
  - US1: T023 (editor) → T024-T033 (command impl) → T034 (registration)
  - US2: T047-T052 (formatter) can run in parallel → T053-T056 (command update) sequential

### Parallel Opportunities

- **Setup (Phase 1)**: T003 and T004 can run in parallel
- **Foundational (Phase 2)**: T005-T006 (entity model) → T009-T011 (repository) sequential, but T007-T008 (migration) can run in parallel with T009-T011 after T005-T006
- **US1 Tests**: All test tasks T012-T022 can run in parallel
- **US1 Implementation**: T025, T026, T027 can run in parallel after T024
- **US2 Tests**: All test tasks T037-T046 can run in parallel
- **US2 Implementation**: T047, T048, T050, T051 can run in parallel
- **Polish**: T059, T060 can run in parallel
- **User Stories**: US1 (Phase 3) and US2 (Phase 4) can be worked on in parallel by different team members after Foundational phase

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all contract tests for User Story 1 together:
Task: "Contract test for `wet entity edit --description` in tests/contract/test_entity_edit_command.rs"
Task: "Contract test for `wet entity edit --description-file` in tests/contract/test_entity_edit_command.rs"
Task: "Contract test for `wet entity edit` interactive editor in tests/contract/test_entity_edit_command.rs"
Task: "Contract test for whitespace-only description (removal) in tests/contract/test_entity_edit_command.rs"
Task: "Contract test for entity not found error in tests/contract/test_entity_edit_command.rs"
Task: "Contract test for file not found error in tests/contract/test_entity_edit_command.rs"

# All these can be written in parallel, run them to verify they all fail (TDD)
```

## Parallel Example: User Story 2 Formatter Implementation

```bash
# Launch formatter functions in parallel (different responsibilities):
Task: "Implement terminal width detection function in src/services/description_formatter.rs"
Task: "Implement first paragraph extraction function in src/services/description_formatter.rs"
Task: "Implement newline collapsing function in src/services/description_formatter.rs"
Task: "Implement word-boundary ellipsization function in src/services/description_formatter.rs"

# Then compose them:
Task: "Implement full preview generation function in src/services/description_formatter.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T004)
2. Complete Phase 2: Foundational (T005-T011) - CRITICAL
3. Write all tests for User Story 1 (T012-T022), verify they FAIL
4. Complete Phase 3: User Story 1 implementation (T023-T036)
5. **STOP and VALIDATE**: Test User Story 1 independently
   - Create entity: `wet add "Learning about @test"`
   - Add description: `wet entity edit test --description "Test description"`
   - Verify storage (check database or list entities)
6. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
   - Users can now add/edit descriptions via three methods
3. Add User Story 2 → Test independently → Deploy/Demo
   - Users can now see description previews when listing entities
4. Each story adds value without breaking previous functionality

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T011)
2. Once Foundational is done:
   - Developer A: User Story 1 (T012-T036) - `wet entity edit` command
   - Developer B: User Story 2 (T037-T058) - Preview display in `entities` command
3. Stories complete and integrate independently
4. Both work together on Polish (T059-T068)

---

## Notes

- [P] tasks = different files, no dependencies - can run in parallel
- [Story] label maps task to specific user story (US1 or US2) for traceability
- Each user story should be independently completable and testable
- **TDD Required**: Verify tests fail before implementing (Constitution requirement)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Constitution compliance: 90%+ coverage, layer separation, Result/Option types, rustdoc, logging
- Entity references reuse existing parser (src/services/entity_parser.rs)
- Terminal width detection uses new terminal_size crate dependency
