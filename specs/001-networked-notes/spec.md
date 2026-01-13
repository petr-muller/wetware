# Feature Specification: Networked Notes with Entity References

**Feature Branch**: `001-networked-notes`
**Created**: 2025-12-30
**Status**: Draft
**Input**: User description: "Wetware is a personal experimental tool to store short networked notes. The notes can refer to named entities. The user can run a CLI tool like this: 1. wet add 'This is a note that does not refer to any entity' 2. wet add 'This is a note that refers to an [entity]' 3. wet add 'This is a note that refers to multiple entities like [one] and [two]'. These notes are parsed and saved to storage (assume the CLI is potentially just one client to the system, additional interfaces can be built later). The content can be listed by the user again through the CLI like this: 1. `wet notes` lists all notes saved 2. `wet notes --on entity` lists all notes that refer to an entity (were added with `....[entity]....`) 3. `wet entities` lists all unique entities"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture Quick Notes (Priority: P1)

As a user, I want to quickly capture short notes through a CLI so that I can record ideas and information without interrupting my workflow.

**Why this priority**: This is the foundational capability - without the ability to add and view notes, the system has no value. This forms the MVP.

**Independent Test**: Can be fully tested by adding several notes and listing them back. Delivers immediate value by providing a simple note-taking system.

**Acceptance Scenarios**:

1. **Given** the system is available, **When** I run `wet add 'Meeting with Sarah at 3pm'`, **Then** the note is saved and I receive confirmation
2. **Given** I have added 5 notes, **When** I run `wet notes`, **Then** all 5 notes are displayed in chronological order
3. **Given** I have no notes saved, **When** I run `wet notes`, **Then** I see an appropriate message indicating no notes exist
4. **Given** I add a note with special characters like quotes or apostrophes, **When** I retrieve the notes, **Then** the text is preserved exactly as entered

---

### User Story 2 - Reference Entities in Notes (Priority: P2)

As a user, I want to reference named entities in my notes using bracket notation so that I can create connections between related information.

**Why this priority**: This enables the "networked" aspect of notes, distinguishing this from a simple list. It's the key differentiator but builds on the basic note-taking foundation.

**Independent Test**: Can be tested by adding notes with entity references and verifying the entities are recognized. Delivers value by enabling structured knowledge capture.

**Acceptance Scenarios**:

1. **Given** the system is available, **When** I run `wet add 'Discussion about [project-alpha] with [Sarah]'`, **Then** the note is saved and entities "project-alpha" and "Sarah" are recognized
2. **Given** I add a note with `[entity]` syntax, **When** I retrieve the note, **Then** the entity reference is preserved in the displayed text
3. **Given** I add multiple notes referencing the same entity, **When** I query the system, **Then** all references to that entity are tracked
4. **Given** I add a note with malformed entity syntax like `[unclosed bracket`, **When** the note is processed, **Then** the system handles it gracefully without errors

---

### User Story 3 - Query Notes by Entity (Priority: P3)

As a user, I want to filter notes by entity so that I can find all information related to a specific topic or person.

**Why this priority**: This unlocks the full value of the entity references by enabling retrieval and discovery. While valuable, the system is still useful without this query capability.

**Independent Test**: Can be tested by adding notes with various entity references and filtering by specific entities. Delivers value through improved information retrieval.

**Acceptance Scenarios**:

1. **Given** I have added notes referencing entities "Sarah", "project-alpha", and "meeting", **When** I run `wet notes --on Sarah`, **Then** only notes containing `[Sarah]` are displayed
2. **Given** I have 10 notes, 3 of which reference `[budget]`, **When** I run `wet notes --on budget`, **Then** exactly those 3 notes are shown
3. **Given** I query for an entity that doesn't exist, **When** I run `wet notes --on nonexistent`, **Then** I see a message indicating no notes found for that entity
4. **Given** I have notes with multiple entity references, **When** I filter by one entity, **Then** notes containing that entity are shown even if they reference other entities too

---

### User Story 4 - List All Entities (Priority: P3)

As a user, I want to see all unique entities I've referenced so that I can discover what topics and people I'm tracking.

