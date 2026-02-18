# Data Model: Entity Descriptions

**Feature**: 001-entity-descriptions
**Date**: 2026-02-01

## Overview

This document defines the data model changes required for entity descriptions, including database schema, domain model updates, and validation rules.

## Database Schema

### Existing Schema (Before Changes)

```sql
CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,  -- Lowercase normalized
    canonical_name TEXT NOT NULL                -- Original capitalization
);
```

### Updated Schema (After Migration)

```sql
CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,  -- Lowercase normalized
    canonical_name TEXT NOT NULL,               -- Original capitalization
    description TEXT                            -- NEW: Multi-paragraph description
);
```

**Changes**:
- Add `description TEXT` column (nullable)
- NULL represents entities without descriptions
- No indexing on description (no queries on content)

### Migration

**File**: `src/storage/migrations/add_entity_descriptions_migration.rs`

```rust
pub fn migrate_add_entity_descriptions(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "ALTER TABLE entities ADD COLUMN description TEXT;"
    )?;
    Ok(())
}
```

**Migration Number**: To be determined based on existing migrations (increment from last)

**Rollback Strategy**: Not implemented (SQLite doesn't support DROP COLUMN before 3.35.0). Migration is forward-only.

## Domain Model

### Entity Struct (Before)

```rust
pub struct Entity {
    pub id: Option<i64>,
    pub name: String,           // Lowercase normalized
    pub canonical_name: String, // Original capitalization
}
```

### Entity Struct (After)

```rust
pub struct Entity {
    pub id: Option<i64>,
    pub name: String,               // Lowercase normalized
    pub canonical_name: String,     // Original capitalization
    pub description: Option<String>, // NEW: Optional multi-paragraph description
}
```

**Changes**:
- Add `description: Option<String>` field
- `None` represents entities without descriptions
- `Some(String)` contains the full description text (may be multiple paragraphs)

**Constructor Updates**:

```rust
impl Entity {
    /// Create new entity without description
    pub fn new(name: String) -> Self {
        Entity {
            id: None,
            name: name.to_lowercase(),
            canonical_name: name,
            description: None,
        }
    }

    /// Create new entity with description
    pub fn with_description(name: String, description: Option<String>) -> Self {
        Entity {
            id: None,
            name: name.to_lowercase(),
            canonical_name: name,
            description,
        }
    }

    /// Get display name (canonical capitalization)
    pub fn display_name(&self) -> &str {
        &self.canonical_name
    }

    /// Check if entity has a description
    pub fn has_description(&self) -> bool {
        self.description.is_some()
    }

    /// Get description or empty string
    pub fn description_or_empty(&self) -> &str {
        self.description.as_deref().unwrap_or("")
    }
}
```

## Validation Rules

### Description Content Validation

**Rule 1: Whitespace-Only Descriptions**
- **Validation**: `description.trim().is_empty()`
- **Action**: Treat as `None` (remove description)
- **Applied**: Before saving to database

**Rule 2: Entity References**
- **Validation**: Parse entity references using `entity_parser::extract_unique_entities()`
- **Action**: Auto-create referenced entities that don't exist
- **Applied**: After description accepted, before saving

**Rule 3: Maximum Length**
- **Validation**: None (no maximum length restriction per spec)
- **Note**: SQLite TEXT type supports up to 1GB, practical limit is much lower

**Rule 4: Character Encoding**
- **Validation**: UTF-8 encoding (Rust String is always valid UTF-8)
- **Action**: Accept all valid UTF-8, including Unicode characters

### Input Validation by Method

**Inline Flag (`--description "text"`)**:
- Parse from command-line argument (String)
- Apply whitespace validation
- Extract entity references

**Interactive Editor**:
- Write current description to temp file (if exists)
- Launch editor
- Read modified temp file
- Apply whitespace validation
- Delete temp file

**File Input (`--description-file path`)**:
- Read file contents to String
- Return error if file doesn't exist or can't be read
- Apply whitespace validation
- Extract entity references

## Repository Interface

### EntitiesRepository Methods (Additions)

```rust
impl EntitiesRepository {
    /// Update entity description (or remove if None)
    pub fn update_description(
        &self,
        entity_name: &str,
        description: Option<String>,
    ) -> Result<(), ThoughtError> {
        // Find entity by name
        // Update description column
        // Return error if entity doesn't exist
    }

    /// Get entity with description
    pub fn find_by_name(&self, name: &str) -> Result<Option<Entity>, ThoughtError> {
        // MODIFY existing method to include description column in SELECT
    }

    /// List all entities with descriptions
    pub fn list_all(&self) -> Result<Vec<Entity>, ThoughtError> {
        // MODIFY existing method to include description column in SELECT
    }
}
```

**Breaking Changes**:
- `find_by_name()` return type changes: Entity now has description field
- `list_all()` return type changes: Entity now has description field
- **Mitigation**: Both methods already exist, extending Entity struct is backward-compatible (new field is Option)

## State Transitions

### Entity Lifecycle with Descriptions

```
[No Entity] --create--> [Entity without description (description: None)]
                                     |
                                     | edit --description "text"
                                     v
                         [Entity with description (description: Some(...))]
                                     |
                                     | edit --description "   " (whitespace)
                                     v
                         [Entity without description (description: None)]
```

**State Invariants**:
1. Entity name and canonical_name are always non-empty
2. Description is either None or Some(non-empty String after trim)
3. Entity references in description always point to existing entities (auto-created if needed)

## Relationships

### Unchanged Relationships

- **Thoughts ↔ Entities**: Many-to-many via `thought_entities` junction table (unchanged)
- **Entities are unique by name**: Case-insensitive uniqueness (unchanged)

### New Implicit Relationships

- **Entities ↔ Entities**: Entity descriptions can reference other entities
  - Not enforced at database level (no foreign key)
  - Entity references extracted and validated at application level
  - Referenced entities auto-created if they don't exist

**Example**:
```
Entity: "rust"
Description: "A systems programming language. See @programming for context."

Entity: "programming" (auto-created if doesn't exist)
Description: None
```

## Data Examples

### Entity without Description

```rust
Entity {
    id: Some(1),
    name: "rust",
    canonical_name: "Rust",
    description: None,
}
```

**Database Row**:
```
| id | name | canonical_name | description |
|----|------|----------------|-------------|
| 1  | rust | Rust           | NULL        |
```

### Entity with Simple Description

```rust
Entity {
    id: Some(2),
    name: "wetware",
    canonical_name: "Wetware",
    description: Some("A CLI tool for managing thoughts and entities. Built in Rust."),
}
```

**Database Row**:
```
| id | name     | canonical_name | description                                              |
|----|----------|----------------|----------------------------------------------------------|
| 2  | wetware  | Wetware        | A CLI tool for managing thoughts and entities. Built... |
```

### Entity with Multi-Paragraph Description and Entity References

```rust
Entity {
    id: Some(3),
    name: "knowledge-management",
    canonical_name: "Knowledge Management",
    description: Some(
        "Knowledge management (KM) is the process of capturing, organizing, and \
        sharing information. Tools like @wetware help individuals manage their \
        personal knowledge graphs.\n\n\
        Effective KM systems support entity references, making it easy to link \
        related concepts. For example, [Zettelkasten](@zettelkasten) is a \
        popular KM methodology."
    ),
}
```

**Database Row**:
```
| id | name                  | canonical_name        | description                        |
|----|----------------------|-----------------------|-------------------------------------|
| 3  | knowledge-management | Knowledge Management  | Knowledge management (KM) is the... |
```

**Entity References Extracted**:
- `@wetware` → Entity "wetware" (auto-created if doesn't exist)
- `[Zettelkasten](@zettelkasten)` → Entity "zettelkasten" (auto-created if doesn't exist)

## Query Patterns

### Fetch Entity with Description

```sql
SELECT id, name, canonical_name, description
FROM entities
WHERE name = ? COLLATE NOCASE;
```

### Fetch All Entities with Descriptions

```sql
SELECT id, name, canonical_name, description
FROM entities
ORDER BY name ASC;
```

### Update Entity Description

```sql
UPDATE entities
SET description = ?
WHERE name = ? COLLATE NOCASE;
```

### Clear Entity Description

```sql
UPDATE entities
SET description = NULL
WHERE name = ? COLLATE NOCASE;
```

**Note**: Use NULL, not empty string, for consistency

## Indexing Strategy

**No New Indexes Required**:
- Description column is not queried (no WHERE clauses on description content)
- Entity lookups use existing `name` column (already unique, effectively indexed)
- Alphabetical ordering uses `name` column (existing index)

**Future Optimization** (if full-text search added later):
- Consider SQLite FTS5 virtual table for description search
- Not in current scope

## Data Integrity

### Constraints

1. **Primary Key**: `id` (auto-increment)
2. **Unique Constraint**: `name` (case-insensitive via COLLATE NOCASE)
3. **Not Null**: `name`, `canonical_name` (unchanged)
4. **Nullable**: `description` (new, intentionally nullable)

### Referential Integrity

- **No foreign keys on entity references in descriptions**
  - Entity references are textual, not relational
  - Auto-creation pattern prevents orphaned references
  - Consistency maintained at application level, not database level

### Data Consistency

- **Entity names**: Always lowercase in `name`, original case in `canonical_name`
- **Descriptions**: Always NULL or non-empty String (whitespace validation)
- **Entity references**: Always point to existing entities (auto-created)

## Migration Impact

### Backward Compatibility

✅ **Fully backward compatible**:
- Adding nullable column doesn't break existing queries
- Existing entities get `description = NULL` by default
- No data loss or transformation required

### Deployment Steps

1. Run migration to add `description` column
2. Deploy updated application code
3. All existing entities have `description = NULL`
4. Users can add descriptions via `wet entity edit`

### Rollback Plan

**Not supported**: SQLite < 3.35.0 doesn't support DROP COLUMN

**Workaround** (if rollback needed):
1. Create new entities table without description column
2. Copy data from old table (excluding description)
3. Drop old table
4. Rename new table

**Risk**: Low - adding nullable column is safe operation
