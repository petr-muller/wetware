# Quickstart: Implementing Edit Existing Thoughts

**Branch**: `004-edit-thoughts` | **Date**: 2026-02-19

## Prerequisites

- Rust 2024 edition, `cargo`, `cargo nextest`
- Familiarity with: `src/cli/add.rs` (the add workflow), `src/storage/thoughts_repository.rs`, `src/storage/entities_repository.rs`, `src/errors/thought_error.rs`

## Step-by-Step Implementation Order

Follow this order to avoid compilation gaps between steps.

### Step 1 — Add `ThoughtNotFound` error variant

**File**: `src/errors/thought_error.rs`

Add to the `ThoughtError` enum:
```rust
#[error("Thought with ID {0} not found")]
ThoughtNotFound(i64),
```

**Test**: Unit test that `ThoughtError::ThoughtNotFound(42)` formats as `"Thought with ID 42 not found"`.

---

### Step 2 — Add `EntitiesRepository::unlink_all_from_thought`

**File**: `src/storage/entities_repository.rs`

```rust
pub fn unlink_all_from_thought(conn: &Connection, thought_id: i64) -> Result<(), ThoughtError> {
    conn.execute("DELETE FROM thought_entities WHERE thought_id = ?1", [thought_id])?;
    Ok(())
}
```

**Test**: Insert a thought with entity links, call this method, assert `thought_entities` is empty for that thought.

---

### Step 3 — Add `ThoughtsRepository::update`

**File**: `src/storage/thoughts_repository.rs`

Signature:
```rust
pub fn update(
    conn: &Connection,
    id: i64,
    content: Option<&str>,
    created_at: Option<DateTime<Utc>>,
) -> Result<(), ThoughtError>
```

Logic:
1. Verify thought exists via `get_by_id(conn, id)` — propagate `ThoughtNotFound` on error.
2. Determine final content and created_at (new value or existing).
3. Execute `UPDATE thoughts SET content = ?1, created_at = ?2 WHERE id = ?3`.

**Test**: Update content only, update date only, update both; assert no-op doesn't corrupt unchanged fields.

---

### Step 4 — Add `Commands::Edit` subcommand

**File**: `src/cli/mod.rs`

```rust
Edit {
    /// ID of the thought to edit
    id: i64,
    /// New content for the thought
    content: Option<String>,
    /// New date for the thought (YYYY-MM-DD)
    #[arg(long)]
    date: Option<String>,
    /// Open the thought in an interactive editor
    #[arg(long, conflicts_with = "content")]
    editor: bool,
},
```

---

### Step 5 — Implement `cli::edit::execute`

**File**: `src/cli/edit.rs` (new file)

```rust
pub fn execute(
    id: i64,
    content: Option<String>,
    date: Option<String>,
    use_editor: bool,
    db_path: Option<&Path>,
) -> Result<(), ThoughtError>
```

Implementation outline:
1. Validate at least one of `content.is_some()`, `date.is_some()`, `use_editor` is true; otherwise return `ThoughtError::InvalidInput("...")`.
2. Open connection, run migrations.
3. Fetch existing thought via `ThoughtsRepository::get_by_id(&conn, id)`.
4. Resolve new content:
   - If `use_editor`: call `editor::launch_editor(Some(&thought.content))`, compare to original.
   - If `content.is_some()`: validate non-empty.
   - Otherwise: `None` (no content change).
5. Resolve new date: parse `date` string via `NaiveDate::parse_from_str(..., "%Y-%m-%d")`, convert to `DateTime<Utc>`.
6. If no effective changes: print "No changes made." and return `Ok(())`.
7. Begin transaction: `let tx = conn.transaction()?;`
8. `ThoughtsRepository::update(&tx, id, new_content, new_date)?`
9. If content changed:
   - `EntitiesRepository::unlink_all_from_thought(&tx, id)?`
   - Extract entities from new content, `find_or_create` each, `link_to_thought` each.
10. `tx.commit()?`
11. Print "Thought {id} updated."

---

### Step 6 — Route `Commands::Edit` in `main.rs`

**File**: `src/main.rs`

Add match arm:
```rust
Commands::Edit { id, content, date, editor } => {
    cli::edit::execute(id, content, date, editor, db_path.as_deref())?;
}
```

---

### Step 7 — Integration Tests

**File**: `tests/edit_thought.rs` (new integration test file)

Cover:
- Edit content via CLI arg: assert content updated, entity associations correct
- Edit date only: assert date updated, content/entities unchanged
- Edit content + date: assert both updated atomically
- Edit via editor (use mock/test editor env var): assert content updated
- Nonexistent ID: assert `ThoughtNotFound` error
- Empty content: assert `EmptyContent` error
- Invalid date: assert `InvalidInput` error
- Atomicity: simulate failure mid-transaction (if feasible) and assert thought unchanged

---

## Running Tests

```bash
# Run all tests
cargo nextest run

# Run only edit-related tests
cargo nextest run edit

# Lint
cargo clippy

# Format
cargo fmt
```

## Key Files Reference

| File | Purpose |
|---|---|
| `src/cli/add.rs` | Reference implementation — mirrors edit workflow |
| `src/input/editor.rs` | Editor launch — reused unchanged |
| `src/services/entity_parser.rs` | Entity extraction — reused unchanged |
| `src/storage/thoughts_repository.rs` | Add `update` method here |
| `src/storage/entities_repository.rs` | Add `unlink_all_from_thought` here |
| `src/errors/thought_error.rs` | Add `ThoughtNotFound` variant here |
| `src/cli/mod.rs` | Add `Commands::Edit` here |
| `src/cli/edit.rs` | New file — main implementation |
| `src/main.rs` | Add routing for `Commands::Edit` |
