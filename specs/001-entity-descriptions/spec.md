# Feature Specification: Entity Descriptions

**Feature Branch**: `001-entity-descriptions`
**Created**: 2026-02-01
**Status**: Draft
**Input**: User description: "Allow entities to have a potentially longer form descriptions. A typical descriptions would be something like two or three paragraphs of plain text. The only markup supported in the descriptions would be other entity references, with syntax identical to references in thoughts (either plain or aliased references). For now the descripions would be only used in `entities` command where every entity would be accompanied with ellipsed start of the description (so that eeach entity with its description would fit on a typical single terminal line). References in ellipsized descriptions would be displayed without markup (either with entity name or the used alias) but without the font or color highlight."

## Clarifications

### Session 2026-02-01

- Q: How should users add or edit descriptions for entities? → A: Support all three methods (command argument via `--description`, interactive editor, file input via `--description-file`) using `wet entity edit` command with flags. Descriptions can only be added to existing entities, not during entity creation.
- Q: How should the system handle descriptions that contain only whitespace (spaces, tabs, newlines)? → A: Treat as empty (clear/remove description).
- Q: How should the system handle entity references in descriptions that point to non-existent entities? → A: Auto-create referenced entities that don't exist (consistent with thought behavior).
- Q: How should newlines and multiple paragraphs be handled in the ellipsized preview? → A: Show only the first paragraph (stop at first double newline).
- Q: How should the system handle very narrow terminal widths (less than 80 characters)? → A: Suppress preview below certain width threshold, show only entity name.

## User Scenarios & Testing

### User Story 1 - Add Description to Entity (Priority: P1)

When I have an existing entity, I want to add or update a multi-paragraph description using command arguments, an interactive editor, or a file input, so that I can provide context and detailed information about what that entity represents.

**Why this priority**: This is the core functionality - without the ability to create and store descriptions, none of the other features matter. This is the foundation that enables all other use cases.

**Independent Test**: Can be fully tested by editing an existing entity with a description using `wet entity edit` and verifying the description is stored and can be retrieved, delivering the fundamental capability to document entities.

**Acceptance Scenarios**:

1. **Given** I have an existing entity, **When** I run `wet entity edit <name> --description "two paragraphs of plain text"`, **Then** the description is stored with the entity
2. **Given** I have an existing entity, **When** I run `wet entity edit <name>` and an interactive editor opens for multi-line input, **Then** the description I enter is stored with the entity
3. **Given** I have a description in a file, **When** I run `wet entity edit <name> --description-file desc.txt`, **Then** the file contents are stored as the entity description
4. **Given** I am adding a description with entity references, **When** I use plain syntax (e.g., "Related to @project") or aliased syntax (e.g., "[the main project](@project)"), **Then** the entity reference syntax is preserved exactly as entered

---

### User Story 2 - View Entities with Description Previews (Priority: P2)

When I list all entities using the `entities` command, I want to see the beginning of each entity's description on the same line so that I can quickly understand what each entity is about without navigating to detailed views.

**Why this priority**: This is the primary user-facing feature that makes descriptions useful. It builds on P1 by providing immediate value to users when browsing their entities.

**Independent Test**: Can be fully tested by running the `entities` command and verifying that entities with descriptions show ellipsized previews on a single terminal line, delivering immediate browsing value.

**Acceptance Scenarios**:

1. **Given** an entity has a two-paragraph description, **When** I run the `entities` command, **Then** I see the entity name followed by the start of the description, ellipsized to fit on one terminal line
2. **Given** an entity's description contains entity references, **When** viewing the description preview in `entities` command, **Then** references are displayed as plain text (using entity name or alias) without formatting or highlighting
3. **Given** an entity has no description, **When** I run the `entities` command, **Then** the entity is displayed without a description preview
4. **Given** multiple entities with varying description lengths, **When** I run the `entities` command, **Then** each entity with its description preview fits on a single terminal line

---

### Edge Cases

- Descriptions containing only whitespace (spaces, tabs, newlines) are treated as empty and clear/remove the description
- Extremely short descriptions (one sentence) are displayed as-is in the preview, ellipsized if needed to fit the terminal line
- Entity references in descriptions that point to non-existent entities automatically create those entities (consistent with thought behavior)
- When description starts with an entity reference, the preview displays it as plain text (entity name or alias) followed by remaining description text
- Very narrow terminal widths (less than 80 characters): preview is suppressed below a certain width threshold, showing only entity name
- Newlines and multiple paragraphs in descriptions: preview shows only the first paragraph (stops at first double newline), then ellipsizes to fit terminal line

## Requirements

### Functional Requirements

- **FR-001**: System MUST allow entities to store descriptions consisting of multiple paragraphs of plain text, with no maximum length restriction
- **FR-002**: System MUST support entity references within descriptions using both plain syntax (e.g., "@entity") and aliased syntax (e.g., "[alias](@entity)")
- **FR-003**: System MUST preserve entity reference syntax exactly as entered when storing descriptions (no modification or normalization)
- **FR-003a**: System MUST automatically create any entities referenced in descriptions that don't already exist (consistent with thought reference behavior)
- **FR-004**: System MUST provide `wet entity edit <entity-name>` command for adding/updating descriptions on existing entities only (not during entity creation)
- **FR-005**: System MUST support three input methods for descriptions: (a) inline via `--description "text"` flag, (b) interactive editor when no flags provided, (c) file input via `--description-file <path>` flag
- **FR-005a**: System MUST treat descriptions containing only whitespace (spaces, tabs, newlines) as empty and clear/remove the description from the entity
- **FR-006**: The `entities` command MUST display an ellipsized preview of each entity's description when a description exists
- **FR-006a**: Description previews MUST show only the first paragraph of the description (stopping at the first double newline) before ellipsizing to fit the terminal line
- **FR-007**: Description previews in the `entities` command MUST be formatted to fit on a single terminal line alongside the entity name, assuming a minimum terminal width of 80 characters
- **FR-007a**: System MUST suppress description previews when terminal width falls below a threshold (suggested: 60 characters), displaying only entity names to ensure readability
- **FR-008**: Entity references within description previews MUST be displayed as plain text without font styling or color highlighting
- **FR-009**: Entity references in previews MUST show either the entity name (for plain references) or the alias (for aliased references)
- **FR-010**: System MUST display entities without descriptions in the `entities` command without showing a preview or placeholder text

### Key Entities

- **Entity Description**: A multi-paragraph text field associated with an entity, supporting plain text and entity references (both plain and aliased syntax). Typical length is 2-3 paragraphs.
- **Entity Reference in Description**: An inline reference to another entity within a description, using the same syntax as thought references. Can be plain (@entity) or aliased ([alias](@entity)).

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can add descriptions of any length to entities, with typical descriptions being 300-500 words (2-3 paragraphs)
- **SC-002**: 100% of entity references in description previews are displayed as plain text without styling, showing the correct entity name or alias
- **SC-003**: Every entity listing in the `entities` command output fits on a single terminal line when viewed on an 80-character wide terminal
- **SC-004**: Users can identify and differentiate entities by reading description previews without needing to view full descriptions, as measured by the preview containing sufficient context (at least 40-60 characters of description text)

## Assumptions

- Entity reference syntax in descriptions is identical to thought reference syntax (as confirmed by user input)
- "Typical" descriptions are 2-3 paragraphs, but the system should support any length
- A "typical single terminal line" assumes standard terminal width (80-120 characters)
- Ellipsization should be smart enough to preserve readability (e.g., cutting at word boundaries)
- The `wet entity edit` command uses standard CLI patterns for flags and interactive editor behavior
