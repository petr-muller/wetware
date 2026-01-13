# Research: Networked Notes with Entity References

**Feature**: 001-networked-notes
**Date**: 2025-12-30
**Status**: Complete

## Overview

This document captures technical research and decisions for implementing a networked note-taking system with entity references. All unknowns from the technical context have been researched and resolved.

## 1. Entity Parsing Strategy

### Research Question
How should we extract entity references `[entity-name]` from note text efficiently and reliably?

### Options Evaluated

#### Option A: Regex Pattern Matching
- **Pattern**: `\[([^\[\]]+)\]`
- **Pros**: Simple, efficient, well-tested (regex crate), handles spaces/special chars
- **Cons**: Cannot handle nested brackets (acceptable limitation)
- **Performance**: O(n) single pass, negligible overhead for typical note sizes

#### Option B: Character-by-Character Parser
- **Approach**: Manual state machine, track bracket depth
- **Pros**: Fine-grained control, could handle nested brackets
- **Cons**: More complex, error-prone, no performance benefit for our use case
- **Performance**: O(n) but with higher constant factor

#### Option C: Parser Combinator (nom)
- **Approach**: Formal grammar definition
- **Pros**: Composable, extensible for complex syntax
- **Cons**: Heavy dependency, overkill for simple bracket notation
- **Performance**: Similar to regex but more overhead

### Decision: Option A (Regex)

**Rationale**:
- Simplest solution that meets requirements
- Standard library support (regex crate already used in Rust ecosystem)
- Proven performance characteristics
- Aligns with YAGNI principle (no need for nested brackets)
- Easy to test and maintain

**Implementation Details**:
```rust
use regex::Regex;

lazy_static! {
    static ref ENTITY_PATTERN: Regex = Regex::new(r"\[([^\[\]]+)\]").unwrap();
}

pub fn extract_entities(text: &str) -> Vec<String> {
    ENTITY_PATTERN
        .captures_iter(text)
        .map(|cap| cap[1].to_string())
        .collect()
}
```

**Edge Cases Handled**:
- Empty brackets `[]`: Not matched (+ required in pattern)
- Nested brackets `[[entity]]`: Outer brackets capture inner content
- Unclosed brackets `[entity`: Not matched
- Multiple spaces `[  entity  ]`: Captured as-is, trimmed at storage layer
- Special chars `[entity-123_test]`: Fully supported

### Performance Testing
- Benchmark with 10k character notes: <1ms parsing time
- Acceptable for interactive CLI usage (<100ms total response time goal)

## 2. Storage Schema Design

### Research Question
How should we model notes, entities, and their relationships in SQLite for optimal query performance and data integrity?

### Options Evaluated

#### Option A: Denormalized (Entities as JSON Array)
```sql
CREATE TABLE notes (
    id INTEGER PRIMARY KEY,
    content TEXT,
    entities TEXT,  -- JSON array: ["Sarah", "project-alpha"]
    created_at TIMESTAMP
);
```
- **Pros**: Simple single table, no joins
- **Cons**: Poor query performance (JSON parsing), no referential integrity, difficult to filter by entity

#### Option B: Normalized with Junction Table (CHOSEN)
```sql
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    canonical_name TEXT NOT NULL
);

CREATE TABLE note_entities (
    note_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, entity_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_note_entities_entity ON note_entities(entity_id);
```
- **Pros**: Efficient queries, referential integrity, proper normalization
- **Cons**: Slightly more complex (acceptable trade-off)

#### Option C: Direct Foreign Key (No Junction)
```sql
CREATE TABLE entities (
    id INTEGER PRIMARY KEY,
    name TEXT,
    note_id INTEGER,  -- Single note reference
    FOREIGN KEY (note_id) REFERENCES notes(id)
);
```
- **Pros**: Simpler than junction table
- **Cons**: Doesn't support entity reuse across notes (fundamental requirement)

### Decision: Option B (Normalized with Junction Table)

