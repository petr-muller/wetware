# Research: Edit Existing Thoughts

**Branch**: `004-edit-thoughts` | **Date**: 2026-02-19

## Decisions

### Decision 1: Thought Identification — Numeric ID (already exists)

**Decision**: Use the existing `id INTEGER PRIMARY KEY AUTOINCREMENT` field on the `thoughts` table. It is already returned by all repository queries and already displayed in listing output as `[id]`.

**Rationale**: No schema changes needed. `ThoughtsRepository::get_by_id()` already exists and returns `ThoughtError` on missing ID.

**Alternatives considered**: None needed — ID already present and surfaced.

---

### Decision 2: New `ThoughtNotFound` Error Variant

**Decision**: Add `ThoughtNotFound(i64)` to the existing `ThoughtError` enum in `src/errors/thought_error.rs`.

**Rationale**: The edit command must distinguish "thought does not exist" from storage errors. No generic error variant covers this case. All existing error-handling patterns use domain-specific `ThoughtError` variants.

**Alternatives considered**: Reuse `InvalidInput(String)` — rejected; semantically wrong and loses the ID for error messaging.

---

### Decision 3: Atomicity via SQLite Transactions

**Decision**: Introduce `conn.transaction()` (rusqlite `Transaction`) for the edit operation. No transactions currently exist in the codebase; the add command has a latent atomicity gap (thought saved before entity links).

**Rationale**: The spec requires SC-006 / FR-015: "a thought is never left in a partially-edited state." Editing touches three tables (thoughts, entities, thought_entities). Without a transaction, a crash after content update but before entity re-linking leaves stale associations. rusqlite's `Transaction` type drops with implicit rollback, making the safe path the default.

**Alternatives considered**:
- Best-effort (no transaction): Rejected — violates FR-015 and SC-006.
- Savepoints only: Overkill for a single compound operation; `transaction()` is sufficient.

**Scope note**: The add command's existing atomicity gap is out of scope for this feature; it should be addressed separately.

---

### Decision 4: Entity Re-association Strategy — Delete-All-Then-Re-Insert

**Decision**: When thought text changes, delete all existing `thought_entities` rows for that thought, then re-parse the new text and insert fresh associations.

**Rationale**: Simpler and more correct than diffing old vs. new entity sets. The `thought_entities` junction table has `ON DELETE CASCADE` on `thought_id`, so a targeted `DELETE FROM thought_entities WHERE thought_id = ?` cleanly removes all prior links. Re-insertion uses the existing `find_or_create` + `link_to_thought` path, reusing all existing logic.

**Alternatives considered**:
- Diff old/new entity sets and apply delta: More complex, no benefit for this use case.
- Re-use the full `add` entity-processing loop: Yes — the existing loop in `cli/add.rs` (lines 27-32) will be extracted or replicated.

---

### Decision 5: Date Representation — `NaiveDate` → `DateTime<Utc>` at midnight

**Decision**: Accept `--date` as `YYYY-MM-DD` string (same format already accepted by `wet add` via the `chrono` crate). Parse to `NaiveDate`, then convert to `DateTime<Utc>` at midnight UTC for storage. The `thoughts.created_at` column stores RFC3339 timestamps.

**Rationale**: Consistent with existing thought creation. chrono's `NaiveDate::parse_from_str` with `"%Y-%m-%d"` handles parsing; `.and_hms_opt(0,0,0).unwrap().and_utc()` converts to UTC timestamp. Invalid dates produce a parse error surfaced as `ThoughtError::InvalidInput`.

**Alternatives considered**: Accept full RFC3339 timestamp — too heavy for a personal CLI tool; date-only is the natural UX.

---

### Decision 6: Editor Pre-population — Reuse `launch_editor(initial_content)`

**Decision**: Reuse `input::editor::launch_editor(Some(&thought.content))` unchanged. This function already accepts `Option<&str>` initial content, writes it to a temp file, opens the editor, and returns the edited string.

**Rationale**: Zero new code needed in the editor module. The existing `entity_edit` command already uses this pattern (`cli/entity_edit.rs` line 83). No-change detection (unchanged content → no write) is handled by comparing returned string to original.

**Alternatives considered**: New editor variant: unnecessary — existing API is sufficient.

---

### Decision 7: Combined Text + Date Edit in Single Command

**Decision**: Both `content` (optional positional) and `--date` (optional flag) can appear in the same `wet edit` invocation. Both are applied within the same transaction.

**Rationale**: Clarified in session (Q2). The clap argument definition uses `Option<String>` for both; the execute function applies whichever fields are `Some`. At least one of `content`, `--date`, or `--editor` must be provided (validated at runtime; clap cannot express this constraint directly).

---

### Decision 8: No Changes to Listing Command

**Decision**: The `wet thoughts` listing command already displays `[id]` prefixes. No changes needed.

**Rationale**: Confirmed from code: `src/cli/thoughts.rs` lines 42-50 format output as `[{}] {} - {}` using `thought.id.unwrap_or(0)`.

---

## Existing Code Reused

| Component | File | Reused As-Is |
|---|---|---|
| `entity_parser::extract_unique_entities` | `src/services/entity_parser.rs:63` | Entity re-extraction after edit |
| `EntitiesRepository::find_or_create` | `src/storage/entities_repository.rs:13` | Entity auto-creation on edit |
| `EntitiesRepository::link_to_thought` | `src/storage/entities_repository.rs:32` | Re-linking entities after edit |
| `ThoughtsRepository::get_by_id` | `src/storage/thoughts_repository.rs:24` | Fetch thought before edit |
| `input::editor::launch_editor` | `src/input/editor.rs:22` | Editor-based edit |
| `ThoughtError` enum | `src/errors/thought_error.rs` | Extended with `ThoughtNotFound` |

## New Code Required

| Component | File | Purpose |
|---|---|---|
| `ThoughtError::ThoughtNotFound(i64)` | `src/errors/thought_error.rs` | Distinguish missing thought from storage errors |
| `ThoughtsRepository::update` | `src/storage/thoughts_repository.rs` | UPDATE thoughts content and/or date by ID |
| `EntitiesRepository::unlink_all_from_thought` | `src/storage/entities_repository.rs` | DELETE thought_entities WHERE thought_id = ? |
| `Commands::Edit { ... }` | `src/cli/mod.rs` | Add edit subcommand to clap enum |
| `cli::edit::execute` | `src/cli/edit.rs` | Full edit workflow with transaction |
| Route in `main.rs` | `src/main.rs` | Dispatch Edit command |
