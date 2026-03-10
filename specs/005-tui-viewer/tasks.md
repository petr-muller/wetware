# Tasks: Interactive TUI Thought Viewer

**Input**: Design documents from `/specs/005-tui-viewer/`
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

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add new dependencies and create module scaffolding

- [x] T001 Add ratatui, crossterm, tui-input, and nucleo-matcher dependencies to Cargo.toml
- [x] T002 Create TUI module directory structure with empty files: src/tui/mod.rs, src/tui/state.rs, src/tui/ui.rs, src/tui/input.rs
- [x] T003 Register the tui module in src/lib.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and CLI integration that ALL user stories depend on

**Warning**: No user story work can begin until this phase is complete

- [x] T004 Add TuiError(String) variant to ThoughtError enum in src/errors/thought_error.rs
- [x] T005 [P] Implement SortOrder enum (Ascending, Descending) with toggle() and label() methods in src/tui/state.rs
- [x] T006 [P] Implement Mode enum (Normal, EntityPicker, EntityDetail) with all variant fields in src/tui/state.rs
- [x] T007 Implement App struct with new() constructor, displayed_thoughts recomputation method, and should_quit flag in src/tui/mod.rs. App::new() takes Vec<Thought> and Vec<Entity>, initializes with SortOrder::Ascending, Mode::Normal, no active filter, and computes initial displayed_thoughts
- [x] T008 Add Tui variant to Commands enum in src/cli/mod.rs
- [x] T009 Create CLI handler src/cli/tui.rs with execute(db_path) function: open DB connection, load thoughts via ThoughtsRepository::list_all() and entities via EntitiesRepository::list_all(), initialize terminal (crossterm raw mode + alternate screen), create App, call App::run(), restore terminal on exit (including on panic via std::panic::set_hook or scopeguard)
- [x] T010 Wire Tui command dispatch in src/main.rs to call cli::tui::execute()
- [x] T011 Implement skeleton App::run() event loop in src/tui/mod.rs: draw frame via ui::render(), read crossterm key event, dispatch to input::handle_key_event(), check should_quit to break loop

**Checkpoint**: `cargo run -- tui` launches an empty TUI that responds to `q` to quit

---

## Phase 3: User Story 1 - Browse Thoughts in TUI (Priority: P1) MVP

**Goal**: Display thoughts in a scrollable list with entity references highlighted in color, keyboard navigation, and clean exit

**Independent Test**: Launch TUI with populated database, verify thoughts displayed with dates and highlighted entities, scroll through list, quit with q/Esc

### Tests for User Story 1 (REQUIRED for 90%+ coverage)

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T012 [P] [US1] Unit tests for App state: test new() initializes with all thoughts in displayed_thoughts, test scrolling updates list_state selection, test quit sets should_quit=true. Add inline tests in src/tui/mod.rs or src/tui/state.rs
- [x] T013 [P] [US1] Unit tests for Normal mode key handling: test Up/Down/PgUp/PgDn/Home/End update list_state, test q sets should_quit, test Esc sets should_quit when no active filter. Add inline tests in src/tui/input.rs
- [x] T014 [P] [US1] Unit tests for entity-to-ratatui color mapping function: test each owo_colors::AnsiColors variant maps to correct ratatui::style::Color. Add inline tests in src/tui/ui.rs
- [x] T015 [P] [US1] Integration test for thought list rendering using ratatui TestBackend: create App with sample thoughts (some with entity references), render a frame, assert buffer contains thought dates and content text with styled entity spans. Add in tests/integration/test_tui_rendering.rs
- [x] T016 [P] [US1] Contract test: run `wet tui` binary, send q keypress, verify process exits with status 0. Add in tests/contract/test_tui_command.rs

### Implementation for User Story 1

