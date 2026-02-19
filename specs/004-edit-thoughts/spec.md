# Feature Specification: Edit Existing Thoughts

**Feature Branch**: `004-edit-thoughts`
**Created**: 2026-02-19
**Status**: Draft
**Input**: User description: "Add the functionality to edit existing thoughts, both through direct CLI command (alternative subcommand to existing adding one) and also through the editor. When thoughts contain references to entities, the references need to be properly updated. It also needs to be possible to edit the date associated with a thought; that may not be necessary to do in the editor, just through CLI."

## Clarifications

### Session 2026-02-19

- Q: Does `wet` listing currently show numeric IDs for thoughts, or does this feature require adding ID display to the listing output? → A: IDs are already displayed in `wet` listing output in the format `[id] TIMESTAMP - content`; no listing changes are needed as part of this feature.
- Q: Should a user be able to edit both text and date in a single command? → A: Yes; a single `wet edit` invocation may combine new text and a new date simultaneously.
- Q: When the editor terminates unexpectedly (crash, force-quit), should the system treat this as no-change or as an error requiring user action? → A: Treat as no-change; leave the thought unmodified and report a warning to the user.
- Q: Should edit operations be atomic (all-or-nothing) or best-effort? → A: Atomic; if any part of an edit fails, the entire operation is rolled back and the thought remains in its original state.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Edit Thought Text via Direct CLI Command (Priority: P1)

A user remembers they recorded a thought with a typo or wants to correct/refine its content. They identify the thought they want to change and provide the corrected text directly as a CLI argument, similar to how they added the thought originally. The system updates the thought's text and recalculates all entity associations from the new content.

**Why this priority**: This is the most fundamental editing capability. Users who have made mistakes in recorded thoughts have no current way to correct them. It mirrors the existing add workflow, making it immediately familiar.

**Independent Test**: Can be tested by recording a thought, then editing it via CLI with different text, and verifying the stored thought reflects the new content with correct entity associations.

**Acceptance Scenarios**:

1. **Given** a thought exists with text "Met with [Alice] about project", **When** the user runs `wet edit <thought-id> "Met with [Alice] and [Bob] about project"`, **Then** the thought's text is updated and both Alice and Bob are associated with the thought.
2. **Given** a thought with text "Reviewed [Project-X] docs", **When** the user edits it to "Reviewed [Project-Y] docs", **Then** Project-X is disassociated and Project-Y is associated with the thought.
3. **Given** no thought exists with the given identifier, **When** the user runs `wet edit <nonexistent-id> "some text"`, **Then** the system displays an error message and makes no changes.
4. **Given** a thought with entity references, **When** the user edits it to text with no entity references, **Then** all previous entity associations are removed.

---

### User Story 2 - Edit Thought Text via Interactive Editor (Priority: P2)

A user wants to make substantial changes to a thought's text and prefers to use their configured text editor, similar to how they can create thoughts using the editor. They invoke an edit command that opens the existing thought content in the editor, allow them to modify it, and upon saving the changes are applied with entity associations updated.

**Why this priority**: The editor workflow is especially valuable for multi-line or complex thought content. It extends the existing editor-based input pattern to editing, providing a consistent experience.

**Independent Test**: Can be tested by recording a thought, invoking editor-based edit, modifying content in the editor, saving, and verifying the stored thought matches the modified content with correct entity associations.

**Acceptance Scenarios**:

1. **Given** a thought exists, **When** the user invokes `wet edit <thought-id> --editor` (or equivalent), **Then** the thought's current text is pre-populated in the editor.
2. **Given** the editor opens with the existing thought text, **When** the user modifies the text and saves the file, **Then** the thought is updated with the new text and entity associations are recalculated.
3. **Given** the editor opens with the existing thought text, **When** the user closes the editor without saving (or saves unchanged content), **Then** the thought is not modified and the user is informed of no change.
4. **Given** the user is in the editor, **When** they save a modified version that adds new entity references, **Then** those entities are associated with the thought after saving.

---

### User Story 3 - Edit Thought Date via CLI Command (Priority: P3)

A user recorded a thought but associated it with the wrong date, or wants to backdate a thought to reflect when something actually occurred. They provide a new date for an existing thought via a CLI command without needing to re-enter the thought's text.

**Why this priority**: Date correction addresses a common user need (backdating or fixing mistaken dates) without requiring full thought re-entry. It is scoped to CLI only, making it simpler to implement.

