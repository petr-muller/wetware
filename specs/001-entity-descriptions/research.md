# Research: Entity Descriptions

**Feature**: 001-entity-descriptions
**Date**: 2026-02-01

## Overview

This document consolidates technical research and decisions for implementing entity descriptions in the wetware CLI tool.

## Key Technical Decisions

### 1. Database Schema Extension

**Decision**: Add `description TEXT` column to existing `entities` table

**Rationale**:
- Simplest approach that maintains data locality
- Entities and descriptions have 1:1 relationship
- No need for separate table or join queries
- Aligns with SQLite best practices for optional text fields
- NULL for entities without descriptions (no storage overhead)

**Alternatives Considered**:
- **Separate descriptions table**: Rejected - unnecessary complexity for 1:1 relationship
- **JSON in separate column**: Rejected - overkill for single text field
- **NoSQL/document store**: Rejected - adds dependency, breaks existing architecture

**Migration Strategy**:
```sql
ALTER TABLE entities ADD COLUMN description TEXT;
```

### 2. Interactive Editor Support

**Decision**: Use `$EDITOR` environment variable with fallback to `vim`/`nano`/`vi`

**Rationale**:
- Standard Unix convention for interactive editing
- Respects user's configured editor preference
- Cross-platform (works on Linux, macOS, Windows with Git Bash)
- Simple implementation with `std::process::Command`

**Implementation Pattern** (Rust):
```rust
use std::env;
use std::process::Command;

fn launch_editor(temp_file: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .unwrap_or_else(|_| "vim".to_string());

    let status = Command::new(editor)
        .arg(temp_file)
        .status()?;

    if !status.success() {
        return Err(EditorError::LaunchFailed);
    }
    Ok(())
}
```

**Fallback Chain**:
1. `$EDITOR` environment variable
2. `vim` (most common)
3. `nano` (user-friendly alternative)
4. `vi` (always available on Unix systems)

**Alternatives Considered**:
- **Built-in multi-line input**: Rejected - poor UX for 2-3 paragraph text
- **GUI editor**: Rejected - breaks CLI-only workflow
- **Specific editor (e.g., nano only)**: Rejected - doesn't respect user preference

### 3. Terminal Width Detection

**Decision**: Use `terminal_size` crate for cross-platform terminal width detection

**Rationale**:
- Lightweight, zero-dependency crate (MIT license)
- Cross-platform (Unix, Windows, WASM)
- Returns `Option<(Width, Height)>` for safe handling
- Well-maintained and widely used

**Implementation**:
```rust
use terminal_size::{terminal_size, Width};

fn get_terminal_width() -> usize {
    terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80) // Default to 80 if detection fails
}
```

**Threshold Logic**:
- Detect width at runtime
- If width < 60: suppress preview, show entity name only
- If width >= 60: calculate available space for preview (width - entity_name_len - 3 for " - ")

**Alternatives Considered**:
- **libc ioctl**: Rejected - platform-specific, unsafe Rust
- **Fixed 80 char assumption**: Rejected - doesn't adapt to actual terminal
- **termion crate**: Rejected - heavier dependency with TUI features we don't need

### 4. Preview Generation Algorithm

**Decision**: Extract first paragraph, collapse newlines to spaces, ellipsize with "…"

**Rationale**:
- First paragraph typically contains the essence of the description
- Collapsing newlines creates readable single-line preview
- Unicode ellipsis ("…") is standard and saves 2 chars vs "..."
- Word-boundary truncation prevents mid-word cuts

**Algorithm**:
1. Split description on double newline (`\n\n` or `\r\n\r\n`)
2. Take first element (first paragraph)
3. Replace all newlines with single spaces
4. Trim whitespace
5. Calculate available width: `terminal_width - entity_name.len() - 3` (for " - ")
6. If text fits: display as-is
7. If text exceeds: truncate at last space before limit, append "…"

**Pseudocode**:
```rust
fn generate_preview(description: &str, max_width: usize) -> String {
    let first_para = description
        .split("\n\n")
        .next()
        .unwrap_or("");

    let normalized = first_para
        .replace('\n', " ")
        .trim()
        .to_string();

    if normalized.len() <= max_width {
        return normalized;
    }

    // Truncate at word boundary
    let truncated = normalized[..max_width]
        .rfind(' ')
        .map(|pos| &normalized[..pos])
        .unwrap_or(&normalized[..max_width.saturating_sub(1)]);

    format!("{}…", truncated)
}
```

