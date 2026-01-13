# Data Model: Networked Notes with Entity References

**Feature**: 001-networked-notes
**Date**: 2025-12-30
**Status**: Complete

## Overview

This document defines the data model for the networked notes feature, including entity schemas, field types, relationships, validation rules, and state transitions.

## Entity Definitions

### 1. Note

**Purpose**: Represents a user-created note with optional entity references.

**Fields**:

| Field        | Type     | Constraints                         | Description              |
|--------------|----------|-------------------------------------|--------------------------|
| `id`         | i64      | PRIMARY KEY, AUTO_INCREMENT         | Unique note identifier   |
| `content`    | String   | NOT NULL, 1-10000 chars             | Note text content        |
| `created_at` | DateTime | NOT NULL, DEFAULT CURRENT_TIMESTAMP | Creation timestamp (UTC) |

**Domain Model** (Rust):
```rust
pub struct Note {
    pub id: Option<i64>,          // None for unsaved notes
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Note {
    pub fn new(content: String) -> Result<Self, NoteError> {
        Self::validate_content(&content)?;
        Ok(Self {
            id: None,
            content,
            created_at: Utc::now(),
        })
    }

    fn validate_content(content: &str) -> Result<(), NoteError> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Err(NoteError::EmptyContent);
        }
        if content.len() > 10_000 {
            return Err(NoteError::ContentTooLong {
                max: 10_000,
                actual: content.len(),
            });
        }
        Ok(())
    }
}
```

**Validation Rules**:
- Content must not be empty (after trimming whitespace)
- Content must not exceed 10,000 characters
- Content must be valid UTF-8
- Created timestamp is immutable after creation

**Relationships**:
- Has many `EntityReference` (via junction table)
- Has many `Entity` (through `EntityReference`)

**Invariants**:
- Once created, `id` and `created_at` are immutable
- Content is immutable (no edit operations)
- Deletion cascades to all associated `EntityReference` records

**State Transitions**:
```
[Draft] --validate--> [Valid] --save--> [Persisted]
                         |
                         v
                    [Invalid]
                      (Error)
```

### 2. Entity

**Purpose**: Represents a unique named entity referenced across multiple notes.

**Fields**:

| Field            | Type   | Constraints                     | Description                                    |
|------------------|--------|---------------------------------|------------------------------------------------|
| `id`             | i64    | PRIMARY KEY, AUTO_INCREMENT     | Unique entity identifier                       |
| `name`           | String | UNIQUE COLLATE NOCASE, NOT NULL | Case-insensitive unique name                   |
| `canonical_name` | String | NOT NULL                        | Display name (first occurrence capitalization) |

**Domain Model** (Rust):
```rust
pub struct Entity {
    pub id: Option<i64>,
    pub name: String,              // Lowercase for lookups
    pub canonical_name: String,    // Original capitalization for display
}

impl Entity {
    pub fn new(name: String) -> Self {
        Self {
            id: None,
            name: name.to_lowercase(),  // Normalize for case-insensitive matching
            canonical_name: name,       // Preserve original
        }
    }

    pub fn display_name(&self) -> &str {
        &self.canonical_name
    }
}
```

**Validation Rules**:
- Name must not be empty
- Name must not contain only whitespace
- Name is normalized to lowercase for storage/lookup
- Canonical name preserves original capitalization

**Relationships**:
- Has many `EntityReference` (via junction table)
- Has many `Note` (through `EntityReference`)

**Invariants**:
- Name uniqueness enforced case-insensitively
- First occurrence determines canonical_name permanently
- Deletion cascades to all associated `EntityReference` records

**State Transitions**:
```
[Extracted] --normalize--> [Created] --save--> [Persisted]
                                                    |
                                                    v
                                        [Referenced] (exists in DB)
                                                    |
                                                    v
                                        [Reused] (subsequent notes reference same entity)
```

### 3. EntityReference

**Purpose**: Links notes to entities in a many-to-many relationship.

**Fields**:

| Field       | Type | Constraints                                   | Description           |
|-------------|------|-----------------------------------------------|-----------------------|
| `note_id`   | i64  | FOREIGN KEY → notes(id), ON DELETE CASCADE    | Note identifier       |
| `entity_id` | i64  | FOREIGN KEY → entities(id), ON DELETE CASCADE | Entity identifier     |
| -           | -    | PRIMARY KEY (note_id, entity_id)              | Composite primary key |

**Domain Model** (Rust):
```rust
// Represented as a relationship, not a separate struct
// Managed implicitly through repository operations

pub struct NoteWithEntities {
    pub note: Note,
    pub entities: Vec<Entity>,
}
```

**Validation Rules**:
- Both note_id and entity_id must reference existing records
- No duplicate (note_id, entity_id) pairs
- Cascade delete when either note or entity is deleted

**Relationships**:
- Belongs to one `Note`
- Belongs to one `Entity`

**Invariants**:
- Junction table entries are automatically managed
- Created when note is saved with entity references
- Deleted when note is deleted (CASCADE)

**State Transitions**:
```
[Parsed] --link--> [Created] --persist--> [Active]
                                             |
                                             v
                                          [Deleted]
                                    (note or entity deleted)
```

## Database Schema

### SQL DDL