**Independent Test**: Can be tested by recording a thought with today's date, editing the date to a past date via CLI, and verifying the thought appears with the new date in listings.

**Acceptance Scenarios**:

1. **Given** a thought recorded today, **When** the user runs `wet edit <thought-id> --date 2026-01-15`, **Then** the thought's date is updated to January 15, 2026.
2. **Given** a thought with an existing date, **When** the user edits only the date, **Then** the thought's text and entity associations remain unchanged.
3. **Given** an invalid date format is provided, **When** the user runs the date edit command, **Then** the system displays an informative error and makes no changes.
4. **Given** a valid thought ID and date, **When** the user edits the date, **Then** the system confirms the change.

---

### Edge Cases

- What happens when the user provides empty text as the new thought content?
- How does the system behave when the editor is unavailable or not configured?
- What if editing introduces a reference to an entity name that does not yet exist in the system?
- What if a thought is referenced by or associated with multiple entities and all references are removed during editing?
- What happens when the user provides a new date but the resulting text (unchanged) still refers to entities — are associations re-verified or left as-is?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Users MUST be able to edit the text of an existing thought by providing the thought's numeric ID and new text as CLI arguments.
- **FR-002**: Users MUST be able to edit the text of an existing thought by opening it in an interactive editor, with the current text pre-populated.
- **FR-003**: Users MUST be able to change the date of an existing thought via a CLI argument without re-entering the thought's text.
- **FR-004**: When a thought's text is modified (via CLI or editor), the system MUST automatically recalculate all entity associations based on the new text.
- **FR-005**: Entity references present in the new text but absent from the old text MUST be added as associations after editing.
- **FR-006**: Entity references present in the old text but absent from the new text MUST be removed as associations after editing.
- **FR-007**: Users MUST provide a thought's numeric ID to target it for editing; the system MUST reject edits referencing non-existent IDs with a clear error message.
- **FR-008**: The system MUST reject edit commands that would set a thought's text to empty.
- **FR-009**: When the editor is closed without changes to the content, the system MUST make no modifications and notify the user.
- **FR-014**: When the editor terminates abnormally (crash or force-quit), the system MUST leave the thought unmodified and display a warning to the user.
- **FR-015**: All edit operations (text update, date update, entity association recalculation) MUST be performed atomically; if any step fails, the system MUST roll back all changes and leave the thought in its original state.
- **FR-010**: New entity names introduced via editing that do not yet exist in the system MUST be handled consistently with how entities are created during thought addition.
- **FR-011**: Users MUST receive confirmation after a successful edit operation.
- **FR-012**: The edit command MUST be a subcommand of the `wet` CLI, parallel in structure to the existing thought-addition subcommand.
- **FR-013**: Users MUST be able to edit both the text and the date of a thought in a single command invocation by providing both a new text argument and a `--date` flag simultaneously.

### Key Entities

- **Thought**: A recorded snippet of information with text content, an associated date, and zero or more links to entities. Identified by a numeric ID displayed in listing output. Editable fields: text, date.
- **Entity**: A named item (person, project, concept, etc.) that can be referenced within thought text using inline syntax. Associations are derived from thought text, not stored independently.
- **Entity Reference**: An inline marker within thought text (e.g., `[EntityName]` or `[alias](EntityName)`) that establishes a link between the thought and an entity. References are re-derived whenever thought text changes.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can correct the text of an existing thought in under 30 seconds using the direct CLI command for simple edits (single-line corrections).
- **SC-002**: After editing a thought's text, 100% of entity associations accurately reflect the new content — no stale or missing associations.
- **SC-003**: The editor-based editing experience opens the existing thought content pre-populated, requiring zero manual re-entry of unchanged content.
- **SC-004**: Users can update a thought's date without entering the thought's text at any point, completing the operation in a single command.
- **SC-005**: 100% of entity references introduced or removed during editing are correctly reflected in the thought's entity associations after the edit.
- **SC-006**: A thought is never left in a partially-edited state; any failed edit leaves the thought identical to its state before the command was run.

## Assumptions

- Thoughts have a stable numeric ID that is already displayed in `wet` listing output as `[id]`; no changes to the listing command are required for this feature.
- The date format accepted for `--date` is the same as the format used when adding thoughts.
- Editing a thought's text in the editor follows the same editor-launch mechanism as the existing editor-based thought addition.
- Entity names introduced via editing that don't exist are auto-created, consistent with the existing thought-addition behavior.
- Combining text and date edits in a single CLI invocation is supported; either or both may be provided in one command.