- [x] T017 [US1] Implement ansi_to_ratatui_color() mapping function in src/tui/ui.rs to convert owo_colors::AnsiColors to ratatui::style::Color for the 12-color entity palette
- [x] T018 [US1] Implement render_thought_list() in src/tui/ui.rs: for each displayed thought, parse entity references using ENTITY_PATTERN regex, build a ratatui Line with Span segments (plain text in default style, entity references in colored style using ansi_to_ratatui_color), prepend date formatted as "YYYY-MM-DD HH:MM". Use ratatui List widget with ListState for selection and scrolling
- [x] T019 [US1] Implement render_status_bar() in src/tui/ui.rs: show key hints at bottom (q: quit, /: filter, s: sort, Enter: details). Use a horizontal layout with styled spans
- [x] T020 [US1] Implement render() in src/tui/ui.rs: split frame into main area (thought list) and bottom row (status bar), delegate to render_thought_list() and render_status_bar()
- [x] T021 [US1] Implement handle_normal_mode() in src/tui/input.rs: handle Up/Down (select prev/next in ListState), PgUp/PgDn (jump by page), Home/End (first/last), q (set should_quit), Esc (set should_quit if no active_filter)
- [x] T022 [US1] Implement handle_key_event() dispatcher in src/tui/input.rs: match on app.mode and delegate to mode-specific handler (only Normal mode for now)
- [x] T023 [US1] Handle empty database state: if no thoughts exist, render an informative centered message "No thoughts recorded yet" instead of the list in src/tui/ui.rs
- [x] T024 [US1] Add rustdoc comments to all public types and functions in src/tui/mod.rs, src/tui/state.rs, src/tui/ui.rs, src/tui/input.rs

**Checkpoint**: `cargo run -- tui` shows all thoughts with colored entity references, scrollable with arrow keys, quit with q

---

## Phase 4: User Story 3 - Sort Thoughts by Date (Priority: P2)

**Goal**: Toggle sort order between ascending (oldest first) and descending (newest first) with a single keypress, with visual indicator in status bar

**Independent Test**: Launch TUI, verify default ascending order, press s, verify order reverses, press s again, verify order restores

### Tests for User Story 3 (REQUIRED for 90%+ coverage)

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T025 [P] [US3] Unit tests for SortOrder: test toggle() flips Ascending to Descending and back, test label() returns "Oldest first" / "Newest first". Add inline tests in src/tui/state.rs
- [x] T026 [P] [US3] Unit tests for sort behavior: test that pressing s in Normal mode toggles sort_order and recomputes displayed_thoughts in correct order. Create App with thoughts at different dates, verify displayed_thoughts order changes. Add inline tests in src/tui/input.rs
- [x] T027 [P] [US3] Integration test: render status bar with TestBackend, verify sort order label appears. Add in tests/integration/test_tui_rendering.rs

### Implementation for User Story 3

- [x] T028 [US3] Implement displayed_thoughts recomputation method on App in src/tui/mod.rs: sort thoughts indices by created_at based on current sort_order, apply active_filter if set
- [x] T029 [US3] Add s key handler in handle_normal_mode() in src/tui/input.rs: call sort_order.toggle() then recompute displayed_thoughts, preserve list selection position if possible
- [x] T030 [US3] Update render_status_bar() in src/tui/ui.rs to show current sort order label (e.g., "Sort: Oldest first" or "Sort: Newest first")

**Checkpoint**: Sort toggle works with s key, status bar shows current order, filter+sort compose correctly

---

## Phase 5: User Story 2 - Filter Thoughts by Entity (Priority: P2)

**Goal**: Open fuzzy entity picker overlay with /, type to fuzzy-filter entities, select to apply filter, Esc to dismiss or clear active filter

**Independent Test**: Launch TUI, press /, verify entity picker appears, type partial entity name, verify fuzzy matches shown, select entity, verify thought list filtered, press Esc to clear filter

### Tests for User Story 2 (REQUIRED for 90%+ coverage)

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T031 [P] [US2] Unit tests for EntityPicker mode transitions: test / in Normal opens EntityPicker with all entities as initial matches, test Esc in EntityPicker returns to Normal without changing filter, test Enter in EntityPicker sets active_filter and returns to Normal. Add inline tests in src/tui/input.rs
- [x] T032 [P] [US2] Unit tests for fuzzy matching: test typing in EntityPicker filters entity list by fuzzy score using nucleo-matcher, test empty input shows all entities, test no matches shows empty list. Add inline tests in src/tui/mod.rs or separate helper
- [x] T033 [P] [US2] Unit tests for filter application: test that setting active_filter recomputes displayed_thoughts to only include thoughts referencing that entity (using entity_parser::extract_entities), test clearing filter restores all thoughts. Add inline tests in src/tui/mod.rs
- [x] T034 [P] [US2] Unit tests for Esc behavior with active filter: test that Esc in Normal mode with active_filter clears the filter instead of quitting. Add inline tests in src/tui/input.rs
- [x] T035 [P] [US2] Integration test: render entity picker overlay with TestBackend, verify overlay contains entity names and search input. Add in tests/integration/test_tui_rendering.rs

