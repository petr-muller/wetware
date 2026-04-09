# 001: Networked Notes with Entity References

## Summary

Foundational feature: capture short notes via CLI that can reference named entities using `[entity-name]` bracket syntax, creating a network of related information. Notes are persisted in SQLite, queryable by entity, and entities are tracked as unique named concepts.

## Requirements

- `wet add '<text>'` saves a note with optional entity references parsed from `[entity-name]` syntax
- `wet thoughts` lists all notes in chronological order (oldest first)
- `wet thoughts --on <entity>` filters notes by entity reference (case-insensitive)
- `wet entities` lists all unique entities alphabetically
- Entity names are case-insensitive; first occurrence determines display capitalization
- Notes are immutable once created (no edit/delete in this feature)
- Note content max 10,000 characters; must not be empty after trimming
- Malformed entity syntax (e.g. `[unclosed`) is ignored gracefully
- Multi-word entities and special characters within brackets are supported
- Clear feedback on success/failure; helpful messages for empty results

## Decisions

- **Regex parser** (`\[([^\[\]]+)\]`) for entity extraction: simple, efficient, handles all required cases. No need for parser combinators.
- **Three-table schema**: `thoughts`, `entities`, `thought_entities` junction table. Normalized design for efficient many-to-many queries.
- **Case handling**: `COLLATE NOCASE` on entity name column + separate `canonical_name` column for display form.
- **Entities auto-created**: referencing a new entity in a note creates it automatically.

## Data Model

```sql
CREATE TABLE thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL
);

CREATE TABLE thought_entities (
    thought_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (thought_id, entity_id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);
```

Indexes on `thought_entities(entity_id)` and `thought_entities(thought_id)` for fast lookups.

## CLI Interface

```
wet add '<text>'                 # Add a note (entities auto-extracted)
wet thoughts                     # List all notes chronologically
wet thoughts --on <entity>       # Filter notes by entity (case-insensitive)
wet entities                     # List all unique entities alphabetically
```

Output format: `[<id>] <timestamp> - <content>`

## Edge Cases

- Empty/whitespace-only note content is rejected
- Notes exceeding 10,000 characters are rejected
- `[]` (empty entity reference) is ignored
- Nested brackets `[entity[sub]]` are not supported (regex doesn't match)
- Querying a non-existent entity returns empty results with a helpful message
- Duplicate entity references in a single note are deduplicated
