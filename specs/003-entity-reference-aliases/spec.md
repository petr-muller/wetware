# Feature Specification: Entity Reference Aliases

**Feature Branch**: `003-entity-reference-aliases`
**Created**: 2026-01-31
**Status**: Draft
**Input**: User description: "When entering a thought, allow entity references to be aliased, using markdown-like link syntax [alias](reference), which makes using entity references much more natural. As an example a thought that references 'robotics' entity may refer to it through [robotics] link directly, or it may be natural to refer to it as [robot](robotics). When these thoughts are rendered in the thoughts command, only the respective alias should be rendered (in colored highlight, like when an alias is not used)."

## User Scenarios & Testing

### User Story 1 - Enter Thought with Aliased Entity Reference (Priority: P1)

A user wants to write a thought that references an entity using more natural language. Instead of forcing the exact entity name into the sentence, they can use an alias that fits grammatically while still linking to the correct entity.

**Why this priority**: This is the core functionality that enables the entire feature. Users must be able to input aliased references for the feature to provide any value.

**Independent Test**: Can be fully tested by entering a thought with the syntax `[alias](entity)` and verifying it is stored correctly without errors. Delivers the value of natural language input.

**Acceptance Scenarios**:

1. **Given** an entity "robotics" exists, **When** user enters thought "I learned about [robots](robotics) today", **Then** the thought is saved successfully with the aliased entity reference
2. **Given** an entity "machine-learning" exists, **When** user enters thought "Started [ML](machine-learning) course", **Then** the thought is saved with the aliased reference
3. **Given** user enters thought with both traditional `[robotics]` and aliased `[robot](robotics)` references, **When** thought is saved, **Then** both reference types are accepted in the same thought

---

### User Story 2 - View Thought with Aliased Entity Reference (Priority: P2)

A user views their thoughts and sees aliased entity references displayed with only the alias text (not the full entity name), styled with the same colored highlight as traditional entity references.

**Why this priority**: This ensures the feature fulfills its promise of making thoughts more natural to read. Without proper rendering, the input capability alone doesn't provide the full user value.

**Independent Test**: Can be fully tested by displaying a previously saved thought containing aliased references and verifying only the alias appears in colored highlight. Delivers the value of natural language output.

**Acceptance Scenarios**:

1. **Given** a thought contains `[robot](robotics)` reference, **When** user views the thought, **Then** only "robot" is displayed in colored highlight (not "robotics" or "[robot](robotics)")
2. **Given** a thought contains both `[robotics]` and `[robot](robotics)` references, **When** user views the thought, **Then** both appear with the same colored highlight styling
3. **Given** a thought contains multiple aliased references to different entities, **When** user views the thought, **Then** each alias is displayed in colored highlight with its respective entity color

---

### User Story 3 - Backward Compatibility with Traditional References (Priority: P3)

Existing users continue to use traditional entity reference syntax `[entity]` alongside the new aliased syntax, and both work seamlessly together.

**Why this priority**: This ensures the feature doesn't break existing workflows and allows gradual adoption. It's P3 because if P1 and P2 work correctly, backward compatibility should naturally emerge from the implementation.

**Independent Test**: Can be fully tested by using only traditional `[entity]` syntax in new thoughts and verifying they still work as before. Delivers the value of non-breaking change.

**Acceptance Scenarios**:

1. **Given** an entity "robotics" exists, **When** user enters thought "Reading about [robotics]", **Then** the thought is saved and rendered exactly as before this feature
2. **Given** existing thoughts with traditional references, **When** user views them, **Then** they display correctly without any changes in behavior
3. **Given** user alternates between `[entity]` and `[alias](entity)` syntax across different thoughts, **When** viewing thought history, **Then** both styles render correctly

---

### Edge Cases

- What happens when the referenced entity doesn't exist (e.g., `[robot](nonexistent)`)? - System should show an error indicating the entity does not exist
- What happens when the alias and reference are identical `[robotics](robotics)`? - Should be accepted as valid (user explicitly chose aliased syntax)
- What happens with multi-word aliases like `[my robot project](robotics)`? - Should be accepted and displayed as-is
- What happens with malformed syntax like `[alias](entity][something]` or nested attempts? - System should either parse the first valid reference or reject as malformed
- What happens when a thought contains partial syntax like `[alias](` without closing? - System should show a syntax error

## Requirements

### Functional Requirements

- **FR-001**: System MUST accept entity references using markdown-like link syntax `[alias](reference)` where alias is the display text and reference is the target entity name
- **FR-002**: System MUST accept traditional entity reference syntax `[entity]` to maintain backward compatibility
- **FR-003**: System MUST allow both aliased and traditional reference syntaxes to be used within the same thought
- **FR-004**: System MUST validate that the referenced entity (the part in parentheses) exists when processing an aliased reference
- **FR-005**: When rendering thoughts, system MUST display only the alias text (the part in square brackets) for aliased references
- **FR-006**: System MUST apply the same colored highlight styling to aliased references as traditional entity references
- **FR-007**: System MUST preserve the association between the displayed alias and the actual entity reference for entity-based queries and filtering
- **FR-008**: System MUST trim leading and trailing whitespace from both alias and reference parts (e.g., `[ robot ](robotics)` is treated as `[robot](robotics)`)
- **FR-009**: System MUST reject empty alias `[](entity)` or empty reference `[alias]()` as invalid syntax errors
- **FR-010**: System MUST reject aliases containing brackets `[` `]` or parentheses `(` `)` as invalid syntax to avoid parsing ambiguity

### Key Entities

- **Entity Reference**: Links a thought to an entity. May be expressed in traditional form `[entity]` or aliased form `[alias](entity)`. The alias is the text displayed to the user, while the entity name is used for lookups, filtering, and entity-based queries.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can enter thoughts with aliased entity references that read naturally in context (e.g., grammatically correct sentences)
- **SC-002**: When viewing thoughts, aliased references are visually indistinguishable from traditional references (same colored highlight styling)
- **SC-003**: System correctly identifies and links aliased references to their target entities (entity-based queries return thoughts with both traditional and aliased references to that entity)
- **SC-004**: 100% of existing thoughts with traditional entity references continue to display and function without any changes in behavior