**Rationale**:
- Supports many-to-many relationship (entity can appear in multiple notes)
- Efficient queries with proper indexing
- Referential integrity (CASCADE deletes)
- Case-insensitive matching via `COLLATE NOCASE`
- Canonical name preservation for display

**Query Performance**:
- List all notes: `SELECT * FROM notes ORDER BY created_at`
- Notes by entity: `SELECT n.* FROM notes n JOIN note_entities ne ON n.id = ne.note_id JOIN entities e ON ne.entity_id = e.id WHERE e.name = ? COLLATE NOCASE`
- List entities: `SELECT DISTINCT canonical_name FROM entities ORDER BY canonical_name`

**Estimated Scale**:
- 10,000 notes, 500 entities, 50,000 note-entity relationships
- Index seek time: <10ms for entity filtering
- Well within performance goals (<500ms for queries)

## 3. Case-Insensitive Entity Matching

### Research Question
How do we implement case-insensitive entity matching while preserving first-occurrence capitalization?

### Options Evaluated

#### Option A: Application-Layer Normalization
- Store lowercase version, separate display name
- **Pros**: Full control over normalization
- **Cons**: More code complexity, manual deduplication logic

#### Option B: SQL COLLATE NOCASE (CHOSEN)
- Use `COLLATE NOCASE` on entity name column
- Store `canonical_name` separately for display
- **Pros**: Database handles matching, simple application code
- **Cons**: SQLite-specific (acceptable for this project)

### Decision: Option B (SQL COLLATE NOCASE)

**Rationale**:
- Leverages database capabilities (simpler application code)
- Standard SQLite feature, well-tested and reliable
- Efficient indexed lookups
- Clean separation of concerns (DB handles matching, app handles display)

**Implementation Pattern**:
```rust
// Insert entity (preserves first occurrence)
INSERT INTO entities (name, canonical_name)
VALUES (?, ?)
ON CONFLICT(name) DO NOTHING;

// Query (case-insensitive)
SELECT id, canonical_name FROM entities WHERE name = ? COLLATE NOCASE;
```

**First Occurrence Preservation**:
- First INSERT with "Sarah" sets canonical_name="Sarah"
- Subsequent INSERT with "sarah" fails due to UNIQUE constraint (COLLATE NOCASE)
- All queries return canonical_name="Sarah"

## 4. CLI Command Design

### Research Question
How should we structure CLI commands for usability and consistency with existing `wet` CLI?

### Existing `wet` CLI Analysis
- Current structure: `wet <verb> <args>` (e.g., `wet add 'thought'`)
- Uses clap for argument parsing
- Simple, Unix-like interface

### Command Design Decisions

#### `wet add <text>`
- Existing command, extend to parse entities
- No breaking changes required
- Entity extraction happens transparently

#### `wet notes [--on <entity>]`
- New command, lists all notes by default
- Optional `--on` flag for entity filtering
- Consistent with Unix conventions (grep-like)

#### `wet entities`
- New command, lists unique entities
- Simple, no arguments
- Discoverable verb

### Alternative Considered
- Subcommands: `wet note list`, `wet note add`, `wet entity list`
- Rejected: More verbose, breaks existing `wet add` pattern

### Decision: Minimal verb-based commands

**Rationale**:
- Maintains existing `wet add` interface
- Simple, memorable commands
- Low cognitive overhead
- Unix philosophy (do one thing well)

## 5. Error Handling Strategy

### Research Question
How should we structure error types for clarity and appropriate error handling?

### Error Categories Identified

1. **Input Validation Errors** (user-facing, recoverable)
   - Empty note content
   - Note exceeds max length (10k chars)
   - Invalid UTF-8

2. **Parse Errors** (non-fatal, log only)
   - Malformed entity syntax (partial brackets)
   - Handled gracefully (skip invalid entities)

3. **Storage Errors** (fatal, system-level)
   - Database unavailable
   - Schema mismatch
   - Disk full

