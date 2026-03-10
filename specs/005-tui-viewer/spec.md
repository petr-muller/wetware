# Feature Specification: Interactive TUI Thought Viewer

**Feature Branch**: `005-tui-viewer`
**Created**: 2026-03-09
**Status**: Draft
**Input**: User description: "Implement a simple interactive TUI viewer for thoughts. The TUI should allow interactive filter by entity, sort by date (ascending and descending) and should highlight entities accordingly. There should be a way to display entity description too. No editing capabilities necessary yet."

## Clarifications

### Session 2026-03-09

- Q: How should entity filtering work - exact match, substring, or fuzzy? → A: Fuzzy picker - user opens a picker overlay listing all entities with fuzzy search, selects which entity to filter by.
- Q: How should entity descriptions be displayed - side panel, modal overlay, or bottom panel? → A: Modal overlay/popup - description appears as a centered popup over the thought list, dismissed with Esc.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse Thoughts in TUI (Priority: P1)

A user launches the TUI viewer to browse all their recorded thoughts in an interactive, scrollable list. Each thought is displayed with its date and content, with entity references visually highlighted in the content text. The user can scroll through the list using keyboard navigation.

**Why this priority**: This is the core value proposition - providing an interactive, readable view of thoughts that improves on the existing static CLI listing.

**Independent Test**: Can be fully tested by launching the TUI with a populated database and verifying that thoughts are displayed in a scrollable list with highlighted entities.

**Acceptance Scenarios**:

1. **Given** a database with multiple thoughts, **When** the user launches the TUI viewer, **Then** all thoughts are displayed in a scrollable list showing date and content
2. **Given** thoughts containing entity references like `[ProjectX]` or `[alias](entity)`, **When** the TUI renders the thought list, **Then** entity references are visually highlighted (distinct color or style) within the thought content
3. **Given** more thoughts than fit on screen, **When** the user presses up/down arrow keys, **Then** the list scrolls accordingly
4. **Given** the TUI is open, **When** the user presses `q` or `Esc`, **Then** the TUI closes and the terminal is restored to its previous state

---

### User Story 2 - Filter Thoughts by Entity (Priority: P2)

A user wants to focus on thoughts related to a specific entity. They press a key to open a fuzzy entity picker overlay that lists all known entities. The user types to fuzzy-filter the entity list, selects an entity, and the thought list narrows to show only thoughts that reference the selected entity. The user can clear the filter to return to the full list.

**Why this priority**: Filtering is the primary interactive capability that distinguishes the TUI from the existing `wet thoughts --on <entity>` command, enabling faster exploratory workflows with a discoverable entity picker.

**Independent Test**: Can be tested by launching the TUI, opening the entity picker, selecting an entity, and verifying that only matching thoughts are shown.

**Acceptance Scenarios**:

1. **Given** the TUI is showing all thoughts, **When** the user presses `/` to open the entity picker, **Then** an overlay appears listing all known entities with a fuzzy search input
2. **Given** the entity picker is open, **When** the user types characters, **Then** the entity list is fuzzy-filtered to show matching entities
3. **Given** the entity picker shows filtered entities, **When** the user selects an entity (e.g., with Enter), **Then** the picker closes and the thought list shows only thoughts referencing that entity
4. **Given** a filter is active, **When** the user clears the filter (e.g., pressing `Esc` or a clear shortcut), **Then** all thoughts are shown again
5. **Given** the user selects an entity that has no associated thoughts, **When** the filter is applied, **Then** the list shows an empty state with a clear message indicating no matching thoughts
6. **Given** the entity picker is open, **When** the user presses `Esc`, **Then** the picker closes without applying a filter

---

### User Story 3 - Sort Thoughts by Date (Priority: P2)

A user wants to change the order in which thoughts are displayed. They toggle between ascending (oldest first) and descending (newest first) date sort order using a keyboard shortcut. The current sort order is indicated in the UI.

**Why this priority**: Sorting gives users control over how they review their thoughts chronologically, which is essential for both reviewing recent activity and exploring history.

**Independent Test**: Can be tested by launching the TUI with thoughts spanning multiple dates, toggling sort order, and verifying the display order changes.

**Acceptance Scenarios**:

