# Tasks: Networked Notes with Entity References

**Input**: Design documents from `/specs/001-networked-notes/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Per Wetware Constitution, 90%+ test coverage is REQUIRED. All user stories must include unit, integration, and contract tests as appropriate. Tests must be written FIRST and fail before implementation (TDD).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Wetware (Rust)**: `src/`, `tests/` at repository root
  - Unit tests: inline in `src/**/*.rs` or `tests/unit/`
  - Integration tests: `tests/integration/`
  - Contract tests: `tests/contract/`
- File extensions: `.rs` (Rust source files)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create database migration for notes, entities, and note_entities tables in src/storage/migrations/001_networked_notes.rs
- [x] T002 [P] Add regex dependency to Cargo.toml for entity parsing
- [x] T003 [P] Add lazy_static dependency to Cargo.toml for regex compilation optimization
- [x] T004 [P] Add thiserror dependency to Cargo.toml for error handling

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Create NoteError enum in src/errors/note_error.rs with EmptyContent, ContentTooLong, ParseError, StorageError variants
- [x] T006 Create Note domain model in src/models/note.rs with id, content, created_at fields
- [x] T007 Create Entity domain model in src/models/entity.rs with id, name, canonical_name fields
- [x] T008 [P] Implement entity parser service in src/services/entity_parser.rs using regex pattern `\[([^\[\]]+)\]`
- [x] T009 Create database connection pool in src/storage/connection.rs
- [x] T010 Implement migration runner in src/storage/migrations/mod.rs to execute schema creation

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Capture Quick Notes (Priority: P1) 🎯 MVP

**Goal**: Enable users to add and list notes via CLI

**Independent Test**: Add several notes and list them back. System provides basic note-taking functionality.

### Tests for User Story 1 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [ ] T011 [P] [US1] Contract test for `wet add` command success in tests/contract/test_add_command.rs
- [ ] T012 [P] [US1] Contract test for `wet add` command with empty content error in tests/contract/test_add_command.rs
- [ ] T013 [P] [US1] Contract test for `wet add` command with oversized content error in tests/contract/test_add_command.rs
- [ ] T014 [P] [US1] Contract test for `wet notes` command listing all notes in tests/contract/test_notes_command.rs
- [ ] T015 [P] [US1] Contract test for `wet notes` command with empty database in tests/contract/test_notes_command.rs
- [ ] T016 [P] [US1] Integration test for note persistence and retrieval in tests/integration/test_note_persistence.rs
- [ ] T017 [P] [US1] Unit tests for Note validation (empty content, max length) in src/models/note.rs

### Implementation for User Story 1

- [ ] T018 [US1] Implement Note::new() constructor with validation in src/models/note.rs
- [ ] T019 [US1] Implement Note::validate_content() method in src/models/note.rs
- [ ] T020 [US1] Create NotesRepository trait in src/storage/notes_repository.rs
- [ ] T021 [US1] Implement NotesRepository::save() method in src/storage/notes_repository.rs
- [ ] T022 [US1] Implement NotesRepository::list_all() method in src/storage/notes_repository.rs
- [ ] T023 [US1] Implement `wet add` CLI command in src/cli/add.rs using clap
- [ ] T024 [US1] Implement `wet notes` CLI command in src/cli/notes.rs
- [ ] T025 [US1] Wire add command to main CLI in src/cli/mod.rs
- [ ] T026 [US1] Wire notes command to main CLI in src/cli/mod.rs
- [ ] T027 [US1] Add error handling and user-friendly messages in src/cli/add.rs
- [ ] T028 [US1] Add chronological ordering logic in src/cli/notes.rs
- [ ] T029 [US1] Verify 90%+ test coverage for User Story 1 (`cargo tarpaulin`)

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently. Users can add and list notes.

---

## Phase 4: User Story 2 - Reference Entities in Notes (Priority: P2)

**Goal**: Enable entity extraction and tracking when notes are added

**Independent Test**: Add notes with `[entity]` syntax and verify entities are extracted and stored

### Tests for User Story 2 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [ ] T030 [P] [US2] Unit tests for entity parser with valid syntax in src/services/entity_parser.rs
- [ ] T031 [P] [US2] Unit tests for entity parser with malformed syntax in src/services/entity_parser.rs
- [ ] T032 [P] [US2] Unit tests for entity parser with edge cases (empty brackets, spaces, special chars) in src/services/entity_parser.rs
- [ ] T033 [P] [US2] Unit tests for Entity model case normalization in src/models/entity.rs
- [ ] T034 [P] [US2] Integration test for entity extraction and persistence in tests/integration/test_entity_references.rs
- [ ] T035 [P] [US2] Integration test for case-insensitive entity matching in tests/integration/test_entity_references.rs
- [ ] T036 [P] [US2] Integration test for first-occurrence capitalization preservation in tests/integration/test_entity_references.rs

### Implementation for User Story 2

- [ ] T037 [P] [US2] Implement Entity::new() constructor with case normalization in src/models/entity.rs
- [ ] T038 [P] [US2] Implement Entity::display_name() method in src/models/entity.rs
- [ ] T039 [US2] Implement EntityParser::extract_entities() method in src/services/entity_parser.rs
- [ ] T040 [US2] Create EntitiesRepository trait in src/storage/entities_repository.rs
- [ ] T041 [US2] Implement EntitiesRepository::find_or_create() method in src/storage/entities_repository.rs
- [ ] T042 [US2] Implement EntitiesRepository::link_to_note() method in src/storage/entities_repository.rs
- [ ] T043 [US2] Integrate entity extraction into `wet add` command in src/cli/add.rs
- [ ] T044 [US2] Update NotesRepository::save() to handle entity linking in src/storage/notes_repository.rs
- [ ] T045 [US2] Add entity count to `wet add` success message in src/cli/add.rs
- [ ] T046 [US2] Add logging for entity extraction operations in src/services/entity_parser.rs
- [ ] T047 [US2] Verify 90%+ test coverage for User Story 2 (`cargo tarpaulin`)

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently. Notes can contain entity references that are automatically extracted and tracked.

---

## Phase 5: User Story 3 - Query Notes by Entity (Priority: P3)

**Goal**: Enable filtering notes by entity reference

**Independent Test**: Add notes with various entity references and filter by specific entities

### Tests for User Story 3 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [ ] T048 [P] [US3] Contract test for `wet notes --on <entity>` command success in tests/contract/test_notes_command.rs
- [ ] T049 [P] [US3] Contract test for `wet notes --on` with non-existent entity in tests/contract/test_notes_command.rs
- [ ] T050 [P] [US3] Contract test for `wet notes --on` with case-insensitive matching in tests/contract/test_notes_command.rs
- [ ] T051 [P] [US3] Integration test for entity filtering query in tests/integration/test_entity_references.rs
- [ ] T052 [P] [US3] Integration test for filtering notes with multiple entity references in tests/integration/test_entity_references.rs

### Implementation for User Story 3

- [ ] T053 [US3] Implement NotesRepository::list_by_entity() method in src/storage/notes_repository.rs
- [ ] T054 [US3] Implement EntitiesRepository::find_by_name() method in src/storage/entities_repository.rs
- [ ] T055 [US3] Add --on <entity> argument to `wet notes` command in src/cli/notes.rs
- [ ] T056 [US3] Implement entity filtering logic in src/cli/notes.rs
- [ ] T057 [US3] Add user-friendly message for non-existent entities in src/cli/notes.rs
- [ ] T058 [US3] Add logging for entity query operations in src/cli/notes.rs
- [ ] T059 [US3] Verify 90%+ test coverage for User Story 3 (`cargo tarpaulin`)

**Checkpoint**: At this point, User Stories 1, 2, AND 3 should all work independently. Users can filter notes by entity references.

---

## Phase 6: User Story 4 - List All Entities (Priority: P3)

**Goal**: Enable discovery of all unique entities

**Independent Test**: Add notes with various entities and verify entity list is accurate and complete

### Tests for User Story 4 (REQUIRED for 90%+ coverage) 🧪

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [ ] T060 [P] [US4] Contract test for `wet entities` command success in tests/contract/test_entities_command.rs
- [ ] T061 [P] [US4] Contract test for `wet entities` with no entities in database in tests/contract/test_entities_command.rs
- [ ] T062 [P] [US4] Contract test for `wet entities` with canonical capitalization in tests/contract/test_entities_command.rs
- [ ] T063 [P] [US4] Integration test for unique entity listing in tests/integration/test_entity_references.rs
- [ ] T064 [P] [US4] Integration test for alphabetical ordering of entities in tests/integration/test_entity_references.rs

### Implementation for User Story 4

- [ ] T065 [US4] Implement EntitiesRepository::list_all() method in src/storage/entities_repository.rs
- [ ] T066 [US4] Implement `wet entities` CLI command in src/cli/entities.rs
- [ ] T067 [US4] Wire entities command to main CLI in src/cli/mod.rs
- [ ] T068 [US4] Add alphabetical sorting in src/cli/entities.rs
- [ ] T069 [US4] Add user-friendly message for empty entity list in src/cli/entities.rs
- [ ] T070 [US4] Add logging for entity listing operations in src/cli/entities.rs
- [ ] T071 [US4] Verify 90%+ test coverage for User Story 4 (`cargo tarpaulin`)

**Checkpoint**: All user stories should now be independently functional. Complete networked note system is operational.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T072 [P] Add rustdoc comments to all public models in src/models/
- [ ] T073 [P] Add rustdoc comments to all public services in src/services/
- [ ] T074 [P] Add rustdoc comments to all public repositories in src/storage/
- [ ] T075 [P] Add rustdoc comments to all CLI commands in src/cli/
- [ ] T076 Verify overall 90%+ test coverage across all stories (`cargo tarpaulin`)
- [ ] T077 Run `cargo clippy` and fix all warnings
- [ ] T078 Run `cargo fmt` to ensure consistent formatting
- [ ] T079 [P] Performance benchmark: Verify `wet add` completes in <100ms
- [ ] T080 [P] Performance benchmark: Verify `wet notes` completes in <100ms
- [ ] T081 [P] Performance benchmark: Verify `wet notes --on` completes in <500ms
- [ ] T082 [P] Performance benchmark: Verify `wet entities` completes in <100ms
- [ ] T083 Run quickstart.md validation (manual testing of all examples)
- [ ] T084 Review error messages for clarity and actionability
- [ ] T085 Verify all edge cases from spec.md are handled

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User Story 1 (P1): Can start after Foundational - No dependencies on other stories
  - User Story 2 (P2): Can start after Foundational - Extends User Story 1 but independently testable
  - User Story 3 (P3): Can start after Foundational - Uses entities from User Story 2 but independently testable
  - User Story 4 (P3): Can start after Foundational - Uses entities from User Story 2 but independently testable
- **Polish (Phase 7)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1 - MVP)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational - Logically extends US1 but can be tested independently
- **User Story 3 (P3)**: Can start after Foundational - Uses entity infrastructure but independent
- **User Story 4 (P3)**: Can start after Foundational - Uses entity infrastructure but independent

**Note**: User Stories 2, 3, and 4 all work with entities, so they share some infrastructure. However, each can be independently tested and delivers standalone value.

### Within Each User Story

- Tests (REQUIRED) MUST be written and FAIL before implementation
- Models before services
- Services before repositories
- Repositories before CLI commands
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- All test tasks for a user story marked [P] can run in parallel
- Models and error types marked [P] can run in parallel
- Different user stories CAN be worked on in parallel by different team members (after Foundational phase)

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task T011: "Contract test for `wet add` command success"
Task T012: "Contract test for `wet add` with empty content error"
Task T013: "Contract test for `wet add` with oversized content"
Task T014: "Contract test for `wet notes` listing all notes"
Task T015: "Contract test for `wet notes` with empty database"
Task T016: "Integration test for note persistence"
Task T017: "Unit tests for Note validation"

# Launch parallel models (after tests fail):
Task T018: "Implement Note::new() constructor"
Task T019: "Implement Note::validate_content()"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo (Networked notes capability)
4. Add User Story 3 → Test independently → Deploy/Demo (Entity filtering)
5. Add User Story 4 → Test independently → Deploy/Demo (Entity discovery)
6. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (MVP)
   - Developer B: User Story 2 (Entity extraction)
   - Developer C: User Story 3 (Entity filtering)
   - Developer D: User Story 4 (Entity listing)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
- TDD is MANDATORY per Wetware Constitution: Write tests first, ensure they fail, then implement

---

## Task Summary

**Total Tasks**: 85
- Setup (Phase 1): 4 tasks
- Foundational (Phase 2): 6 tasks
- User Story 1 (P1 - MVP): 19 tasks (7 tests + 12 implementation)
- User Story 2 (P2): 18 tasks (7 tests + 11 implementation)
- User Story 3 (P3): 12 tasks (5 tests + 7 implementation)
- User Story 4 (P3): 12 tasks (5 tests + 7 implementation)
- Polish & Cross-Cutting: 14 tasks

**Parallel Opportunities**: 45 tasks marked [P] can run concurrently

**MVP Scope** (User Story 1 only): 29 tasks (Setup + Foundational + US1)

**Test Coverage**: 24 test tasks ensuring 90%+ coverage requirement

**Independent Test Criteria**:
- US1: Add notes and list them chronologically
- US2: Add notes with `[entity]` and verify extraction
- US3: Filter notes by entity reference
- US4: List all unique entities with canonical capitalization

All tasks follow strict checklist format and include exact file paths for immediate execution.
