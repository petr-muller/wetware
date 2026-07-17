# Storage

## Purpose

SQLite persistence for Thoughts and Entities: connection handling, schema migrations, and the two
repositories (`ThoughtsRepository`, `EntitiesRepository`) every other system goes through to read or
write data.

## Questions this doc answers

- What does the schema look like?
- How and when do migrations run?
- What methods do the repositories expose, and what do they error on?
- How is the data directory / database path resolved?

## Scope

`src/storage/connection.rs`, `data_dir.rs`, `migrations/`, `entities_repository.rs`,
`thoughts_repository.rs`, `entity_relations_repository.rs`.

## Non-scope

Domain type definitions (see [`models.md`](models.md)); config file storage, which lives alongside the
database in the same data directory but is handled by [`config.md`](config.md), not this module.

## Key concepts

- **Data Directory** — see [glossary](../glossary.md#data-directory).
- **Migration** — see [glossary](../glossary.md#migration).
- **Repository** — see [glossary](../glossary.md#repository).

## How the system works

**Connection** (`connection.rs`): `get_connection(db_path)` opens/creates the SQLite file and enables
`PRAGMA foreign_keys = ON`. `get_memory_connection()` opens an in-memory database, used by tests. There is
**no connection pooling** — every CLI command opens a fresh, short-lived `Connection` for the duration of
that command. `get_connection` does **not** run migrations itself; every command explicitly calls
`storage::run_migrations(&conn)` right after connecting.

**Data directory** (`data_dir.rs`): `resolve_data_dir(override_path)` — if an override is given, uses it;
otherwise, in release builds, falls back to `dirs::data_dir()/wetware`. **In debug builds, this panics**
if no override is given — a deliberate guard rail so `cargo run`/`cargo test` can never accidentally touch
a real user's data. `ensure_data_dir(path)` creates the directory. `default_db_path_in(data_dir)` returns
`<data_dir>/default.db`.

**Migrations** (`migrations/mod.rs`): `run_migrations(conn)` runs, in order, every time it's called:

1. `networked_notes_migration::migrate` — creates the base schema (below).
2. `add_entity_descriptions_migration::migrate_add_entity_descriptions` — adds `entities.description`.
3. `entity_relations_migration::migrate` — creates `entity_relations` (below).

Both are **idempotent**: `CREATE TABLE IF NOT EXISTS` and a `pragma_table_info` column-existence check
before adding a column. There is **no migration-version tracking table** — idempotency is the entire
safety net, and the pattern is strictly additive (no down-migrations). Because every command calls
`run_migrations` before doing anything else, the schema is always brought up to date on first use, at the
cost of a small idempotency check on every invocation.

**Schema** (from `networked_notes_migration.rs` + `add_entity_descriptions_migration.rs`):

```sql
CREATE TABLE IF NOT EXISTS thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_thoughts_created_at ON thoughts(created_at);

CREATE TABLE IF NOT EXISTS entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL,
    description TEXT  -- added by add_entity_descriptions_migration
);

CREATE TABLE IF NOT EXISTS thought_entities (
    thought_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (thought_id, entity_id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_thought_entities_entity ON thought_entities(entity_id);
CREATE INDEX IF NOT EXISTS idx_thought_entities_thought ON thought_entities(thought_id);

CREATE TABLE IF NOT EXISTS entity_relations (
    child_id INTEGER NOT NULL,
    parent_id INTEGER NOT NULL,
    PRIMARY KEY (child_id, parent_id),
    FOREIGN KEY (child_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES entities(id) ON DELETE CASCADE,
    CHECK (child_id != parent_id)
);
CREATE INDEX IF NOT EXISTS idx_entity_relations_parent ON entity_relations(parent_id);
CREATE INDEX IF NOT EXISTS idx_entity_relations_child ON entity_relations(child_id);
```

`entity_relations` is a directed edge table (`child_id` → `parent_id`) forming a DAG — an entity may have
multiple parents. The `CHECK (child_id != parent_id)` constraint prevents direct self-relations at the DB
level; longer cycles (A→B→A via an intermediate entity) are prevented at the application layer instead
(see below), since SQLite `CHECK` constraints can't express a recursive graph traversal. See
[`../architecture/decisions/0012-entity-relations.md`](../architecture/decisions/0012-entity-relations.md).

`entities.name` is `UNIQUE COLLATE NOCASE` — case-insensitivity is enforced at the database level, in
addition to app-level lowercasing in `Entity::new` (see [`models.md`](models.md)). `thought_entities` is
the many-to-many junction table between Thoughts and Entities; `ON DELETE CASCADE` in both directions
means deleting either side automatically cleans up the link. See
[`../architecture/decisions/0001-networked-notes-schema.md`](../architecture/decisions/0001-networked-notes-schema.md)
for why this normalized shape was chosen.

**Repositories** — all methods are `Self`-less static functions taking `&Connection`, not stateful
objects:

`EntitiesRepository`: `find_or_create`, `link_to_thought` (`INSERT OR IGNORE`), `find_by_name`
(case-insensitive), `list_all` (alphabetical by `canonical_name`), `unlink_all_from_thought`,
`update_description` (errors `EntityNotFound` if absent), `rename` (updates `name`+`canonical_name`,
errors `EntityNotFound`/`EntityAlreadyExists`; the collision check compares entity IDs, so a self-rename
or case-only casing change is allowed).

`ThoughtsRepository`: `save` (stores `created_at` as an RFC3339 string), `get_by_id`, `list_all`
(chronological ascending), `update` (errors `ThoughtNotFound` if zero rows affected), `delete` (errors
`ThoughtNotFound` if zero rows affected; relies on `ON DELETE CASCADE` for `thought_entities` cleanup),
`list_by_entity`, `list_latest_by_entity(limit)` (joins `thought_entities`/`entities`, `ORDER BY
created_at DESC LIMIT`). **Both `list_by_entity` and `list_latest_by_entity` are reachability-aware**: each
opens with a `WITH RECURSIVE reachable(id) AS (...)` CTE that starts at the named entity and follows
`entity_relations` child edges downward, then joins `thought_entities` against that CTE instead of
against a single entity ID directly — so both return thoughts tagged on the named entity *and* every
entity transitively reachable from it via child relations (its descendants), with no separate round trip
or dynamic `IN (...)` list. `DISTINCT` in the `SELECT` prevents a thought reachable via more than one path
in a diamond-shaped DAG from appearing twice.

`EntityRelationsRepository` (all methods static, taking `&Connection`, same style as the other
repositories): `add_relation(child_id, parent_id)` (`INSERT OR IGNORE` — idempotent), `remove_relation`
(`DELETE` — idempotent/no-op-safe if the relation doesn't exist), `would_create_cycle(child_id,
parent_id)` — runs the same recursive-descendants CTE rooted at `child_id` and checks whether `parent_id`
is already in that set (i.e. already a descendant of `child_id`); if so, adding `child_id`→`parent_id`
would close a cycle. `list_parents`/`list_children` return only *direct* (non-transitive) relations, for
`wet entity show`'s `Parents:`/`Children:` display. `list_all_edges` returns every `(child_id, parent_id)`
pair, used by the TUI to build its in-memory relation graph at startup (see
[`tui.md`](tui.md)).

Multi-step operations that touch more than one table (`cli/edit.rs`, `cli/entity_rename.rs`) wrap their
repository calls in `conn.transaction()` for atomicity — see [`flows/edit-thought.md`](../flows/edit-thought.md)
and [`flows/entity-rename.md`](../flows/entity-rename.md).

## Important flows

- [`../flows/edit-thought.md`](../flows/edit-thought.md)
- [`../flows/entity-rename.md`](../flows/entity-rename.md)

Adding a thought, showing an entity, and deleting a thought are single-transaction (or single-call)
operations covered here rather than as separate flow docs: "add" is `ThoughtsRepository::save` +
`entity_parser::extract_unique_entities` + `EntitiesRepository::find_or_create`/`link_to_thought` for each
extracted entity, all within one command invocation; "show" is a single
`ThoughtsRepository::list_latest_by_entity` read; "delete" is a single `ThoughtsRepository::delete` call
relying on `ON DELETE CASCADE`, from either `cli/delete.rs` or the TUI's `delete_selected_thought` (see
[`cli.md`](cli.md), [`tui.md`](tui.md)).

## Data and state

The SQLite database file at the resolved db path (default `<data_dir>/default.db`, override via
`WETWARE_DB`). Dates are always stored and parsed as RFC3339 strings; a parse failure surfaces as
`rusqlite::Error::FromSqlConversionFailure`.

## Interfaces and entry points

`get_connection`, `get_memory_connection`, `resolve_data_dir`, `ensure_data_dir`, `default_db_path_in`,
`run_migrations`, `EntitiesRepository::*`, `ThoughtsRepository::*`, `EntityRelationsRepository::*`.

## Dependencies

`errors` (`ThoughtError`), `models` (`Thought`, `Entity`), `rusqlite`, `dirs`.

## Downstream effects

Every CLI command and the TUI's startup load and delete path go through this layer. A schema change here
requires a new, idempotent migration — never edit an existing migration once it's shipped.

## Invariants and assumptions

- Debug builds always require an explicit `WETWARE_DATA_DIR` override — there is no fallback path in
  debug builds, by design, to prevent dev/test runs from touching production data.
- Migrations must remain idempotent and additive; there is no rollback mechanism.
- `entities.name` uniqueness is case-insensitive at the DB level (`COLLATE NOCASE`) — don't rely solely
  on app-level lowercasing when writing new queries.
- `entity_relations` cycle prevention beyond direct self-relation (the DB-level `CHECK`) is
  application-side only — a direct write to the table bypassing `EntityRelationsRepository::add_relation`
  (after a `would_create_cycle` check) could introduce a longer cycle undetected by the schema itself.

## Error handling

`update`/`delete` on `ThoughtsRepository` return `ThoughtError::ThoughtNotFound` when zero rows are
affected (not a SQLite-level error — checked explicitly). `EntitiesRepository::rename`/
`update_description` return `EntityNotFound`/`EntityAlreadyExists` similarly.

## Security and privacy notes

The database is a local, unencrypted SQLite file containing all thought content and entity descriptions
the user has entered — treat the data directory as sensitive local user data.

## Observability and debugging

Inspect the database directly with `sqlite3 <data_dir>/default.db`; `pragma_table_info` is what the
migrations use internally to check for existing columns, useful for manually verifying migration state.

## Testing notes

`get_memory_connection()` gives tests an isolated in-memory database; repository tests typically call
`run_migrations` against it first, then exercise the repository functions directly.

## Common pitfalls

- No connection pooling means every command pays SQLite connection-open + migration-idempotency-check
  cost on every invocation — fine at current scale, but worth knowing if performance ever becomes a
  concern.
- There's no migration-version table, so a migration bug can only be caught by its `IF NOT EXISTS`/
  column-check logic being correct — test new migrations against both a fresh database and one that
  already has the prior schema applied.

## Source map

- [`src/storage/connection.rs`](../../src/storage/connection.rs)
- [`src/storage/data_dir.rs`](../../src/storage/data_dir.rs)
- [`src/storage/migrations/mod.rs`](../../src/storage/migrations/mod.rs)
- [`src/storage/migrations/networked_notes_migration.rs`](../../src/storage/migrations/networked_notes_migration.rs)
- [`src/storage/migrations/add_entity_descriptions_migration.rs`](../../src/storage/migrations/add_entity_descriptions_migration.rs)
- [`src/storage/entities_repository.rs`](../../src/storage/entities_repository.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)
- [`src/storage/entity_relations_repository.rs`](../../src/storage/entity_relations_repository.rs)
- [`src/storage/migrations/entity_relations_migration.rs`](../../src/storage/migrations/entity_relations_migration.rs)

## Related docs

- [`models.md`](models.md), [`cli.md`](cli.md), [`tui.md`](tui.md)
- [`../architecture/README.md`](../architecture/README.md) — core data model as architectural fact.
- [`../architecture/decisions/0001-networked-notes-schema.md`](../architecture/decisions/0001-networked-notes-schema.md)
- [`../architecture/decisions/0007-data-directory.md`](../architecture/decisions/0007-data-directory.md)
- [`../architecture/decisions/0012-entity-relations.md`](../architecture/decisions/0012-entity-relations.md)