```sql
-- Notes table
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_notes_created_at ON notes(created_at);

-- Entities table
CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL
);

-- Junction table for many-to-many relationship
CREATE TABLE note_entities (
    note_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, entity_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX idx_note_entities_entity ON note_entities(entity_id);
CREATE INDEX idx_note_entities_note ON note_entities(note_id);
```

### Indexes

| Index                      | Table         | Columns    | Purpose                                          |
|----------------------------|---------------|------------|--------------------------------------------------|
| `idx_notes_created_at`     | notes         | created_at | Chronological ordering for `wet notes`           |
| `idx_note_entities_entity` | note_entities | entity_id  | Fast entity-to-notes lookup for `wet notes --on` |
| `idx_note_entities_note`   | note_entities | note_id    | Fast note-to-entities lookup (reverse)           |

**Performance Impact**:
- Chronological listing: O(log n) with index seek
- Entity filtering: O(log m + k) where m=entities, k=matching notes
- Insert overhead: Minimal (< 5ms per note with entity extraction)

## Relationships

### ER Diagram (Text Representation)

```
┌─────────────────┐
│      Note       │
│─────────────────│
│ id (PK)         │──┐
│ content         │  │
│ created_at      │  │
└─────────────────┘  │
                     │
                     │ 1
                     │
                     │
                     ▼ *
           ┌──────────────────────┐
           │   note_entities      │
           │──────────────────────│
           │ note_id (PK, FK)     │
           │ entity_id (PK, FK)   │
           └──────────────────────┘
                     │ *
                     │
                     ▼ 1
┌─────────────────┐
│     Entity      │
│─────────────────│
│ id (PK)         │
│ name (UNIQUE)   │
│ canonical_name  │
└─────────────────┘
```

### Relationship Cardinality

- **Note** → **Entity**: Many-to-Many (via note_entities)
  - One note can reference zero or more entities
  - One entity can be referenced by zero or more notes

- **Note** → **EntityReference**: One-to-Many
  - One note creates multiple entity references
  - Each reference belongs to exactly one note

- **Entity** → **EntityReference**: One-to-Many
  - One entity can appear in multiple references
  - Each reference points to exactly one entity

## Data Flow

### Note Creation Flow

```
User Input: "Meeting with [Sarah] about [project-alpha]"
           |
           v
    [Validation]
           |
           v
    [Entity Extraction]  →  ["Sarah", "project-alpha"]
           |                            |
           v                            v
    [Save Note]              [Normalize & Dedupe]
           |                            |
           v                            v
    Note(id=1, ...)         ["sarah", "project-alpha"]
           |                            |
           |                            v
           |              [Lookup/Create Entities]
           |                            |
           |                ┌───────────┴───────────┐
           |                v                       v
           |        Entity(id=1, "sarah",   Entity(id=2, "project-alpha",
           |               "Sarah")                "project-alpha")
           |                |                       |
           └────────────────┴───────────────────────┘
                            |
                            v
                  [Create EntityReferences]
                            |
                ┌───────────┴───────────┐
                v                       v
    EntityRef(note_id=1,     EntityRef(note_id=1,
              entity_id=1)              entity_id=2)
```

### Query Flow (Entity Filtering)

```
User Input: `wet notes --on Sarah`
           |
           v
    [Normalize Entity Name]  →  "sarah"
           |
           v
    [Lookup Entity]
           |
           v
    Entity(id=1, name="sarah", canonical="Sarah")
           |
           v
    [Query Note IDs via Junction Table]
           |
           v
    note_ids = [1, 5, 12, ...]
           |
           v
    [Fetch Notes by ID]
           |
           v
    [Return Notes with Entity References]
```

## Validation Rules Summary

| Entity          | Rule                           | Error Type                    |
|-----------------|--------------------------------|-------------------------------|
| Note            | Content not empty (after trim) | `NoteError::EmptyContent`     |
| Note            | Content ≤ 10,000 chars         | `NoteError::ContentTooLong`   |
| Note            | Valid UTF-8                    | `NoteError::InvalidInput`     |
| Entity          | Name not empty (after trim)    | `NoteError::InvalidInput`     |
| Entity          | Unique (case-insensitive)      | Database constraint violation |
| EntityReference | note_id exists                 | Foreign key constraint        |
| EntityReference | entity_id exists               | Foreign key constraint        |

## Migration Strategy

### Initial Schema Creation

```sql
-- Run at first use of new feature
-- Safe to run multiple times (IF NOT EXISTS)

CREATE TABLE IF NOT EXISTS notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS note_entities (
    note_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (note_id, entity_id),
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at);
CREATE INDEX IF NOT EXISTS idx_note_entities_entity ON note_entities(entity_id);
CREATE INDEX IF NOT EXISTS idx_note_entities_note ON note_entities(note_id);
```

### Backwards Compatibility

- Existing `thoughts` table (if present) is unaffected
- New `notes` table is separate and independent
- No breaking changes to existing functionality
- Database file (wetware.db) extended with new tables

## Summary

This data model provides:
- ✅ Strong data integrity (foreign keys, constraints)
- ✅ Case-insensitive entity matching (COLLATE NOCASE)
- ✅ First-occurrence capitalization preservation (canonical_name)
- ✅ Efficient queries (indexed lookups)
- ✅ Clear validation rules (domain and database level)
- ✅ Immutable notes (no update operations)
- ✅ Cascade deletion (referential integrity)

All entities and relationships are defined with clear constraints and validation rules, ready for implementation.
