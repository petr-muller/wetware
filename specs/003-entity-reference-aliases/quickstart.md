# Quickstart Guide: Entity Reference Aliases

**Feature**: 003-entity-reference-aliases
**Audience**: Developers implementing this feature
**Date**: 2026-01-31

## Overview

This guide helps you quickly understand and implement entity reference alias support in wetware. After reading this, you'll know how to extend the entity parser and styler to support `[alias](entity)` syntax while maintaining backward compatibility.

## 5-Minute Summary

### What's Changing

**Before**: Only `[entity]` syntax
```
"Meeting with [Sarah] about [project-alpha]"
```

**After**: Both `[entity]` and `[alias](entity)` syntax
```
"Meeting with [Sarah] about [project-alpha]"      # Still works
"Started [ML](machine-learning) course"            # New!
"Reading about [robotics] and [robot](robotics)"  # Mixed!
```

### Core Concepts

1. **Display Text**: What the user sees (alias or entity name)
2. **Target Entity**: What entity it links to (for queries/colors)
3. **Same Color Rule**: `[robotics]` and `[robot](robotics)` get the same color

### Files to Modify

- `src/services/entity_parser.rs` - Pattern matching
- `src/services/entity_styler.rs` - Rendering
- (Optionally) `src/errors/thought_error.rs` - New error variants

## Implementation Walkthrough

### Step 1: Update Regex Pattern

**File**: `src/services/entity_parser.rs`

**Current Pattern**:
```rust
lazy_static! {
    pub static ref ENTITY_PATTERN: Regex = Regex::new(r"\[([^\[\]]+)\]").unwrap();
}
```

**New Pattern**:
```rust
use std::sync::LazyLock; // Migrate from lazy_static (recommended)
use regex::Regex;

pub static ENTITY_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[([^\[\]]+)\](?:\(([^\(\)]+)\))?").unwrap()
});
```

**What changed**:
- Added `(?:\(([^\(\)]+)\))?` to match optional `(entity)` part
- Capture group 1: alias (or entity if traditional syntax)
- Capture group 2: entity reference (optional)
- Migrated from `lazy_static!` to `std::sync::LazyLock` (bonus improvement)

