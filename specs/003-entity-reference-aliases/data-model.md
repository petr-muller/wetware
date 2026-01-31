# Data Model: Entity Reference Aliases

**Feature**: 003-entity-reference-aliases
**Date**: 2026-01-31
**Status**: Design

## Overview

This document defines the data model for entity reference aliases, extending the existing `[entity]` syntax to support markdown-like `[alias](entity)` syntax while maintaining full backward compatibility.

## Key Concepts

### Entity Reference

An **Entity Reference** links a thought to an entity. It can be expressed in two forms:

1. **Traditional Form**: `[entity]` - Display text and target entity are identical
2. **Aliased Form**: `[alias](entity)` - Display text (alias) differs from target entity

Both forms:
- Display text in colored highlight when rendering
- Link to an entity for queries and filtering
- Support case-insensitive entity matching
- Trim leading/trailing whitespace

## Data Structures

### EntityMatch (Conceptual)

Represents a parsed entity reference extracted from thought content.

```rust
/// Represents a matched entity reference in thought text
pub struct EntityMatch {
    /// Text to display to the user (entity name or alias)
    pub display_text: String,

    /// Name of the target entity (for lookups and filtering)
    pub target_entity: String,

    /// Position in the original text (start, end)
    pub span: (usize, usize),
}
```

**Invariants**:
- Both `display_text` and `target_entity` are non-empty after trimming
- `display_text` contains no brackets `[`, `]` or parentheses `(`, `)`
- `target_entity` must reference an existing entity in the database
- Whitespace is trimmed from both fields

**Construction**:
```rust
impl EntityMatch {
    /// Create from traditional syntax [entity]
    pub fn traditional(entity: &str, span: (usize, usize)) -> Result<Self, ValidationError> {
        let trimmed = entity.trim();
        if trimmed.is_empty() {
            return Err(ValidationError::EmptyEntity);
        }

        Ok(Self {
            display_text: trimmed.to_string(),
            target_entity: trimmed.to_string(),
            span,
        })
    }

    /// Create from aliased syntax [alias](entity)
    pub fn aliased(alias: &str, entity: &str, span: (usize, usize))
        -> Result<Self, ValidationError>
    {
        let alias_trimmed = alias.trim();
        let entity_trimmed = entity.trim();

        if alias_trimmed.is_empty() {
            return Err(ValidationError::EmptyAlias);
        }
        if entity_trimmed.is_empty() {
            return Err(ValidationError::EmptyEntity);
        }
        if alias_trimmed.contains(['[', ']', '(', ')']) {
            return Err(ValidationError::InvalidAliasCharacters {
                alias: alias_trimmed.to_string()
            });
        }

        Ok(Self {
            display_text: alias_trimmed.to_string(),
            target_entity: entity_trimmed.to_string(),
            span,
        })
    }
}
```

## Validation Rules

### Alias Validation

An alias is valid if:
1. **Non-empty** after trimming whitespace
2. **No special characters**: Must not contain `[`, `]`, `(`, `)`
3. **Length limit**: Inherits from entity name limit (database CHECK constraint)

**Valid aliases**:
- `robot` → OK
- `ML project` → OK (multi-word)
- `C++` → OK (special chars except brackets/parens)
- `my-robot-2024` → OK (hyphens, numbers)

**Invalid aliases**:
- ` ` (whitespace-only) → Error: EmptyAlias
- `[test]` → Error: InvalidAliasCharacters
- `alias(1)` → Error: InvalidAliasCharacters
- `` (empty) → Error: EmptyAlias

### Entity Reference Validation

An entity reference is valid if:
1. **Non-empty** after trimming whitespace
2. **Entity exists** in the database
3. **Length limit**: Respects database CHECK constraint

**Valid references**:
- `robotics` → OK (entity exists)
- `machine-learning` → OK (entity exists)

**Invalid references**:
- ` ` (whitespace-only) → Error: EmptyEntity
- `nonexistent` → Error: EntityNotFound
- `` (empty) → Error: EmptyEntity

### Whitespace Handling

**Rule**: Trim leading and trailing whitespace from both alias and entity reference.

**Examples**:
```rust
"[ robot ]( robotics )" → EntityMatch {
    display_text: "robot",
    target_entity: "robotics"
}

"[  ML  ](machine-learning)" → EntityMatch {
    display_text: "ML",
    target_entity: "machine-learning"
}

"[entity]" → EntityMatch {
    display_text: "entity",
    target_entity: "entity"
}
```

## State Diagram

Entity reference processing follows this state flow:

