# 012: Entity Relations

## Summary

Allow directed parent/child relations between entities (e.g. AWS is a child of Amazon), via `wet entity relate <name> --parent <parent>` and `wet entity unrelate <name> --parent <parent>`. Relations form a DAG — an entity may have multiple parents. Filtering thoughts by an entity anywhere in the app (`wet thoughts --on <name>`, `wet entity show <name>`, and the TUI's entity-picker filter) includes thoughts for that entity and every entity transitively reachable from it via child relations (its descendants), so a user filtering on a broad entity automatically sees thoughts tagged only with a more specific descendant.

## Requirements

- `wet entity relate <entity-name> --parent <parent-name>` marks `entity-name` as a child of `parent-name`. Both entities must already exist (matched case-insensitively); if either doesn't, errors with `Entity '<name>' not found` plus a hint to create it via `wet add "...[name]..."`.
- `wet entity unrelate <entity-name> --parent <parent-name>` removes that relation. Both entities must still exist. Idempotent: succeeds even if the relation didn't exist.
- Relating an entity to a parent it's already related to succeeds without error (idempotent).
- An entity cannot be related to itself (`entity-name` == `parent-name`, case-insensitive) — rejected.
- An entity may have more than one parent (DAG, not a strict tree).
- Adding a relation that would create a cycle (i.e. the proposed parent is already a descendant of the entity) is rejected — no partial state change.
- `wet thoughts --on <name>`, the thought list in `wet entity show <name>`, and the TUI entity-picker filter all include thoughts linked to `<name>` and to every entity transitively reachable from it via child relations.
- `wet entity show <name>` displays direct (non-transitive) `Parents:` and `Children:` lines when the entity has any, omitted entirely when it has none.

## Decisions

- **DAG, not a tree**: an entity may have multiple parents (e.g. "Amazon Leo" could be a child of both "Amazon" and "satellite connectivity domain"). Modeled as a plain edge table rather than a single nullable `parent_id` column on `entities`.
- **Recursive CTE for reachability, not a materialized closure table**: SQLite's `WITH RECURSIVE` computes descendants on read, avoiding the complexity of maintaining a separate transitive-closure table on every write. At this project's scale (personal notes, not a large graph) the recursive query is fast and requires no rebalancing.
- **Reachability query lives in `ThoughtsRepository`**: `list_by_entity`/`list_latest_by_entity` inline the recursive CTE directly (root entity + descendants) rather than resolving a descendant-id list in a separate round trip and binding a dynamic `IN (...)` list. This keeps both call sites (`wet thoughts --on`, `wet entity show`) reachability-aware automatically, with no CLI-layer changes.
- **Dedicated `EntityRelationsRepository` and `entity_relations` table**: kept separate from `EntitiesRepository`/`entities`, mirroring how `thought_entities` is a distinct concern from `thoughts`/`entities` themselves. Keeps the graph-traversal SQL colocated and independently testable.
- **Cycle check happens application-side (in the CLI command), not via a DB trigger**: `EntityRelationsRepository::would_create_cycle` is a reusable primitive the CLI calls before inserting; `add_relation` itself never validates, matching the "dumb insert" style of `EntitiesRepository::link_to_thought`.
- **`unrelate` is idempotent/no-op-safe**: removing a relation that doesn't exist succeeds silently, matching `EntitiesRepository::unlink_all_from_thought`'s no-op style, rather than erroring.
- **Both entities must already exist for `relate`/`unrelate`**: consistent with `entity rename`/`entity show`, which require the entity to already exist rather than auto-creating it. Keeps `relate` focused on linking, not entity creation.
- **`entity_relate.rs` holds both `execute_relate` and `execute_unrelate`**: unlike the one-file-per-command precedent (`entity_rename.rs`, `entity_show.rs`), these two operations are small, symmetric, and share entity-resolution logic, so splitting them into separate files would only duplicate that logic.
- **TUI filtering stays in-memory**: the TUI doesn't re-query the database per keystroke; it loads the full relation-edge list once at startup (alongside thoughts/entities) and computes the reachable descendant-name set once when an entity is picked, rather than querying on every filter recomputation.

## CLI Interface

```
wet entity relate aws --parent amazon
# Entity 'aws' is now a child of 'amazon'.

wet entity relate "amazon leo" --parent "satellite connectivity domain"
# Entity 'amazon leo' is now a child of 'satellite connectivity domain'.

wet entity unrelate aws --parent amazon
# Removed 'aws' as a child of 'amazon'.

wet thoughts --on amazon
# includes thoughts tagged only [aws], not just thoughts tagged [amazon]
```

Errors:
```
wet entity relate aws --parent nonexistent
# Error: Entity 'nonexistent' not found

wet entity relate amazon --parent amazon
# Error: An entity cannot be its own parent: 'amazon'

wet entity relate amazon --parent aws   # (aws already a child of amazon)
# Error: Cannot make 'amazon' a child of 'aws': 'aws' is already a descendant of 'amazon' (would create a cycle)
```

## Edge Cases

- Entity with no relations at all: filtering by it behaves exactly as before this feature (regression guard).
- Diamond-shaped DAG (an entity reachable from the filtered root via two different parent paths): its thoughts appear once in results, not duplicated.
- Relating an already-related pair: succeeds, no duplicate edge created.
- Filtering by the root entity itself still includes the root's own thoughts, not just descendants'.
- Filtering by an unknown entity name still returns an empty result, not an error (unchanged from pre-feature behavior).
- `wet entity show` on an entity with parents but no children (or vice versa) prints only the applicable line.
- Long transitive cycles (not just direct A->B->A) are still correctly rejected, since the cycle check follows the same recursive reachability traversal as filtering.
- No `entity delete` command exists yet; `entity_relations` rows use `ON DELETE CASCADE` on both `child_id` and `parent_id` so they clean up automatically if deletion is ever added.
