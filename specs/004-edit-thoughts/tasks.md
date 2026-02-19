# Tasks: Edit Existing Thoughts

**Input**: Design documents from `specs/004-edit-thoughts/`
**Prerequisites**: plan.md ✓, spec.md ✓, research.md ✓, data-model.md ✓, contracts/cli-contract.md ✓, quickstart.md ✓

**Tests**: Per Wetware Constitution, 90%+ test coverage is REQUIRED. Tests are written FIRST (TDD): write a failing test, then implement to make it pass. Stubs with `todo!()` are created in Phase 2 to allow tests to compile and fail in Phase 3+.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no unresolved dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths are included in all descriptions

## Path Conventions

- Unit tests: inline in `src/**/*.rs` test modules
- Integration tests: `tests/` at repository root (e.g., `tests/edit_thought.rs`)

---

## Phase 1: Setup (Command Plumbing)

**Purpose**: Wire the new `wet edit` subcommand into the CLI infrastructure so all user story phases can compile.

- [x] T001 Add `Commands::Edit { id: i64, content: Option<String>, date: Option<String>, editor: bool }` variant to the `Commands` enum in `src/cli/mod.rs` (add `#[arg(long)]` to `date` and `editor`; add `#[arg(long, conflicts_with = "content")]` to `editor`)
- [x] T002 Create `src/cli/edit.rs` with a stub `pub fn execute(id: i64, content: Option<String>, date: Option<String>, use_editor: bool, db_path: Option<&std::path::Path>) -> Result<(), crate::errors::thought_error::ThoughtError>` that returns `todo!()`
- [x] T003 Route `Commands::Edit { id, content, date, editor }` to `cli::edit::execute(id, content, date, editor, db_path.as_deref())?` in `src/main.rs` (after T001, T002)

**Checkpoint**: Project compiles with `cargo build` (stub panics at runtime — that is expected)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Error types and repository stubs that MUST exist before any user story tests can compile.

**⚠️ CRITICAL**: No user story test can compile until this phase is complete.

- [x] T004 [P] Add `#[error("Thought with ID {0} not found")] ThoughtNotFound(i64)` variant to `ThoughtError` enum in `src/errors/thought_error.rs`
- [x] T005 [P] Add stub `pub fn update(conn: &rusqlite::Connection, id: i64, content: Option<&str>, created_at: Option<chrono::DateTime<chrono::Utc>>) -> Result<(), ThoughtError>` returning `todo!()` to `ThoughtsRepository` in `src/storage/thoughts_repository.rs`
- [x] T006 [P] Add stub `pub fn unlink_all_from_thought(conn: &rusqlite::Connection, thought_id: i64) -> Result<(), ThoughtError>` returning `todo!()` to `EntitiesRepository` in `src/storage/entities_repository.rs`

**Checkpoint**: Project compiles. All stubs panic at runtime. User story tests can now be written.

---

## Phase 3: User Story 1 — Edit Thought Text via Direct CLI Command (Priority: P1) 🎯 MVP

**Goal**: Users can correct or replace the text of an existing thought by ID using a direct CLI argument. Entity associations are atomically recalculated from the new text.

**Independent Test**: Record a thought with entity references, run `wet edit <id> "new text with [NewEntity]"`, verify the thought's content changed and entity associations reflect the new text only.

### Tests for User Story 1 (TDD — write FIRST, ensure they FAIL before implementing) 🧪

- [x] T007 [P] [US1] Write unit test for `ThoughtNotFound` error message formatting (`assert_eq!(format!("{}", ThoughtError::ThoughtNotFound(42)), "Thought with ID 42 not found")`) as an inline `#[test]` in `src/errors/thought_error.rs`
- [x] T008 [P] [US1] Write unit tests for `ThoughtsRepository::update` in `src/storage/thoughts_repository.rs`: (a) updating content stores new text and leaves date unchanged; (b) calling with a nonexistent ID returns `ThoughtNotFound`; (c) updating to empty string is rejected with `EmptyContent` — use `get_memory_connection()` and `run_migrations()`
- [x] T009 [P] [US1] Write unit test for `EntitiesRepository::unlink_all_from_thought` in `src/storage/entities_repository.rs`: insert a thought with two entity links, call `unlink_all_from_thought`, assert `thought_entities` table is empty for that thought ID
- [x] T010 [P] [US1] Write integration tests in `tests/edit_thought.rs` for the direct-argument success path: edit thought to new text without entity refs (entities cleared), edit thought to new text replacing one entity with another (old unlinked, new linked), edit thought adding an entity that didn't exist (auto-created)
- [x] T011 [P] [US1] Write integration tests in `tests/edit_thought.rs` for the failure path: nonexistent ID returns error, empty string argument returns error, providing no arguments (no content/date/editor) returns usage error