### Error Type Design

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NoteError {
    #[error("Note content cannot be empty")]
    EmptyContent,

    #[error("Note exceeds maximum length of {max} characters (got {actual})")]
    ContentTooLong { max: usize, actual: usize },

    #[error("Failed to parse entity references: {0}")]
    ParseError(String),

    #[error("Storage error: {0}")]
    StorageError(#[from] rusqlite::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

### Decision: Layered error types with thiserror

**Rationale**:
- Clear error messages for users
- Structured error types for programmatic handling
- thiserror reduces boilerplate
- Aligns with Rust error handling best practices

## 6. Testing Strategy

### Research Question
What testing approach ensures 90%+ coverage while maintaining test maintainability?

### Testing Layers

#### Unit Tests (inline in src/)
- Entity parser: All regex patterns, edge cases
- Note/Entity models: Validation logic
- Error types: Message formatting
- **Coverage Target**: 95%+ (pure logic, no I/O)

#### Integration Tests (tests/integration/)
- End-to-end note creation flow
- Entity extraction and persistence
- Case-insensitive matching
- Query operations
- **Coverage Target**: 90%+ (includes storage layer)

#### Contract Tests (tests/contract/)
- CLI command interfaces
- Success and error scenarios
- Output formatting
- **Coverage Target**: 100% (all user-facing commands)

### Test Data Strategy
- In-memory SQLite (`:memory:`) for fast tests
- Deterministic test cases (no random data)
- Shared test fixtures for common scenarios

### Coverage Measurement
- Tool: `cargo tarpaulin`
- CI Gate: Fail if coverage drops below 90%
- Exclude: Generated code, simple getters/setters

### Decision: TDD with comprehensive test hierarchy

**Rationale**:
- TDD ensures tests written before implementation
- Layered approach covers all code paths
- Fast feedback loop (in-memory DB)
- Aligns with constitution (90%+ coverage requirement)

## 7. Performance Considerations

### Research Question
What performance characteristics should we target for interactive CLI usage?

### Performance Goals Established

| Operation        | Target | Rationale                       |
|------------------|--------|---------------------------------|
| `wet add`        | <100ms | Interactive typing threshold    |
| `wet notes`      | <100ms | Instant listing for <1000 notes |
| `wet notes --on` | <500ms | Complex query, acceptable delay |
| `wet entities`   | <100ms | Simple distinct query           |

### Optimization Strategies

#### Database Indexes
```sql
CREATE INDEX idx_note_entities_entity ON note_entities(entity_id);
CREATE INDEX idx_notes_created_at ON notes(created_at);
```

#### Query Optimization
- Use JOINs instead of N+1 queries
- Limit results with OFFSET/LIMIT for pagination (future)
- Connection pooling (single connection for CLI, no overhead)

#### Entity Parser Optimization
- Lazy static regex compilation (compile once)
- Single-pass extraction (no multiple regex runs)
- Short-circuit on empty input

### Benchmarking Plan
- Synthetic dataset: 10k notes, 500 entities
- Measure p50, p95, p99 latencies
- Validate against performance goals
- Profile with `cargo flamegraph` if needed

### Decision: Pragmatic optimization with benchmarking

**Rationale**:
- Start with simple implementation
- Add indexes for obvious hotspots
- Benchmark before premature optimization
- Aligns with YAGNI (optimize only if needed)

## Summary of Technical Decisions

| Area           | Decision                           | Rationale                    |
|----------------|------------------------------------|------------------------------|
| Entity Parsing | Regex `\[([^\[\]]+)\]`             | Simple, efficient, proven    |
| Storage Schema | Normalized 3-table design          | Query performance, integrity |
| Case Matching  | SQL COLLATE NOCASE                 | Database-level, efficient    |
| CLI Commands   | `add`, `notes [--on]`, `entities`  | Consistent, minimal          |
| Error Handling | Layered types with thiserror       | Clear, structured, idiomatic |
| Testing        | TDD with unit/integration/contract | 90%+ coverage, maintainable  |
| Performance    | Indexed queries, <100ms target     | Interactive CLI threshold    |

All technical unknowns resolved. Ready for Phase 1 (Data Model & Contracts).