**Why this priority**: This provides an overview of the knowledge graph and helps with discovery, but the system is functional without it.

**Independent Test**: Can be tested by adding notes with various entities and verifying the entity list is accurate and complete.

**Acceptance Scenarios**:

1. **Given** I have added notes referencing entities "Sarah", "project-alpha", "budget", and "Sarah" again, **When** I run `wet entities`, **Then** I see a list with 3 unique entities: "Sarah", "project-alpha", "budget"
2. **Given** I have no notes with entity references, **When** I run `wet entities`, **Then** I see a message indicating no entities have been referenced
3. **Given** I have 50 notes referencing 20 different entities, **When** I run `wet entities`, **Then** all 20 unique entities are listed
4. **Given** multiple notes reference the same entity with different casings like `[Sarah]` and `[sarah]`, **When** I list entities, **Then** only one entity appears in the list, using the capitalization from the first occurrence

---

### Edge Cases

- What happens when a note contains only whitespace?
- What happens when a note exceeds a certain length (e.g., 10,000 characters)?
- What happens when an entity name contains special characters like brackets `[entity[sub]]`?
- How does the system handle empty entity references like `[]`?
- What happens when notes are added in rapid succession (concurrent writes)?
- What happens when storage is full or unavailable?
- How are entity names with spaces handled: `[multi word entity]`?
- What happens if a user tries to add a note with no text content?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept note text input through the `wet add` command
- **FR-002**: System MUST persist notes so they survive application restarts
- **FR-003**: System MUST parse entity references using the `[entity-name]` syntax within note text
- **FR-004**: System MUST extract and track all unique entity names referenced across all notes
- **FR-005**: System MUST provide a command to list all saved notes
- **FR-006**: System MUST provide a command to filter notes by a specific entity reference
- **FR-007**: System MUST provide a command to list all unique entities
- **FR-008**: System MUST preserve the exact text of notes including entity reference syntax when displaying them
- **FR-009**: System MUST handle special characters in note text (quotes, apostrophes, newlines, etc.)
- **FR-010**: System MUST support notes referencing zero, one, or multiple entities
- **FR-011**: System MUST maintain chronological order of notes for the `wet notes` command
- **FR-012**: System MUST provide clear feedback when operations succeed or fail
- **FR-013**: System MUST handle gracefully when querying for non-existent entities
- **FR-014**: System MUST treat entity names as case-insensitive (e.g., `[Sarah]`, `[sarah]`, and `[SARAH]` refer to the same entity)
- **FR-015**: System MUST preserve and display the capitalization from the first occurrence of each entity name

### Key Entities

- **Note**: A short text entry created by the user. Contains arbitrary text content, may include zero or more entity references, has a creation timestamp.

- **Entity**: A named concept or person referenced in notes using bracket notation `[entity-name]`. Represents a connection point between related notes. Has a unique name identifier.

- **Entity Reference**: A link between a Note and an Entity, created when a note's text contains `[entity-name]` syntax. Multiple notes can reference the same entity, creating a network of related information.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can add a new note and retrieve it in under 5 seconds total time
- **SC-002**: System correctly parses and extracts entity references from 100% of valid `[entity]` syntax patterns
- **SC-003**: Users can retrieve all notes referencing a specific entity in under 2 seconds regardless of total note count
- **SC-004**: The system maintains data integrity with zero data loss across application restarts
- **SC-005**: Users can view all unique entities in under 2 seconds regardless of total note count
- **SC-006**: Command-line operations complete successfully 99% of the time under normal conditions
- **SC-007**: Note text is preserved with 100% fidelity (no character corruption or truncation for notes under reasonable size limits)

## Assumptions

- Notes are assumed to be "short" - reasonable default maximum length of 10,000 characters
- The system is single-user (no concurrent access from multiple users)
- Entity names are case-insensitive but preserve the capitalization from first occurrence
- Entity names include spacing and special characters exactly as written between brackets
- Multiple entity references in a single note are supported and tracked independently
- The system runs on a local machine with adequate storage space
- CLI is the primary interface for this feature; the architecture should support future interfaces but this spec only covers CLI functionality
- Notes are immutable once created (no edit or delete operations in this specification)