```
┌─────────────────┐
│ Raw Thought Text│
│ "I learned about│
│ [robots](      │
│  robotics)"    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Regex Matching  │
│ Pattern:        │
│ \[([^\[\]]+)\]  │
│ (?:\(([^\(\)]+)\))?│
└────────┬────────┘
         │
         ▼
    ┌────┴────┐
    │ Match?  │
    └────┬────┘
         │
    ┌────┴────────────┐
    │                 │
    ▼                 ▼
┌───────┐      ┌─────────────┐
│  No   │      │   Yes       │
│ Match │      │ Captured:   │
└───┬───┘      │ Group 1: alias│
    │          │ Group 2: entity?│
    │          └─────┬───────┘
    │                │
    ▼                ▼
┌───────────┐  ┌────┴─────────┐
│ Plain     │  │ Validate &   │
│ Text      │  │ Trim         │
│ (ignore)  │  │ - Empty?     │
└───────────┘  │ - Special?   │
               │ - Whitespace │
               └─────┬────────┘
                     │
                ┌────┴────┐
                │ Valid?  │
                └────┬────┘
                     │
          ┌──────────┴──────────┐
          ▼                     ▼
    ┌──────────┐         ┌──────────┐
    │  Error   │         │   OK     │
    │ Return   │         │ Create   │
    │ Result   │         │EntityMatch│
    └──────────┘         └─────┬────┘
                               │
                               ▼
                        ┌──────────────┐
                        │ Group 2      │
                        │ exists?      │
                        └──────┬───────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
              ┌───────────┐         ┌──────────┐
              │ Aliased   │         │Traditional│
              │ display=G1│         │display=G1 │
              │ target=G2 │         │target=G1  │
              └─────┬─────┘         └─────┬─────┘
                    │                     │
                    └──────────┬──────────┘
                               ▼
                        ┌──────────────┐
                        │ Rendering:   │
                        │ Display      │
                        │ display_text │
                        │ with color   │
                        │ from         │
                        │ target_entity│
                        └──────────────┘
```

## Storage Considerations

### No Database Changes Required

**Current Schema** (from migrations):
```sql
CREATE TABLE thoughts (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    CHECK(length(trim(content)) > 0 AND length(content) <= 10000)
);

CREATE TABLE entities (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    CHECK(length(trim(name)) > 0)
);
```

**Design Decision**: Store raw content with markup

The thought content is stored exactly as entered by the user:
- `"I learned about [robots](robotics)"` → Stored as-is
- `"Reading about [robotics]"` → Stored as-is

**Rationale**:
1. **No migration needed** - Existing thoughts continue to work
2. **Alias preserved** - User's exact wording is maintained
3. **Runtime extraction** - Entity references extracted during display/query
4. **Backward compatible** - Both syntaxes stored identically (as text)

### Entity Extraction at Runtime

Entity names are extracted when:
1. **Displaying thoughts** - For syntax highlighting
2. **Querying by entity** - To find all thoughts referencing an entity
3. **Validating references** - To ensure entity exists

**Extraction behavior**:
- Traditional `[robotics]` → Extracts "robotics"
- Aliased `[robot](robotics)` → Extracts "robotics" (NOT "robot")
- Mixed `[robotics] and [robot](robotics)` → Extracts ["robotics", "robotics"] → Deduplicated to ["robotics"]

**Query implications**:
```sql
-- This query continues to work (content-based search)
SELECT * FROM thoughts WHERE content LIKE '%[robotics]%' OR content LIKE '%](robotics)%';
```

But runtime extraction is preferred for accurate results.

## Rendering Model

### Display vs. Target

When rendering a thought:

**Display Text**: What the user sees (with colored highlight)
- Traditional: entity name (e.g., "robotics")
- Aliased: alias text (e.g., "robot")

**Target Entity**: What determines the color and query matching
- Traditional: same as display text (e.g., "robotics")
- Aliased: entity reference (e.g., "robotics")

### Color Assignment

Colors are assigned based on **target entity**, not display text:

```rust
// Pseudocode for rendering
for match in entity_matches {
    let color = get_color(&match.target_entity); // Color by target
    let styled = match.display_text.color(color).bold(); // Display alias
    output.push_str(&styled);
}
```

**Example**:
```
Thought: "I learned about [robots](robotics) and [robotics]"

Rendering:
- "robots" → Colored by "robotics" entity → e.g., CYAN
- "robotics" → Colored by "robotics" entity → e.g., CYAN
- Both have SAME color (same target entity)
```

## Error Model

### ValidationError

Represents validation failures during entity reference parsing.

```rust
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ValidationError {
    #[error("Entity alias cannot be empty")]
    EmptyAlias,

    #[error("Entity name cannot be empty")]
    EmptyEntity,

    #[error("Entity alias cannot contain brackets or parentheses: '{alias}'")]
    InvalidAliasCharacters { alias: String },

    #[error("Entity '{entity}' not found")]
    EntityNotFound { entity: String },

    #[error("Malformed entity reference syntax: {details}")]
    MalformedSyntax { details: String },
}
```

