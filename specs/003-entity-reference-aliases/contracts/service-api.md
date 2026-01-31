# Service API Contracts: Entity Reference Aliases

**Feature**: 003-entity-reference-aliases
**Date**: 2026-01-31
**Status**: Design

## Overview

This document defines the service layer API contracts for entity reference alias support. These are internal APIs within the wetware application, not external REST/GraphQL endpoints.

## Entity Parser Service

Module: `src/services/entity_parser.rs`

### extract_entities

Extracts entity names from thought text, supporting both traditional and aliased syntax.

#### Function Signature

```rust
pub fn extract_entities(text: &str) -> Vec<String>
```

#### Behavior

**Current Behavior** (before this feature):
- Matches traditional syntax: `[entity]`
- Returns entity names
- Trims whitespace from entity names
- Ignores malformed syntax

**New Behavior** (with this feature):
- Matches traditional syntax: `[entity]`
- Matches aliased syntax: `[alias](entity)`
- For traditional syntax: returns entity name
- For aliased syntax: returns **target entity** (not alias)
- Trims whitespace from both alias and entity
- Ignores malformed syntax

#### Examples

```rust
// Traditional syntax (unchanged behavior)
let entities = extract_entities("Meeting with [Sarah]");
assert_eq!(entities, vec!["Sarah"]);

// Aliased syntax (new behavior)
let entities = extract_entities("Started [ML](machine-learning) course");
assert_eq!(entities, vec!["machine-learning"]); // Returns target, not alias

// Mixed syntax
let entities = extract_entities("[robotics] and [robot](robotics)");
assert_eq!(entities, vec!["robotics", "robotics"]); // Both extract same entity

// Whitespace trimming (both syntaxes)
let entities = extract_entities("[ Sarah ] and [ ML ]( machine-learning )");
assert_eq!(entities, vec!["Sarah", "machine-learning"]);

// No matches
let entities = extract_entities("Just plain text");
assert_eq!(entities, vec![]);

// Malformed syntax ignored
let entities = extract_entities("[alias]( unclosed");
assert_eq!(entities, vec!["alias"]); // Treats as traditional syntax

let entities = extract_entities("[](empty) and [valid]");
assert_eq!(entities, vec!["valid"]); // Empty alias ignored
```

#### Edge Cases

| Input                            | Output                   | Reason                                  |
|----------------------------------|--------------------------|-----------------------------------------|
| `[entity]`                       | `["entity"]`             | Traditional syntax                      |
| `[alias](entity)`                | `["entity"]`             | Aliased syntax → returns target         |
| `[alias](entity1) and [entity1]` | `["entity1", "entity1"]` | Both extract same entity                |
| `[  spaced  ]`                   | `["spaced"]`             | Whitespace trimmed                      |
| `[ alias ]( entity )`            | `["entity"]`             | Whitespace trimmed from both            |
| `[]`                             | `[]`                     | Empty content → no match                |
| `[](entity)`                     | `[]`                     | Empty alias → no match                  |
| `[alias]()`                      | `["alias"]`              | Empty entity → treated as traditional   |
| `[alias](`                       | `["alias"]`              | Unclosed paren → treated as traditional |
| `[entity] (text)`                | `["entity"]`             | Space breaks aliased syntax             |
| `unclosed [entity`               | `[]`                     | Malformed → no match                    |

#### Backward Compatibility

**Guarantee**: All existing thoughts with traditional `[entity]` syntax will extract entities identically to the current implementation.

**Test Coverage**:
- All 14 existing tests in entity_parser.rs must pass
- Traditional syntax extraction unchanged
- Case-insensitive deduplication unchanged

### extract_unique_entities

Extracts unique entity names with case-insensitive deduplication.

#### Function Signature

```rust
pub fn extract_unique_entities(text: &str) -> Vec<String>
```

#### Behavior

**Current Behavior** (before this feature):
- Calls `extract_entities()`
- Deduplicates case-insensitively
- Preserves first occurrence capitalization
- Maintains order

**New Behavior** (with this feature):
- No change in behavior
- Works with both traditional and aliased syntax
- Deduplicates based on **target entity**, not alias

#### Examples

