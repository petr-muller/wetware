# Data Model: Styled Entity Output

**Feature Branch**: `002-styled-entity-output`
**Date**: 2026-01-15

## Overview

This feature does not introduce persistent data model changes. All data structures are runtime-only for the duration of a single command execution.

## Runtime Data Structures

### ColorMode (Enum)

Controls color output behavior based on user preference and environment detection.

```
ColorMode
├── Always    # Force colors regardless of terminal
├── Auto      # Detect terminal and use colors when appropriate (default)
└── Never     # Never use colors
```

**Fields**: N/A (unit variants)

**Validation**: N/A (clap ValueEnum handles parsing)

**State Transitions**: Set once at CLI argument parsing, immutable during execution

### EntityColorMap (Runtime Cache)

Maps entity names to assigned colors for consistent rendering within a single execution.

```
EntityColorMap
├── entries: Map<EntityName, Color>
└── next_color_index: Integer
```

**Fields**:
- `entries`: Hash map from lowercase entity name to assigned terminal color
- `next_color_index`: Counter for round-robin color assignment when entities exceed palette size

**Relationships**:
- References entities extracted by existing `entity_parser` service
- Uses colors from 12-color ANSI palette

**Validation**:
- Entity names normalized to lowercase for consistent lookup
- Color index wraps around using modulo when palette exhausted

**State Transitions**:
```
Empty → Populated (on first entity encounter)
Populated → Populated (additional entities assigned colors)
```

### StyledSegment (Output Unit)

Represents a segment of thought content with optional styling.

```
StyledSegment
├── text: String
├── is_entity: Boolean
└── color: Option<Color>
```

**Fields**:
- `text`: The display text (entity name without brackets, or plain text)
- `is_entity`: Whether this segment represents an entity
- `color`: Assigned color when `is_entity` is true and styling enabled

**Relationships**:
- Derived from thought content parsing
- Color looked up from EntityColorMap

## Entity Relationships

```
Thought (existing)
    │
    ├──extracts──> Entity references (existing)
    │                    │
    │                    └──maps to──> EntityColorMap entry
    │
    └──renders as──> List<StyledSegment>
                          │
                          └──uses──> ColorMode for styling decision
```

## Existing Entities (Unchanged)

### Thought
- No changes to persistence model
- `content` field still stores raw markup syntax
- Rendering extracts entities at display time

### Entity
- No changes to persistence model
- `canonical_name` used for display
- `name` (lowercase) used for color map lookup

## Color Assignment Algorithm

1. Parse thought content into segments (text and entity references)
2. For each entity reference:
   a. Normalize name to lowercase
   b. If name exists in EntityColorMap, use assigned color
   c. If name is new:
      - Assign color at `next_color_index % PALETTE_SIZE`
      - Increment `next_color_index`
      - Store mapping in EntityColorMap
3. Render each segment with appropriate styling based on ColorMode

## No Persistence Changes

This feature modifies only the output rendering layer. No database schema changes, migrations, or persistent data structure modifications are required.
