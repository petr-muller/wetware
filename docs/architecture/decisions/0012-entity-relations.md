---
status: Accepted
date: "2026-07-17"
---

# Entity Relations

## Context

Entities were flat — no way to express that one entity is a more specific instance of another (e.g. AWS
under Amazon). Users filtering by a broad entity (`wet thoughts --on amazon`) wanted to automatically see
thoughts tagged only with a more specific descendant (`[aws]`), without manually filtering by every
descendant name.

## Decision

Add directed parent/child relations between entities via `wet entity relate <entity-name> --parent
<parent-name>` and `wet entity unrelate <entity-name> --parent <parent-name>`. Relations form a DAG (an
entity may have multiple parents), stored as a plain edge table (`entity_relations`) rather than a single
nullable `parent_id` column on `entities` — modeling multi-parent cases like an entity belonging to two
unrelated broader categories.

Reachability (which entities are transitively reachable as descendants of a given entity) is computed with
a SQLite recursive CTE (`WITH RECURSIVE`) at read time, not maintained as a separate materialized
transitive-closure table — at this project's scale (personal notes, not a large graph), the recursive
query is fast and needs no rebalancing on writes. `ThoughtsRepository::list_by_entity`/
`list_latest_by_entity` inline the CTE directly, so both call sites (`wet thoughts --on`, `wet entity
show`) become reachability-aware with no CLI-layer changes. The TUI, which loads all data upfront and
never re-queries mid-session, instead loads the full relation-edge list once at startup and computes the
reachable descendant-name set once per entity-picker selection (not per keystroke).

A dedicated `EntityRelationsRepository` (backed by the new `entity_relations` table) keeps this
graph-traversal SQL colocated and independently testable, mirroring how `thought_entities` is a distinct
concern from `entities`/`thoughts` themselves. The cycle check (`would_create_cycle`, using the same
recursive-reachability pattern) happens application-side in the CLI command before inserting, not via a DB
trigger or constraint — consistent with the "dumb insert" style of `EntitiesRepository::link_to_thought`.
Both `relate` and `unrelate` require both entities to already exist (consistent with `entity rename`/
`entity show`); `unrelate` is idempotent/no-op-safe on a nonexistent relation, matching
`EntitiesRepository::unlink_all_from_thought`'s style. `wet entity show` additionally displays direct
(non-transitive) `Parents:`/`Children:` lines when present.

## Consequences

- Filtering by a broad entity anywhere in the app (`wet thoughts --on`, `wet entity show`, TUI entity
  picker) now implicitly includes every descendant's thoughts — a behavior change for any existing entity
  that later gets a relation added, not just newly-created ones.
- The CTE-based approach means reachability is always computed fresh from current relation state; there's
  no separate cache to invalidate when a relation is added or removed.
- `ThoughtsRepository`'s two read methods gained non-trivial SQL (recursive CTE) where they previously did
  a simple join — a cost accepted for keeping reachability transparent to CLI-layer callers.
- The TUI's relation-loading-once-at-startup mirrors its existing thoughts/entities loading pattern (see
  [`0006-tui-viewer.md`](0006-tui-viewer.md)): a relation added via the CLI mid-TUI-session won't be
  reflected until restart, same caveat as any other concurrent external change.

## Alternatives considered

- **Single nullable `parent_id` column on `entities`** — rejected: forces a strict tree, but an entity
  legitimately needing two unrelated parents (e.g. "Amazon Leo" under both "Amazon" and "satellite
  connectivity domain") wouldn't fit.
- **Materialized transitive-closure table**, updated on every relation write — rejected: adds write-path
  complexity (rebalancing on insert/delete) that isn't justified at this project's data scale; a recursive
  CTE computed at read time is simpler and fast enough.
- **Resolving a descendant-id list in a separate round trip and binding a dynamic `IN (...)` list** in
  `ThoughtsRepository` — rejected in favor of inlining the CTE directly in the same query, avoiding a
  second round trip and keeping the reachability logic in one place per query.
- **DB trigger or `CHECK` constraint for cycle prevention** — rejected: SQLite's `CHECK` constraints can't
  express a recursive graph traversal; application-side validation via a reusable, independently-testable
  `would_create_cycle` primitive was simpler than a trigger-based approach.

## Related code

- [`src/storage/entity_relations_repository.rs`](../../../src/storage/entity_relations_repository.rs)
- [`src/storage/migrations/entity_relations_migration.rs`](../../../src/storage/migrations/entity_relations_migration.rs)
- [`src/cli/entity_relate.rs`](../../../src/cli/entity_relate.rs)
- [`src/storage/thoughts_repository.rs`](../../../src/storage/thoughts_repository.rs) (`list_by_entity`, `list_latest_by_entity`)
- [`src/tui/mod.rs`](../../../src/tui/mod.rs) (`with_relations`, `reachable_names`)

## Related docs

- [`../../systems/storage.md`](../../systems/storage.md)
- [`../../systems/cli.md`](../../systems/cli.md)
- [`../../systems/tui.md`](../../systems/tui.md)
- [`../../flows/tui-entity-filter.md`](../../flows/tui-entity-filter.md)
- [`0006-tui-viewer.md`](0006-tui-viewer.md)