### Implementation for User Story 2

- [x] T036 [US2] Implement fuzzy match helper on App in src/tui/mod.rs: given current EntityPicker input text, use nucleo-matcher to score all entity canonical_names, return sorted indices of matching entities (score > 0, or all if input is empty)
- [x] T037 [US2] Implement handle_entity_picker_mode() in src/tui/input.rs: handle text input (forward to tui-input Input, recompute fuzzy matches), Up/Down (move picker selection), Enter (apply selected entity as active_filter, recompute displayed_thoughts, switch to Normal mode), Esc (switch to Normal mode, no filter change)
- [x] T038 [US2] Update handle_normal_mode() in src/tui/input.rs: handle / key to switch to EntityPicker mode (initialize Input, compute initial matches as all entities)
- [x] T039 [US2] Update handle_normal_mode() Esc behavior in src/tui/input.rs: if active_filter is Some, clear it and recompute displayed_thoughts instead of quitting
- [x] T040 [US2] Implement render_entity_picker() in src/tui/ui.rs: render a centered overlay (Clear widget + Block with border), show tui-input text field at top, list of fuzzy-matched entity names below with current selection highlighted, show count of matches
- [x] T041 [US2] Update render() in src/tui/ui.rs to call render_entity_picker() when mode is EntityPicker (after rendering main list)
- [x] T042 [US2] Update render_status_bar() in src/tui/ui.rs to show active filter entity name when filter is active (e.g., "Filter: ProjectX")
- [x] T043 [US2] Handle no-match filter state: when filter is active but no thoughts reference the entity, show "No thoughts referencing [entity]" message in the list area in src/tui/ui.rs

**Checkpoint**: Fuzzy entity picker works, filter applies/clears correctly, composes with sort order

---

## Phase 6: User Story 4 - View Entity Description (Priority: P3)

**Goal**: Select a thought and press Enter/d to open a centered modal popup showing descriptions of all entities referenced in that thought

**Independent Test**: Select a thought with entity references, press d, verify popup shows entity descriptions, press Esc to close

### Tests for User Story 4 (REQUIRED for 90%+ coverage)

> **CRITICAL: Write these tests FIRST, ensure they FAIL before implementation (TDD)**

- [x] T044 [P] [US4] Unit tests for EntityDetail mode transitions: test Enter/d in Normal mode with selected thought opens EntityDetail with correct entity indices, test Esc in EntityDetail returns to Normal. Add inline tests in src/tui/input.rs
- [x] T045 [P] [US4] Unit tests for entity extraction from selected thought: test that pressing detail key on a thought parses its content for entity references and looks up matching entities from the loaded entities list. Add inline tests in src/tui/mod.rs
- [x] T046 [P] [US4] Integration test: render entity detail popup with TestBackend using sample entities (with and without descriptions), verify popup contains entity names and descriptions, verify "No description" indicator for entities without one. Add in tests/integration/test_tui_rendering.rs

### Implementation for User Story 4

- [x] T047 [US4] Implement helper on App to extract entity indices for selected thought in src/tui/mod.rs: parse selected thought's content with entity_parser::extract_unique_entities(), find matching entities in the loaded entities list by name, return their indices
- [x] T048 [US4] Implement handle_entity_detail_mode() in src/tui/input.rs: handle Up/Down (scroll within popup), Esc (switch to Normal mode)
- [x] T049 [US4] Update handle_normal_mode() in src/tui/input.rs: handle Enter and d keys to extract entity indices for selected thought and switch to EntityDetail mode (skip if no entities found in selected thought)
- [x] T050 [US4] Implement render_entity_detail() in src/tui/ui.rs: render a centered modal popup (Clear + Block with border and title), list each entity's canonical_name followed by its description (or "No description available"), support scrolling for long descriptions, highlight entity references within descriptions using the same color mapping
- [x] T051 [US4] Update render() in src/tui/ui.rs to call render_entity_detail() when mode is EntityDetail (after rendering main list)