**Edge Cases Handled**:
- Empty description: Return empty string
- Single-paragraph description: Use entire text
- Description shorter than max_width: No ellipsis
- No spaces in truncation range: Hard cut at limit - 1 char

### 5. Whitespace-Only Detection

**Decision**: Use `str::trim().is_empty()` pattern

**Rationale**:
- Built-in Rust method, no regex needed
- Handles all Unicode whitespace (spaces, tabs, newlines, etc.)
- Efficient O(n) operation
- Clear intent in code

**Implementation**:
```rust
fn is_whitespace_only(text: &str) -> bool {
    text.trim().is_empty()
}
```

**Behavior**:
- If user provides whitespace-only description: treat as `None` (remove description)
- Empty string: same as whitespace-only
- Database stores NULL for entities without descriptions

### 6. Entity Reference Auto-Creation in Descriptions

**Decision**: Reuse existing `find_or_create()` pattern from thought creation

**Rationale**:
- Consistent behavior with thought entity references
- Users expect same syntax and semantics
- Existing code path is well-tested
- Maintains referential integrity

**Implementation Flow**:
1. User saves description with entity references
2. Extract entities using `entity_parser::extract_unique_entities()`
3. For each entity: call `entities_repository::find_or_create()`
4. Auto-created entities have no description (NULL)
5. Save description to database

**Consistency Note**:
- This matches the thought creation behavior documented in `src/cli/add.rs`
- Entity parser already handles both `@entity` and `[alias](@entity)` syntax

### 7. Description Rendering in Preview

**Decision**: Strip entity reference markup, display as plain text (no colors)

**Rationale**:
- Spec requirement: "References in ellipsized descriptions would be displayed without markup (either with entity name or the used alias) but without the font or color highlight"
- Prevents visual clutter in single-line previews
- Focus on description content, not styling

**Rendering Algorithm**:
```rust
fn strip_entity_markup(text: &str) -> String {
    // Regex: \[([^\[\]]+)\](?:\(([^\(\)]+)\))?
    // Replace:
    // - [alias](entity) → alias
    // - [entity] → entity

    ENTITY_REGEX.replace_all(text, |caps: &Captures| {
        caps.get(2) // aliased entity (group 2 = entity in parens)
            .map(|_| caps[1].to_string()) // Use alias (group 1)
            .unwrap_or_else(|| caps[1].to_string()) // Use entity name (group 1)
    }).to_string()
}
```

**Example Transformations**:
- `"This relates to @project and [the main task](@task)"`
  → `"This relates to project and the main task"`
- `"See @docs for details"`
  → `"See docs for details"`

## Best Practices Applied

### Rust-Specific

1. **Error Handling**: Use `Result<T, ThoughtError>` throughout, add context with `context()`
2. **Ownership**: Pass `&str` for read-only, `String` for owned text
3. **Option Handling**: Use `Option<String>` for optional descriptions (NULL in DB)
4. **Testing**: Follow existing patterns in tests/contract/, tests/integration/
5. **Documentation**: Add rustdoc comments to all public functions

### SQLite

1. **NULL for optional fields**: Don't use empty strings, use NULL
2. **Migration ordering**: Migrations run sequentially in order
3. **Indexing**: Description column doesn't need index (no queries on content)

### CLI UX

1. **Progressive disclosure**: Show preview by default, full description on demand (future feature)
2. **Sensible defaults**: 80-char terminal width, vim editor
3. **Graceful degradation**: Hide preview on narrow terminals
4. **Clear feedback**: Inform user when editor launches, when description saved

## Open Questions (None)

All technical unknowns have been resolved through research and alignment with existing codebase patterns.

## Dependencies

**New Dependency Required**:
- `terminal_size = "0.3"` (MIT license, 0 dependencies)

**Existing Dependencies Reused**:
- `rusqlite` (database operations)
- `regex` (entity reference parsing)
- `clap` (CLI argument parsing)
- `std::process::Command` (editor launching)
- `std::fs` (file I/O for --description-file)

## References

- Existing entity parser: `src/services/entity_parser.rs`
- Existing entity repository: `src/storage/entities_repository.rs`
- Existing thought creation flow: `src/cli/add.rs`
- SQLite ALTER TABLE docs: https://www.sqlite.org/lang_altertable.html
- terminal_size crate: https://crates.io/crates/terminal_size