### Implementation for User Story 1

- [x] T012 [US1] Implement `ThoughtsRepository::update` body in `src/storage/thoughts_repository.rs`: call `get_by_id` to verify existence (propagate `ThoughtNotFound`), build values from `content`/`created_at` params (fall back to existing if `None`), execute `UPDATE thoughts SET content = ?1, created_at = ?2 WHERE id = ?3` (after T008 tests exist and fail)
- [x] T013 [US1] Implement `EntitiesRepository::unlink_all_from_thought` body in `src/storage/entities_repository.rs`: execute `DELETE FROM thought_entities WHERE thought_id = ?1` (after T009 test exists and fails)
- [x] T014 [US1] Implement the content-argument path in `cli::edit::execute` in `src/cli/edit.rs`: (1) validate at least one of `content`/`date`/`use_editor` is provided — return `InvalidInput` if none; (2) call `get_by_id` to fetch existing thought; (3) open a `conn.transaction()`; (4) call `ThoughtsRepository::update` with new content; (5) call `EntitiesRepository::unlink_all_from_thought`; (6) re-extract entities from new text via `entity_parser::extract_unique_entities`, call `EntitiesRepository::find_or_create` and `link_to_thought` for each; (7) `tx.commit()`; (8) print `"Thought {id} updated."` (after T010, T011 tests exist and fail)
- [x] T015 [US1] Run `cargo nextest run` and verify all US1 tests pass; run `cargo clippy` and resolve any warnings

**Checkpoint**: `wet edit <id> "new text"` fully functional. US1 independently testable and complete.

---

## Phase 4: User Story 2 — Edit Thought Text via Interactive Editor (Priority: P2)

**Goal**: Users can open an existing thought in their configured text editor, modify the content, save, and have entity associations updated — mirroring the `wet add --editor` experience.

**Independent Test**: Record a thought, run `wet edit <id> --editor` with `EDITOR` set to a script that appends `" [NewEntity]"` to the file, verify thought content and entity associations updated.

### Tests for User Story 2 (TDD — write FIRST, ensure they FAIL before implementing) 🧪

- [x] T016 [P] [US2] Write integration test in `tests/edit_thought.rs` for the editor success path: set `EDITOR` to a script that replaces the file content with known text, run `wet edit <id> --editor`, verify content and entities updated
- [x] T017 [P] [US2] Write integration tests in `tests/edit_thought.rs` for editor no-change cases: editor leaves content identical → `"No changes made to thought {id}."` printed, no DB modification; editor exits with non-zero status → warning printed, thought unchanged

### Implementation for User Story 2

- [x] T018 [US2] Implement the `--editor` path in `cli::edit::execute` in `src/cli/edit.rs`: call `input::editor::launch_editor(Some(&thought.content))`; compare returned string to `thought.content` — if identical, print `"No changes made to thought {id}."` and return `Ok(())`; if editor exits abnormally (error from `launch_editor`), print warning and return `Ok(())`; otherwise proceed with the same transaction block as the content-argument path (T014), reusing entity re-association logic (after T016, T017 tests exist and fail)
- [x] T019 [US2] Run `cargo nextest run` and verify all US2 tests pass; run `cargo clippy` and resolve any warnings

**Checkpoint**: `wet edit <id> --editor` fully functional. US2 independently testable and complete.

---

## Phase 5: User Story 3 — Edit Thought Date via CLI Command (Priority: P3)

**Goal**: Users can correct the date of an existing thought using `--date YYYY-MM-DD` without re-entering the thought's text. Text and entity associations are unchanged.

**Independent Test**: Record a thought, run `wet edit <id> --date 2026-01-15`, verify the thought's date is 2026-01-15 and content/entities are unchanged.

### Tests for User Story 3 (TDD — write FIRST, ensure they FAIL before implementing) 🧪

- [x] T020 [P] [US3] Write unit tests for `ThoughtsRepository::update` date path in `src/storage/thoughts_repository.rs`: updating only `created_at` stores the new date and leaves content unchanged; use an in-memory connection
- [x] T021 [P] [US3] Write integration tests in `tests/edit_thought.rs` for `--date`: valid date updates thought date and leaves content/entities unchanged; invalid date format returns a clear error; `--date` combined with new content text updates both atomically

### Implementation for User Story 3