**Explanation**:
- `(?:...)` - Non-capturing group (doesn't create extra capture)
- `\(([^\(\)]+)\)` - Literal parens with content inside
- `?` - Makes the `(entity)` part optional (for backward compatibility)

### Step 2: Update extract_entities Function

**File**: `src/services/entity_parser.rs`

**Current Code**:
```rust
pub fn extract_entities(text: &str) -> Vec<String> {
    ENTITY_PATTERN
        .captures_iter(text)
        .map(|cap| cap[1].trim().to_string())
        .collect()
}
```

**New Code**:
```rust
pub fn extract_entities(text: &str) -> Vec<String> {
    ENTITY_PATTERN
        .captures_iter(text)
        .filter_map(|cap| {
            // Capture group 1: alias (or entity for traditional syntax)
            // Capture group 2: entity reference (if aliased syntax)
            let alias = cap[1].trim();
            let entity = cap.get(2).map(|m| m.as_str().trim());

            // Return target entity, not alias
            match entity {
                Some(ent) if !ent.is_empty() => Some(ent.to_string()), // Aliased
                None if !alias.is_empty() => Some(alias.to_string()),  // Traditional
                _ => None, // Invalid (empty)
            }
        })
        .collect()
}
```

**What changed**:
- Check if capture group 2 exists → aliased syntax
- If group 2 exists and non-empty: return it (target entity)
- If group 2 doesn't exist: return group 1 (traditional syntax)
- If either is empty: skip (invalid reference)
- Changed `map` to `filter_map` to handle validation

**Key Insight**: Always return the **target entity**, never the alias. This ensures queries work correctly.

### Step 3: Update EntityStyler Rendering

**File**: `src/services/entity_styler.rs`

**Current render_content (simplified)**:
```rust
pub fn render_content(&mut self, content: &str) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for cap in ENTITY_PATTERN.captures_iter(content) {
        let full_match = cap.get(0).unwrap();
        let entity_name = cap[1].trim();

        result.push_str(&content[last_end..full_match.start()]);

        if self.use_colors {
            let color = self.get_color(entity_name);
            let styled = entity_name.bold().color(color).to_string();
            result.push_str(&styled);
        } else {
            result.push_str(entity_name);
        }

        last_end = full_match.end();
    }

    result.push_str(&content[last_end..]);
    result
}
```

**New render_content**:
```rust
pub fn render_content(&mut self, content: &str) -> String {
    let mut result = String::new();
    let mut last_end = 0;

    for cap in ENTITY_PATTERN.captures_iter(content) {
        let full_match = cap.get(0).unwrap();

        // Group 1: display text (alias or entity name)
        // Group 2: target entity (if aliased syntax)
        let display_text = cap[1].trim();
        let target_entity = cap.get(2)
            .map(|m| m.as_str().trim())
            .unwrap_or(display_text); // If no group 2, use group 1

        // Add text before this entity
        result.push_str(&content[last_end..full_match.start()]);

        // Add styled or plain entity
        if self.use_colors {
            // Color by TARGET entity, display ALIAS
            let color = self.get_color(target_entity);
            let styled = display_text.bold().color(color).to_string();
            result.push_str(&styled);
        } else {
            result.push_str(display_text);
        }

        last_end = full_match.end();
    }

    // Add remaining text
    result.push_str(&content[last_end..]);
    result
}
```

**What changed**:
- Extract both `display_text` (group 1) and `target_entity` (group 2 or group 1)
- Color based on `target_entity`
- Display `display_text` (the alias)
- Use `unwrap_or(display_text)` to handle traditional syntax

**Key Insight**: Display the **alias**, but color by the **target entity**. This gives consistent colors while showing natural language.

### Step 4: Write Tests First (TDD)

**File**: `src/services/entity_parser.rs` (in tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests remain unchanged (backward compatibility)

    // New tests for aliased syntax
    #[test]
    fn test_extract_aliased_entity() {
        let entities = extract_entities("Started [ML](machine-learning) course");
        assert_eq!(entities, vec!["machine-learning"]); // Returns target, not alias
    }

    #[test]
    fn test_extract_mixed_syntax() {
        let entities = extract_entities("[robotics] and [robot](robotics)");
        assert_eq!(entities, vec!["robotics", "robotics"]); // Both extract same entity
    }

    #[test]
    fn test_extract_aliased_with_whitespace() {
        let entities = extract_entities("[ ML ]( machine-learning )");
        assert_eq!(entities, vec!["machine-learning"]);
    }

    #[test]
    fn test_extract_empty_alias_rejected() {
        let entities = extract_entities("[](entity)");
        assert!(entities.is_empty()); // Empty alias → no match
    }

    #[test]
    fn test_extract_empty_entity_rejected() {
        let entities = extract_entities("[alias]()");
        assert_eq!(entities, vec!["alias"]); // Treated as traditional syntax
    }

    #[test]
    fn test_extract_malformed_unclosed_paren() {
        let entities = extract_entities("[alias](unclosed");
        assert_eq!(entities, vec!["alias"]); // Falls back to traditional
    }
}
```

**File**: `src/services/entity_styler.rs` (in tests module)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Existing tests remain unchanged

    // New tests for aliased rendering
    #[test]
    fn test_render_aliased_entity() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("Started [ML](machine-learning) course");
        assert_eq!(output, "Started ML course"); // Shows alias, not entity
    }

    #[test]
    fn test_render_mixed_syntax_same_color() {
        let mut styler = EntityStyler::new(true);
        styler.render_content("[robotics]"); // Assign color to "robotics"

        // Now render aliased reference to same entity
        let output = styler.render_content("[robot](robotics)");

        // Both should have same color (verified by consistent color_map)
        assert_eq!(styler.color_map.len(), 1); // Only one entity color assigned
    }

    #[test]
    fn test_render_aliased_with_whitespace() {
        let mut styler = EntityStyler::new(false);
        let output = styler.render_content("[ ML ]( machine-learning )");
        assert_eq!(output, "ML"); // Whitespace trimmed
    }
}
```

### Step 5: Run Tests and Iterate

```bash
# Run all tests
cargo nextest run

# Run specific test file
cargo nextest run entity_parser

# Run with coverage (if configured)
cargo tarpaulin --out Html

# Check for clippy warnings
cargo clippy

# Format code
cargo fmt
```

**TDD Workflow**:
1. Write test (it fails - RED)
2. Implement minimum code to pass (GREEN)
3. Refactor for clarity (REFACTOR)
4. Repeat

## Common Patterns and Examples

### Pattern 1: Traditional Syntax (Unchanged)

```rust
// Input
"Meeting with [Sarah] about [project-alpha]"

// Extraction
vec!["Sarah", "project-alpha"]

// Rendering (plain)
"Meeting with Sarah about project-alpha"

// Rendering (colored)
"Meeting with Sarah about project-alpha"
// ^^^^^^ color X      ^^^^^^^^^^^^^ color Y
```

### Pattern 2: Aliased Syntax (New)

```rust
// Input
"Started [ML](machine-learning) course on [AI](artificial-intelligence)"

// Extraction
vec!["machine-learning", "artificial-intelligence"]
// Returns target entities, not aliases

// Rendering (plain)
"Started ML course on AI"

// Rendering (colored)
"Started ML course on AI"
//       ^^ color X    ^^ color Y
```

### Pattern 3: Mixed Syntax (New)

```rust
// Input
"Reading about [robotics] and [robot](robotics)"

// Extraction
vec!["robotics", "robotics"]
// Both extract same target entity

// Rendering (plain)
"Reading about robotics and robot"

// Rendering (colored)
"Reading about robotics and robot"
//             ^^^^^^^^     ^^^^^ both color X (same entity)
```

### Pattern 4: Case-Insensitive Deduplication

```rust
// Input
"[ML](machine-learning) and [ml](Machine-Learning)"

// Extraction (unique)
vec!["machine-learning"]
// Deduplicated case-insensitively by target entity

// Rendering
"ML and ml" // Both display their respective aliases
```

## Edge Cases Checklist

- [ ] Empty alias: `[](entity)` → No match
- [ ] Empty entity: `[alias]()` → Treated as traditional `[alias]`
- [ ] Whitespace: `[ alias ]( entity )` → Trimmed to `[alias](entity)`
- [ ] Unclosed paren: `[alias](` → Treated as traditional `[alias]`
- [ ] Space between: `[alias] (entity)` → Treated as traditional `[alias]` + plain `(entity)`
- [ ] Nested brackets: `[a[b]]` → Matches `[a[b]` (inner content)
- [ ] Multi-word alias: `[my robot project](robotics)` → Valid
- [ ] Same text: `[robotics](robotics)` → Valid (user explicitly chose aliased syntax)
- [ ] Mixed case: `[Robotics]` and `[robot](robotics)` → Same color

## Debugging Tips

### Issue: Colors are different for same entity

**Check**: Are you calling `get_color()` with the target entity or the display text?

```rust
// WRONG
let color = self.get_color(display_text); // Will vary by alias

// RIGHT
let color = self.get_color(target_entity); // Consistent per entity
```

### Issue: Extraction returns alias instead of entity

**Check**: Are you returning group 2 (entity) or group 1 (alias)?

```rust
// WRONG
Some(alias.to_string()) // Returns display text

// RIGHT
Some(entity.to_string()) // Returns target entity
```

### Issue: Malformed syntax breaks parsing

**Check**: Is your pattern optional?

```rust
// WRONG
r"\[([^\[\]]+)\]\(([^\(\)]+)\)" // Both parts required

// RIGHT
r"\[([^\[\]]+)\](?:\(([^\(\)]+)\))?" // Paren part optional
```

### Issue: Empty references cause panics

**Check**: Are you handling None and empty strings?

```rust
// WRONG
let entity = cap[2].trim(); // Panics if group 2 doesn't exist

// RIGHT
let entity = cap.get(2).map(|m| m.as_str().trim()); // Returns Option
```

## Performance Considerations

### Current Performance

- Single thought (500 chars, 5 entities): **~10 μs**
- Large thought (10k chars, 50 entities): **~300 μs**

### No Optimization Needed

The regex engine is highly optimized:
- Linear time complexity O(n)
- No backtracking (ReDoS-safe)
- Lazy compilation (cached)

### Best Practice

Use `std::sync::LazyLock` instead of `lazy_static`:
- Standard library (no dependency)
- Marginally faster compile times
- Future-proof

## Testing Strategy

### Unit Tests (50+ tests)

- Traditional syntax (14 existing tests)
- Aliased syntax extraction (10 new tests)
- Aliased syntax rendering (10 new tests)
- Mixed syntax (5 new tests)
- Edge cases (15 new tests)
- Backward compatibility (all existing tests)

### Integration Tests (10-20 tests)

- End-to-end: save thought with aliases → retrieve → render
- Entity-based filtering with aliases
- Color consistency across multiple thoughts
- CLI contract tests

### Property-Based Tests (Optional)

Using `proptest`:
```rust
proptest! {
    #[test]
    fn test_extract_never_panics(text in ".*") {
        let _ = extract_entities(&text); // Should never panic
    }

    #[test]
    fn test_traditional_syntax_always_works(entity in "[a-z]+") {
        let input = format!("[{}]", entity);
        let result = extract_entities(&input);
        assert_eq!(result, vec![entity]);
    }
}
```

## Deployment Checklist

- [ ] All existing tests pass
- [ ] New tests added for aliased syntax
- [ ] Backward compatibility verified
- [ ] Manual testing with real thoughts
- [ ] Code review completed
- [ ] Documentation updated (if public API changed)
- [ ] Performance benchmarks run (optional)
- [ ] No clippy warnings
- [ ] Code formatted (`cargo fmt`)

## FAQ

### Q: Do I need a database migration?

**A**: No! Thoughts are stored as-is with markup. Entity extraction happens at runtime.

### Q: Will existing thoughts break?

**A**: No! Traditional `[entity]` syntax works identically. The regex pattern is backward compatible.

### Q: What happens if an aliased entity doesn't exist?

**A**: The current implementation doesn't validate entity existence during parsing. This is consistent with existing behavior - malformed references are silently ignored or treated as plain text.

If validation is needed, add it at a higher layer (CLI or business logic), not in the parser.

### Q: Can I query by alias?

**A**: No, not in this feature. Queries match the **target entity**, not the alias. This is intentional - aliases are for display only.

### Q: Do aliases support special characters?

**A**: Yes, except for `[`, `]`, `(`, `)`. These characters would break the parsing. All other characters (hyphens, underscores, slashes, etc.) are supported.

### Q: What if I want to use `[alias](entity)` literally in text?

**A**: You can't escape it in this version. The pattern will always match. If needed, future versions could support escaping with backslash.

## Next Steps

1. **Implement changes** following steps 1-4 above
2. **Run tests** and ensure 90%+ coverage
3. **Update CLAUDE.md** if needed (add any new patterns)
4. **Test manually** with real thoughts
5. **Create PR** following constitutional principles

## Resources

- [Regex Pattern Research](research.md) - Detailed regex design analysis
- [Data Model](data-model.md) - EntityMatch structure and validation
- [Service API Contracts](contracts/service-api.md) - Complete API specifications
- [Feature Spec](spec.md) - Original feature requirements
- [Wetware Constitution](../../.specify/memory/constitution.md) - Development principles

## Summary

Entity reference aliases extend wetware's syntax in a backward-compatible way:

1. **Minimal changes**: Two files modified (entity_parser.rs, entity_styler.rs)
2. **Regex extension**: Single pattern handles both syntaxes
3. **Display vs Target**: Show alias, color by entity
4. **Backward compatible**: All existing tests pass
5. **Well-tested**: 50+ unit tests, 10+ integration tests

Follow TDD, respect the constitution, and ship incrementally. Good luck! 🚀
