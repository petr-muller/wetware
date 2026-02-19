# Data Model: Edit Existing Thoughts

**Branch**: `004-edit-thoughts` | **Date**: 2026-02-19

## Schema Changes

No schema changes are required. All necessary tables and columns exist.

## Existing Schema (Relevant Portions)

### `thoughts` table

```sql
CREATE TABLE IF NOT EXISTS thoughts (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    content     TEXT    NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
)
```

**Editable fields via this feature**:
- `content` — updated by direct CLI text argument or interactive editor
- `created_at` — updated by `--date YYYY-MM-DD` flag (stored as RFC3339 timestamp)

**Validation rules (enforced at domain layer, not just DB)**:
- `content` must not be empty after trimming
- `content` must not exceed 10,000 characters
- `created_at` must be a valid calendar date (YYYY-MM-DD)

### `entities` table

```sql
CREATE TABLE IF NOT EXISTS entities (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    name           TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL,
    description    TEXT
)
```

No changes. Entity auto-creation on edit reuses existing `find_or_create` logic.

### `thought_entities` junction table

```sql
CREATE TABLE IF NOT EXISTS thought_entities (
    thought_id  INTEGER NOT NULL,
    entity_id   INTEGER NOT NULL,
    PRIMARY KEY (thought_id, entity_id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id)  REFERENCES entities(id)  ON DELETE CASCADE
)
```

**Edit behaviour**: When thought content changes, all rows for that `thought_id` are deleted and re-inserted based on entity references in the new content. This happens atomically within a single SQLite transaction.

## Domain Model Changes

### `Thought` struct — unchanged

```rust
pub struct Thought {
    pub id: Option<i64>,       // None before persistence, Some(id) after
    pub content: String,
    pub created_at: DateTime<Utc>,
}
```

No new fields. The existing struct is sufficient for reading and updating thoughts.

### `ThoughtError` — new variant

```rust
#[error("Thought with ID {0} not found")]
ThoughtNotFound(i64),
```

Added to distinguish "thought does not exist" from general storage errors, enabling a clear user-facing error message.

## Repository Interface Changes

### `ThoughtsRepository` — new method

```
update(conn, id: i64, content: Option<&str>, created_at: Option<DateTime<Utc>>) -> Result<(), ThoughtError>
```

- Fetches thought by `id`; returns `ThoughtNotFound(id)` if missing
- Builds a single `UPDATE thoughts SET ... WHERE id = ?` with only changed fields
- Called within a transaction from `cli::edit::execute`

### `EntitiesRepository` — new method

```
unlink_all_from_thought(conn, thought_id: i64) -> Result<(), ThoughtError>
```

- Executes `DELETE FROM thought_entities WHERE thought_id = ?`
- Used to clear prior entity associations before re-inserting from new content
- Idempotent: safe to call even if no links exist

## Edit Operation — Transactional Flow

```
BEGIN TRANSACTION
  1. SELECT thought by id           → ThoughtNotFound if missing
  2. Resolve new content            → from arg, editor, or unchanged
  3. Resolve new date               → from --date, or unchanged
  4. If no fields changed           → ROLLBACK (no-op, notify user)
  5. UPDATE thoughts SET ...        → apply content/date changes
  6. If content changed:
       DELETE FROM thought_entities WHERE thought_id = ?
       For each entity in new content:
         find_or_create entity
         INSERT OR IGNORE thought_entities
COMMIT
```

If any step in the transaction fails, all changes are rolled back automatically (rusqlite `Transaction` rolls back on drop if not committed).
