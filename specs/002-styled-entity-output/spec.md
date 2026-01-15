# Feature Specification: Styled Entity Output

**Feature Branch**: `002-styled-entity-output`
**Created**: 2026-01-15
**Status**: Draft
**Input**: User description: "When the user runs any form of wet thoughts (filtered or not), the entities should be rendered without the entry markup. The entities should be nicely styled, preferably bold and colorful by default, but it should be possible to disable the styling. The tool should also detect whether the output is an interactive terminal or not and disable styled output when not, unless explicitly enabled. In each execution, any single entity should use exactly the same color throughout the whole output. The color can differ between executions, and multiple entities may share a color, but ideally that should only happen when there is too many different entities (I would say there could be 10-15 different colors)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Styled Thoughts in Terminal (Priority: P1)

As a user running `wet thoughts` in an interactive terminal, I want entities to be displayed with bold text and distinct colors so that I can quickly scan and identify different entities visually.

**Why this priority**: This is the core feature that provides immediate visual value to users reviewing their thoughts. Bold, colorful entities make the output more scannable and help users quickly identify patterns and relationships.

**Independent Test**: Can be fully tested by running `wet thoughts` in a terminal and verifying entities appear bold and colored, with each unique entity maintaining the same color throughout the output.

**Acceptance Scenarios**:

1. **Given** a user has thoughts containing multiple entities, **When** they run `wet thoughts` in an interactive terminal, **Then** each entity is displayed in bold with a color, and the same entity uses the same color throughout the output.
2. **Given** a user has thoughts with 5 different entities, **When** they run `wet thoughts`, **Then** each entity has a distinct color (no color sharing when under the color limit).
3. **Given** a user has thoughts with 20 different entities, **When** they run `wet thoughts`, **Then** entities are colored using available colors with some entities sharing colors as needed.

---

### User Story 2 - Clean Entity Display Without Markup (Priority: P1)

As a user viewing thoughts, I want entities to be displayed cleanly without their internal markup syntax so that the output is human-readable.

**Why this priority**: Removing entry markup is essential for readable output. Users should see entity names clearly, not internal syntax that was used when creating thoughts.

**Independent Test**: Can be fully tested by running `wet thoughts` with thoughts containing entities and verifying entities appear as clean text without surrounding markup characters.

**Acceptance Scenarios**:

1. **Given** a thought contains an entity with markup syntax, **When** the user runs `wet thoughts`, **Then** the entity is displayed without the markup characters.
2. **Given** multiple thoughts contain the same entity, **When** displayed, **Then** all instances show the clean entity name consistently.

---

### User Story 3 - Automatic Plain Output for Non-Interactive Use (Priority: P2)

As a user piping `wet thoughts` output to another program or redirecting to a file, I want styling to be automatically disabled so that the output is clean and parseable without escape codes.

**Why this priority**: Automatic TTY detection ensures the tool works correctly in scripts and pipelines without requiring users to remember to disable styling manually.

**Independent Test**: Can be fully tested by running `wet thoughts | cat` or `wet thoughts > file.txt` and verifying no ANSI escape codes appear in the output.

**Acceptance Scenarios**:

1. **Given** a user pipes `wet thoughts` output to another command, **When** the command executes, **Then** the output contains no ANSI escape codes or styling.
2. **Given** a user redirects `wet thoughts` output to a file, **When** the file is created, **Then** the file contains plain text without escape codes.

---

### User Story 4 - Explicit Styling Control (Priority: P3)

As a user, I want to explicitly enable or disable styling regardless of terminal detection so that I have full control over the output format.

**Why this priority**: Power users and specific use cases may need to override automatic behavior - forcing colors in CI logs that support them, or ensuring plain output even in a terminal.

**Independent Test**: Can be fully tested by running `wet thoughts` with explicit flags and verifying styling is enabled/disabled as requested regardless of terminal type.

**Acceptance Scenarios**:

1. **Given** a user runs `wet thoughts` with styling explicitly disabled, **When** in an interactive terminal, **Then** the output contains no styling (plain text).
2. **Given** a user runs `wet thoughts` with styling explicitly enabled, **When** piping to another command, **Then** the output contains ANSI styling codes.
3. **Given** a user runs `wet thoughts` with styling explicitly enabled, **When** redirecting to a file, **Then** the file contains ANSI escape codes.

---

### Edge Cases

- What happens when there are exactly 0 entities in the output? Output displays normally without any colored elements.
- What happens when there are more entities than available colors (e.g., 25+ entities)? Colors are reused, with the system distributing colors as evenly as possible among entities.
- What happens when an entity name contains special characters? The entity is displayed with its special characters intact, just without markup and with styling applied.
- What happens when the terminal doesn't support colors? The system should detect this and fall back to plain output (or bold-only if supported).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render entities without their entry markup syntax in all `wet thoughts` output.
- **FR-002**: System MUST apply bold styling to entities by default when output is to an interactive terminal.
- **FR-003**: System MUST apply color styling to entities by default when output is to an interactive terminal.
- **FR-004**: System MUST assign the same color to the same entity throughout a single command execution.
- **FR-005**: System MUST provide 10-15 distinct colors for entity styling.
- **FR-006**: System MUST reuse colors only when the number of distinct entities exceeds the available color count.
- **FR-007**: System MUST automatically detect whether output is to an interactive terminal (TTY detection).
- **FR-008**: System MUST disable styling automatically when output is not to an interactive terminal.
- **FR-009**: System MUST provide a way to explicitly disable styling regardless of terminal detection.
- **FR-010**: System MUST provide a way to explicitly enable styling regardless of terminal detection.
- **FR-011**: System MUST apply styling behavior consistently across all `wet thoughts` subcommands (filtered and unfiltered).

### Key Entities

- **Entity**: A named reference within a thought (e.g., a person, project, concept) that should be visually distinguished in output.
- **Color Assignment**: A mapping between an entity and its display color for a single execution, ensuring consistency throughout the output.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can visually distinguish entities from regular text at a glance when viewing styled output.
- **SC-002**: Each unique entity maintains consistent visual appearance (same color) throughout a single output.
- **SC-003**: Output piped to files or other programs contains no escape codes unless explicitly requested.
- **SC-004**: Users can override automatic styling detection with a single command-line option.
- **SC-005**: At least 10 distinct colors are available for entity differentiation.

## Assumptions

- The terminal color support detection will rely on standard mechanisms (environment variables, TTY detection).
- Color assignment within a single execution does not need to be deterministic across executions (the same entity may get different colors on different runs).
- The "entry markup" refers to the syntax used to denote entities when creating thoughts (the specific syntax is defined in the existing codebase).
- Bold styling is supported by virtually all modern terminals that support colors.