**Error Handling Strategy**:
- Validation errors are non-fatal for thought creation
- Malformed syntax is ignored (treated as plain text)
- This matches current behavior: `[unclosed` doesn't block thought creation

## Comparison: Traditional vs. Aliased

| Aspect            | Traditional `[entity]`    | Aliased `[alias](entity)`      |
|-------------------|---------------------------|--------------------------------|
| **Syntax**        | `[robotics]`              | `[robot](robotics)`            |
| **Display text**  | "robotics"                | "robot"                        |
| **Target entity** | "robotics"                | "robotics"                     |
| **Storage**       | Stored as-is in content   | Stored as-is in content        |
| **Extraction**    | Returns "robotics"        | Returns "robotics"             |
| **Rendering**     | Shows "robotics" in color | Shows "robot" in color         |
| **Color**         | Based on "robotics"       | Based on "robotics"            |
| **Query match**   | Matches entity "robotics" | Matches entity "robotics"      |
| **Validation**    | Entity must exist         | Entity must exist, alias valid |

## Examples

### Example 1: Traditional Syntax Only

**Input**: `"Meeting with [Sarah] about [project-alpha]"`

**Parsed**:
```rust
vec![
    EntityMatch { display_text: "Sarah", target_entity: "Sarah", span: (13, 20) },
    EntityMatch { display_text: "project-alpha", target_entity: "project-alpha", span: (27, 43) },
]
```

**Rendered**: `"Meeting with Sarah about project-alpha"` (with colors)

**Extracted entities**: `["Sarah", "project-alpha"]`

### Example 2: Aliased Syntax Only

**Input**: `"Started [ML](machine-learning) course on [AI](artificial-intelligence)"`

**Parsed**:
```rust
vec![
    EntityMatch { display_text: "ML", target_entity: "machine-learning", span: (8, 31) },
    EntityMatch { display_text: "AI", target_entity: "artificial-intelligence", span: (42, 73) },
]
```

**Rendered**: `"Started ML course on AI"` (with colors)

**Extracted entities**: `["machine-learning", "artificial-intelligence"]`

### Example 3: Mixed Syntax

**Input**: `"[robotics] intro with [robot](robotics) examples"`

**Parsed**:
```rust
vec![
    EntityMatch { display_text: "robotics", target_entity: "robotics", span: (0, 10) },
    EntityMatch { display_text: "robot", target_entity: "robotics", span: (22, 40) },
]
```

**Rendered**: `"robotics intro with robot examples"` (both same color)

**Extracted entities**: `["robotics"]` (deduplicated)

### Example 4: Validation Errors

**Input**: `"Testing [](empty) and [alias with (parens)](entity)"`

**Parsed**:
```rust
// First reference: [](empty)
Err(ValidationError::EmptyAlias)
// Treated as plain text

// Second reference: [alias with (parens)](entity)
Err(ValidationError::InvalidAliasCharacters {
    alias: "alias with (parens)".to_string()
})
// Treated as plain text
```

**Rendered**: `"Testing [](empty) and [alias with (parens)](entity)"` (no highlighting)

**Extracted entities**: `[]` (empty)

## Future Considerations

### Not Included in This Feature

The following are explicitly **NOT** part of this feature:

1. **Alias-based queries**: Cannot query "show thoughts where alias was 'robot'"
2. **Alias storage**: Aliases are not stored in a separate table
3. **Alias history**: Cannot track which aliases were used for an entity
4. **Alias suggestions**: No autocomplete for aliases
5. **Multi-alias support**: Cannot have multiple aliases per entity in schema

**Rationale**: The spec (spec.md) focuses on natural language readability during input/output, not on creating a managed alias system. These features could be added later if needed.

### Potential Extensions

If future requirements emerge:

1. **Alias Analytics**: Store alias → entity mappings for usage tracking
2. **Alias Validation**: Suggest common aliases when creating references
3. **Alias Normalization**: Define canonical aliases for common entities
4. **Alias Search**: Enable searching by alias text

These would require:
- Schema changes (new `entity_aliases` table)
- Migration strategy
- Additional validation logic

## Summary

The data model for entity reference aliases:

1. **Extends existing pattern** - No breaking changes to schema or API
2. **Runtime extraction** - Entity references parsed from content on demand
3. **Validation at parse time** - Errors caught during extraction, not storage
4. **Backward compatible** - Traditional syntax continues to work exactly as before
5. **Simple implementation** - Reuses existing color assignment and styling logic
6. **User-focused** - Display what user wrote (alias), query by what they meant (entity)

The design prioritizes simplicity, backward compatibility, and user experience over complex alias management features.