```rust
// Traditional syntax (unchanged)
let unique = extract_unique_entities("[Sarah] and [sarah] and [SARAH]");
assert_eq!(unique, vec!["Sarah"]); // First capitalization preserved

// Aliased syntax (new)
let unique = extract_unique_entities("[robot](robotics) and [robotics]");
assert_eq!(unique, vec!["robotics"]); // Deduplicated by target entity

// Mixed case aliases
let unique = extract_unique_entities("[ML](machine-learning) and [ml](Machine-Learning)");
assert_eq!(unique, vec!["machine-learning"]); // Case-insensitive dedup of target

// Order preservation
let unique = extract_unique_entities("[Z] [A] [M] [a]");
assert_eq!(unique, vec!["Z", "A", "M"]); // First occurrence of each entity
```

#### Edge Cases

| Input                               | Output                   | Reason                                        |
|-------------------------------------|--------------------------|-----------------------------------------------|
| `[robotics] [robot](robotics)`      | `["robotics"]`           | Same target entity → deduplicated             |
| `[A](entity) [B](entity)`           | `["entity"]`             | Different aliases, same target → deduplicated |
| `[alias](entity1) [alias](entity2)` | `["entity1", "entity2"]` | Same alias, different targets → both kept     |
| `[Entity] [entity]`                 | `["Entity"]`             | Case-insensitive dedup of traditional         |
| `[Alias](Entity) [alias](entity)`   | `["Entity"]`             | Case-insensitive dedup of aliased             |

## Entity Styler Service

Module: `src/services/entity_styler.rs`

### EntityStyler::render_content

Renders thought content with styled entity references.

#### Function Signature

```rust
impl EntityStyler {
    pub fn render_content(&mut self, content: &str) -> String;
}
```

#### Behavior

**Current Behavior** (before this feature):
- Finds `[entity]` patterns
- Removes bracket markup
- Applies bold + color styling (if use_colors=true)
- Assigns consistent colors per entity

**New Behavior** (with this feature):
- Finds `[entity]` and `[alias](entity)` patterns
- Removes markup (brackets and parentheses)
- For traditional: displays entity name
- For aliased: displays **alias** (not entity name)
- Colors based on **target entity** (not alias)
- Consistent color for same entity regardless of alias

#### Examples

```rust
// Traditional syntax (unchanged)
let mut styler = EntityStyler::new(false); // plain mode
let output = styler.render_content("Meeting with [Sarah]");
assert_eq!(output, "Meeting with Sarah");

// Aliased syntax (new)
let mut styler = EntityStyler::new(false);
let output = styler.render_content("Started [ML](machine-learning) course");
assert_eq!(output, "Started ML course"); // Shows alias, not entity

// Mixed syntax
let mut styler = EntityStyler::new(false);
let output = styler.render_content("[robotics] and [robot](robotics)");
assert_eq!(output, "robotics and robot"); // Each shows its display text

// Color mode (same entity = same color)
let mut styler = EntityStyler::new(true);
let output = styler.render_content("[robotics] and [robot](robotics)");
// Both "robotics" and "robot" are colored the same (e.g., CYAN)
// because they reference the same target entity

// Whitespace trimming
let mut styler = EntityStyler::new(false);
let output = styler.render_content("[  Sarah  ] and [ ML ]( machine-learning )");
assert_eq!(output, "Sarah and ML"); // Whitespace trimmed, markup removed
```

#### Color Assignment

**Rule**: Colors are assigned based on **target entity**, not display text (alias).

```rust
// Pseudocode for color assignment logic
fn render_with_color(&mut self, capture: Capture) -> String {
    let alias = capture[1].trim(); // Display text
    let entity = capture.get(2).map(|r| r.as_str().trim()).unwrap_or(alias); // Target

    let color = self.get_color(entity); // Color by TARGET entity
    alias.bold().color(color).to_string() // Display ALIAS
}
```

**Examples**:

| Input                                 | Display Text            | Color Based On     | Result           |
|---------------------------------------|-------------------------|--------------------|------------------|
| `[Sarah]`                             | "Sarah"                 | "Sarah"            | Sarah in color X |
| `[ML](machine-learning)`              | "ML"                    | "machine-learning" | ML in color Y    |
| `[robot](robotics)`                   | "robot"                 | "robotics"         | robot in color Z |
| `[robotics]` then `[robot](robotics)` | "robotics" then "robot" | both "robotics"    | Both in color Z  |

#### Edge Cases