- [x] T022 [US3] Implement `--date` argument parsing in `cli::edit::execute` in `src/cli/edit.rs`: parse `date` string with `chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")`, convert to `DateTime<Utc>` via `.and_hms_opt(0,0,0).unwrap().and_utc()`; return `ThoughtError::InvalidInput(format!("Invalid date format '{}'. Expected YYYY-MM-DD.", s))` on parse failure; pass resulting `Option<DateTime<Utc>>` to the existing transaction block alongside optional new content (after T020, T021 tests exist and fail)
- [x] T023 [US3] Run `cargo nextest run` and verify all US3 tests pass; run `cargo clippy` and resolve any warnings

**Checkpoint**: `wet edit <id> --date YYYY-MM-DD` fully functional. All three user stories independently testable and complete.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T024 [P] Add rustdoc comments to all new public functions: `ThoughtsRepository::update`, `EntitiesRepository::unlink_all_from_thought`, `cli::edit::execute` — document parameters, return values, and error variants
- [x] T025 [P] Write atomicity integration test in `tests/edit_thought.rs`: verify that a transaction rollback on simulated failure (e.g., providing invalid content after unlink) leaves the thought in its original state with original entity associations
- [x] T026 Update `README.md` with `wet edit` command examples covering all three user story scenarios (direct text, editor, date)
- [x] T027 Run full `cargo nextest run`, `cargo clippy`, and `cargo fmt` — all must pass with zero warnings or formatting changes

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user story test compilation
- **US1 (Phase 3)**: Depends on Phase 2 — foundational stubs must exist for tests to compile
- **US2 (Phase 4)**: Depends on Phase 3 — shares entity re-association logic from US1's `execute` implementation
- **US3 (Phase 5)**: Depends on Phase 2 — shares `ThoughtsRepository::update` from Phase 2 stub (implemented in Phase 3); can begin in parallel with Phase 4 after Phase 3 is complete
- **Polish (Phase 6)**: Depends on all user story phases

### User Story Dependencies

- **US1 (P1)**: Independent after Phase 2
- **US2 (P2)**: Shares `execute` function body with US1; Phase 4 adds the `--editor` branch to existing code
- **US3 (P3)**: Shares `ThoughtsRepository::update` and transaction block with US1; Phase 5 adds date parsing to existing code

### Within Each User Story

1. Write failing tests (TDD) — all marked [P] within a story can be written simultaneously
2. Implement repository methods to pass unit tests
3. Implement CLI execute path to pass integration tests
4. Run `cargo nextest run` to confirm all pass

### Parallel Opportunities

- T004, T005, T006 can run in parallel (different files)
- T007, T008, T009, T010, T011 can all be written in parallel (different test locations)
- T016, T017 can be written in parallel
- T020, T021 can be written in parallel
- T024, T025 can run in parallel

---

## Parallel Example: User Story 1

```bash
# Write all US1 tests simultaneously (they all fail):
Task T007: Unit test for ThoughtNotFound in src/errors/thought_error.rs
Task T008: Unit tests for ThoughtsRepository::update in src/storage/thoughts_repository.rs
Task T009: Unit test for unlink_all_from_thought in src/storage/entities_repository.rs
Task T010: Integration tests (success path) in tests/edit_thought.rs
Task T011: Integration tests (failure path) in tests/edit_thought.rs

# Then implement sequentially (T012 → T013 → T014):
Task T012: ThoughtsRepository::update body
Task T013: EntitiesRepository::unlink_all_from_thought body
Task T014: cli::edit::execute content path with transaction
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T003)
2. Complete Phase 2: Foundational (T004–T006)
3. Complete Phase 3: US1 (T007–T015)
4. **STOP and VALIDATE**: `wet edit <id> "new text"` works end-to-end with correct entity associations
5. Ship or demo the MVP

### Incremental Delivery

1. Setup + Foundational → compile, stub panics → Foundation ready
2. US1 complete → `wet edit <id> "text"` functional → MVP
3. US2 complete → `wet edit <id> --editor` functional → Full text editing
4. US3 complete → `wet edit <id> --date YYYY-MM-DD` functional → Full feature
5. Polish → docs, atomicity test, README

---

## Notes

- [P] tasks operate on different files and have no unresolved dependencies at the time they run
- All new public functions need rustdoc (Constitution IV)
- Stubs with `todo!()` in Phase 2 allow tests to compile and fail cleanly — do not skip Phase 2
- The transaction in `cli::edit::execute` wraps the entire write path: update thought + delete entity links + re-insert entity links. This is the atomicity guarantee from FR-015/SC-006.
- `wet add` in `src/cli/add.rs` is the reference implementation — study it before implementing `cli::edit::execute`