1. **Given** the TUI is showing thoughts, **When** the user presses the sort toggle key (e.g., `s`), **Then** the sort order toggles between ascending and descending by date
2. **Given** a sort order is active, **When** the user looks at the TUI header or status area, **Then** the current sort order is visually indicated (e.g., "Date: Oldest first" or "Date: Newest first")
3. **Given** a filter is active and the user toggles sort, **When** the sort changes, **Then** the filtered results are re-sorted without losing the filter

---

### User Story 4 - View Entity Description (Priority: P3)

A user sees an entity reference in a thought and wants to learn more about that entity. They select a thought and invoke a detail action that opens a centered modal popup displaying the descriptions of entities referenced in that thought. The popup overlays the thought list and is dismissed with Esc, returning the user to the list. This provides context without leaving the TUI.

**Why this priority**: Entity descriptions add contextual depth but are supplementary to the core browsing and filtering workflows.

**Independent Test**: Can be tested by selecting a thought that references an entity with a description, invoking the detail popup, and verifying the description is displayed.

**Acceptance Scenarios**:

1. **Given** the user has selected a thought referencing an entity with a description, **When** the user presses a detail key (e.g., `Enter` or `d`), **Then** a centered modal popup displays the entity's description over the thought list
2. **Given** a thought references multiple entities, **When** the user views entity details, **Then** the popup shows descriptions for all referenced entities
3. **Given** a thought references an entity without a description, **When** the user views entity details, **Then** the entity is listed in the popup with an indication that no description is available
4. **Given** an entity description popup is shown, **When** the user presses `Esc`, **Then** the popup closes and the user returns to the thought list

---

### Edge Cases

- What happens when the database is empty? The TUI shows an informative empty state message.
- What happens when the terminal is very narrow? The TUI adapts layout to available terminal width, truncating or wrapping content as needed.
- What happens when the terminal is resized while the TUI is open? The TUI redraws to fit the new dimensions.
- What happens when a thought content is very long? Content is truncated in the list view with an indication that more text exists.
- What happens when entity descriptions contain entity references themselves? Entity references within descriptions are highlighted consistently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a new CLI subcommand that launches the interactive TUI viewer
- **FR-002**: System MUST display thoughts in a scrollable list showing the thought date and content
- **FR-003**: System MUST visually highlight entity references within thought content using distinct colors or text styles
- **FR-004**: System MUST support keyboard-based scrolling through the thought list (arrow keys, Page Up/Down, Home/End)
- **FR-005**: System MUST allow the user to filter the thought list by opening a fuzzy entity picker overlay
- **FR-006**: System MUST display all known entities in the picker and support fuzzy search to narrow the list
- **FR-007**: System MUST allow the user to toggle sort order between ascending and descending by date
- **FR-008**: System MUST visually indicate the current sort order in the interface
- **FR-009**: System MUST display entity descriptions in a centered modal popup overlay when the user invokes the detail action on a selected thought
- **FR-010**: System MUST allow the user to exit the TUI and restore the terminal to its previous state
- **FR-011**: System MUST display a help indicator or key legend so the user knows available shortcuts
- **FR-012**: System MUST handle empty databases and no-match filter results with informative messages

### Key Entities

- **Thought**: A timestamped piece of text content that may contain entity references. Key attributes: id, content, date created.
- **Entity**: A named concept referenced within thoughts. Key attributes: name, canonical display name, optional description.
- **Entity Reference**: An inline reference to an entity within thought content, either as `[entity]` or `[alias](entity)`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can browse their full thought history interactively without needing to pipe CLI output through external tools
- **SC-002**: Users can filter thoughts to a specific entity in under 3 seconds using the interactive filter
- **SC-003**: Users can toggle sort order with a single keypress and see results update immediately
- **SC-004**: Entity references are visually distinguishable from surrounding text at a glance
- **SC-005**: Users can access entity descriptions from the thought list without running separate commands
- **SC-006**: The TUI remains responsive and usable with databases containing at least 1,000 thoughts

## Assumptions

- The TUI is a read-only viewer; no editing or creation of thoughts is in scope for this feature.
- The existing database and domain model remain unchanged; the TUI reads from the same SQLite database.
- Default sort order when launching the TUI is ascending (oldest first), matching existing `wet thoughts` behavior.
- The TUI subcommand name will be determined during implementation planning.
- Entity highlighting in the TUI reuses the same entity reference parsing logic already in the codebase.
- The filter mode uses the `/` key convention common in terminal applications (vim, less, etc.).