**Checkpoint**: Entity description popup works for thoughts with/without entities, scrollable, dismissible with Esc

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, resilience, and final quality checks

- [x] T052 [P] Handle terminal resize events in App::run() event loop in src/tui/mod.rs: on crossterm Resize event, the next draw call will automatically use new dimensions (ratatui handles this), but ensure list selection stays valid
- [x] T053 [P] Handle long thought content in render_thought_list() in src/tui/ui.rs: truncate content that exceeds available width with ellipsis indicator
- [x] T054 [P] Ensure terminal restoration on panic in src/cli/tui.rs: use a scopeguard or panic hook to call crossterm::terminal::disable_raw_mode() and LeaveAlternateScreen even if App::run() panics
- [x] T055 Run cargo clippy and fix all warnings across new TUI code
- [x] T056 Run cargo fmt to format all new code
- [x] T057 Verify 90%+ test coverage across all TUI code (cargo tarpaulin or similar)
- [x] T058 Run full test suite: cargo nextest run — verify no regressions in existing tests

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **US1 (Phase 3)**: Depends on Foundational — this is the MVP
- **US3 Sort (Phase 4)**: Depends on US1 (needs thought list and status bar)
- **US2 Filter (Phase 5)**: Depends on US1 (needs thought list), benefits from US3 (sort composability)
- **US4 Entity Detail (Phase 6)**: Depends on US1 (needs thought list with selection)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Depends only on Foundational — no other story dependencies
- **US3 (P2)**: Depends on US1 (needs the displayed_thoughts infrastructure and status bar)
- **US2 (P2)**: Depends on US1 (needs rendered thought list), should follow US3 (sort+filter composition)
- **US4 (P3)**: Depends on US1 (needs list selection), independent of US2/US3

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- State/logic before rendering
- Rendering before input handling
- Core functionality before edge cases

### Parallel Opportunities

- T005 and T006 (state types) can run in parallel
- All test tasks within each user story marked [P] can run in parallel
- T052, T053, T054 (polish tasks) can run in parallel
- US4 can run in parallel with US2/US3 (after US1 completes)

---

## Parallel Example: User Story 1

```text
# Launch all tests for US1 together:
T012: Unit tests for App state in src/tui/mod.rs
T013: Unit tests for Normal mode key handling in src/tui/input.rs
T014: Unit tests for color mapping in src/tui/ui.rs
T015: Integration test for thought list rendering in tests/integration/test_tui_rendering.rs
T016: Contract test for wet tui command in tests/contract/test_tui_command.rs
```

## Parallel Example: User Story 2

```text
# Launch all tests for US2 together:
T031: Unit tests for EntityPicker mode transitions in src/tui/input.rs
T032: Unit tests for fuzzy matching in src/tui/mod.rs
T033: Unit tests for filter application in src/tui/mod.rs
T034: Unit tests for Esc with active filter in src/tui/input.rs
T035: Integration test for entity picker rendering in tests/integration/test_tui_rendering.rs
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: `cargo run -- tui` shows scrollable thought list with entity highlighting
5. Demo-ready MVP

### Incremental Delivery

1. Setup + Foundational → TUI scaffolding compiles and launches
2. US1 → Scrollable thought list with entity highlighting (MVP!)
3. US3 → Sort toggle with visual indicator
4. US2 → Fuzzy entity picker filter
5. US4 → Entity description popup
6. Polish → Edge cases, resilience, coverage verification

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US3 (Sort) is placed before US2 (Filter) despite both being P2, because Sort is simpler and its status bar infrastructure is used by Filter
- US4 (Entity Detail) can run in parallel with US2/US3 after US1 completes
- Reuse existing entity_parser and entity_styler — do not duplicate parsing logic
- All ratatui rendering can be tested via TestBackend without a real terminal
- Commit after each task or logical group