| Input                  | Plain Output      | Notes                                  |
|------------------------|-------------------|----------------------------------------|
| `[entity]`             | `"entity"`        | Traditional syntax                     |
| `[alias](entity)`      | `"alias"`         | Shows alias, not entity                |
| `[robotics](robotics)` | `"robotics"`      | Alias = entity OK                      |
| `[]`                   | `"[]"`            | Empty brackets → no match, shown as-is |
| `[](entity)`           | `"[](entity)"`    | Empty alias → no match, shown as-is    |
| `[alias]()`            | `"alias"`         | Empty entity → treated as traditional  |
| `[alias](`             | `"alias"`         | Unclosed → treated as traditional      |
| `[alias](entity)[x]`   | `"aliasx"`        | Two separate matches                   |
| `[entity] (text)`      | `"entity (text)"` | Space → breaks aliased syntax          |
| `[  spaced  ]`         | `"spaced"`        | Whitespace trimmed                     |

#### Backward Compatibility

**Guarantee**: All existing thoughts with traditional `[entity]` syntax will render identically to current implementation (aside from ANSI escape sequences).

**Test Coverage**:
- All existing entity_styler tests must pass
- Color consistency maintained
- Bracket stripping unchanged for traditional syntax
- Case-insensitive color mapping unchanged

## Error Handling

### Current Error Handling

Currently, entity parser and styler do not return errors:
- Malformed syntax is silently ignored
- Invalid patterns treated as plain text
- Extraction returns empty vec for no matches

### New Error Handling (Optional Enhancement)

**Option 1: Silent Degradation (Recommended)**
- Continue current behavior
- Malformed aliased syntax treated as plain text or traditional syntax
- No user-facing errors

**Option 2: Warning Logging**
- Log warnings for malformed syntax (but don't fail)
- Helps debugging without blocking user workflow

**Option 3: Strict Validation**
- Return `Result<Vec<String>, ValidationError>` from extract_entities
- Thought creation fails if entity references are invalid
- **Not recommended**: breaks backward compatibility and user experience

**Recommendation**: Use Option 1 (silent degradation) to match current behavior and maintain backward compatibility.

### Validation at Higher Layers

Validation should happen at higher layers if needed:
- CLI layer: Validate entity exists before saving thought
- Display layer: Show warning icon for non-existent entities
- Not in service layer: Keep parsing pure and forgiving

## API Evolution Strategy

### Phase 1: Internal Changes (This Feature)

- Update regex pattern in entity_parser.rs
- Modify extraction logic to handle capture group 2
- Update rendering logic in entity_styler.rs
- Maintain existing function signatures
- Backward compatible

### Phase 2: Optional Enhancements (Future)

If needed, could add:

```rust
// New function returning structured matches
pub fn extract_entity_matches(text: &str) -> Vec<EntityMatch> {
    // Returns EntityMatch with display_text, target_entity, span
}

// Validation helper
pub fn validate_entity_reference(alias: &str, entity: &str)
    -> Result<(), ValidationError>
{
    // Explicit validation
}
```

These would augment, not replace, existing APIs.

## Testing Requirements

### Unit Tests (Service Layer)

**Entity Parser**:
- Traditional syntax extraction (existing tests)
- Aliased syntax extraction (new)
- Mixed syntax extraction (new)
- Whitespace trimming (existing + new)
- Empty content handling (new)
- Malformed syntax degradation (new)
- Case sensitivity (existing)
- Deduplication (existing + new)

**Entity Styler**:
- Traditional syntax rendering (existing)
- Aliased syntax rendering (new)
- Mixed syntax rendering (new)
- Color consistency across syntaxes (new)
- Whitespace trimming in rendering (existing + new)
- Empty brackets handling (existing)
- Plain mode vs color mode (existing)

### Integration Tests

**End-to-End Workflows**:
- Save thought with aliased entities → retrieve → render
- Query thoughts by entity → includes both syntaxes
- Color consistency across multiple thoughts
- Migration path: old thoughts continue to work

### Contract Tests

**CLI Behavior**:
- `wet add "thought with [alias](entity)"` → success
- `wet thoughts` → displays with correct styling
- `wet entities` → no changes (lists entities, not aliases)

## Summary

The service API contracts ensure:

1. **Backward Compatibility**: All existing APIs work identically for traditional syntax
2. **Consistent Behavior**: Extraction and rendering follow same rules
3. **Color Consistency**: Same entity = same color, regardless of alias
4. **Error Handling**: Silent degradation maintains user experience
5. **Testability**: Clear expectations for unit and integration tests

The contracts extend existing functionality without breaking changes, following Rust's principle of "making illegal states unrepresentable" at the type level while maintaining ergonomic APIs.
