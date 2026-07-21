# Entity Alias Resolution

## Purpose

Let a name that isn't an entity's canonical name still resolve to that entity everywhere a name is
looked up — filtering, `entity show`/`edit`/`rename`/`relate`/`unrelate`, and `[bracket]` mentions
extracted from thought/description text — via a persisted, per-entity alias registry.

## Trigger

Any lookup of an entity by a user-supplied name string: `wet thoughts --on <name>`, `wet entity show
<name>`, `wet entity edit <name>`, `wet entity rename <name> ...`, `wet entity relate/unrelate <name>
--parent <name>`, and entity extraction from `[bracket]` mentions during `wet add`/`wet edit`/`wet entity
edit`'s description text.

## Participants

- `storage/entities_repository.rs` (`EntitiesRepository::resolve`, `find_by_name`)
- `storage/entity_aliases_repository.rs` (`EntityAliasesRepository::find_entities_by_alias`)
- `services/entity_resolution.rs` (`resolve_or_create_entity`, used by `add`/`edit`/`entity edit`)
- `cli/entity_alias.rs` (`wet entity alias`/`unalias`, to manage the registry)
- `storage/thoughts_repository.rs` (`list_by_entity`/`list_latest_by_entity`, which resolve their filter
  root through this same path)

## Step-by-step flow

1. `EntitiesRepository::resolve(conn, name)` first checks for an exact, case-insensitive canonical-name
   match (`find_by_name`). If found, it's returned immediately — a canonical name always wins, even if
   some other entity also has that string registered as one of its aliases.
2. If there's no canonical match, `EntityAliasesRepository::find_entities_by_alias(conn, name)` looks up
   every entity that has `name` registered as an alias (case-insensitive):
   - **Zero matches** → `resolve` returns `Ok(None)` — the name is unresolved.
   - **Exactly one match** → `resolve` returns that entity.
   - **Two or more matches** (the same alias string registered to different entities) → `resolve` returns
     `Err(ThoughtError::AmbiguousAlias { alias, entities })`, naming every candidate's canonical name.
3. Direct lookup call sites (`entity show`/`edit`/`rename`/`relate`/`unrelate`) propagate `AmbiguousAlias`
   as a hard error via `?`, same as `EntityNotFound`.
4. `ThoughtsRepository::list_by_entity`/`list_latest_by_entity` can't call `resolve` from inside raw SQL,
   so they resolve the filter root in Rust first, then seed their `WITH RECURSIVE reachable(id)` CTE with
   the resolved entity's id instead of a `WHERE name = ?1` clause. An unresolved name still returns an
   empty result (preserving the pre-alias contract); an ambiguous alias now returns `AmbiguousAlias`
   instead of silently matching nothing.
5. `[bracket]` mention extraction (`add`/`edit`/`entity edit`) goes through
   `entity_resolution::resolve_or_create_entity` instead of calling `resolve` directly, since this path has
   a third possible outcome beyond found/not-found/ambiguous: an unresolved name falls back to creating a
   brand-new literal entity (the pre-alias behavior, unchanged). An ambiguous match prints a warning to
   stderr and skips linking that one mention — it does not fail the surrounding `add`/`edit`, and it does
   not fall back to creating a literal entity for the ambiguous string.

## Data and state changes

None by itself — this is a read path. `wet entity alias`/`unalias` (see [`../systems/cli.md`](../systems/cli.md))
are what mutate the `entity_aliases` table this flow reads from.

## Success behavior

A name resolves to exactly one entity if it's that entity's canonical name, or an alias registered to
exactly one entity. Filtering, showing, editing, renaming, relating, and linking a `[bracket]` mention all
behave identically whether the user typed the canonical name or a registered alias.

## Failure behavior

- Direct lookups (`entity show`/`edit`/`rename`/`relate`/`unrelate`, `thoughts --on`): an unresolvable name
  → `ThoughtError::EntityNotFound` (unchanged from before aliases existed); a name matching more than one
  entity's alias → `ThoughtError::AmbiguousAlias`, naming every candidate.
- `[bracket]` mention extraction (`add`/`edit`/`entity edit`): an unresolvable name creates a new entity
  (unchanged); an ambiguous alias prints a warning to stderr, skips linking that mention, and otherwise
  completes normally — the whole command still succeeds.

## External dependencies

None.

## Invariants and assumptions

- A canonical entity name can never be shadowed by an alias — `resolve` always checks canonical names
  first. `wet entity rename` separately guards against creating a *new* shadowing situation (see
  [`entity-rename.md`](entity-rename.md)).
- Aliases are unique per entity, not globally (`PRIMARY KEY (entity_id, alias)`) — the same alias string
  may legitimately be registered to more than one entity, and ambiguity is an expected, handled outcome,
  not a bug.
- `resolve_or_create_entity`'s ambiguous-mention warning is the only place in `add`/`edit`/`entity edit`
  where entity extraction has ever printed a warning — entity extraction elsewhere in those commands is
  otherwise infallible.

## Security and privacy notes

Not applicable beyond general local-data sensitivity noted in [`../systems/storage.md`](../systems/storage.md).

## Observability and debugging

If a `[bracket]` mention silently isn't linked to any entity after `add`/`edit`, check stderr for an
"matches multiple entities" warning — this is the ambiguous-alias skip path, not a crash.

## Testing notes

Cover: canonical-name match wins over a conflicting alias; unambiguous alias match; unresolved name;
ambiguous alias match (`AmbiguousAlias`, sorted candidate list); `--on` filtering by alias matches filtering
by canonical name; `--on` filtering by an ambiguous alias errors; unresolved `--on` name still returns
empty (not an error); `resolve_or_create_entity`'s three outcomes (create-new, resolve-existing,
skip-with-warning-on-ambiguity) via both unit tests and `add`/`edit` integration tests.

## Source map

- [`src/storage/entities_repository.rs`](../../src/storage/entities_repository.rs)
- [`src/storage/entity_aliases_repository.rs`](../../src/storage/entity_aliases_repository.rs)
- [`src/services/entity_resolution.rs`](../../src/services/entity_resolution.rs)
- [`src/cli/entity_alias.rs`](../../src/cli/entity_alias.rs)
- [`src/storage/thoughts_repository.rs`](../../src/storage/thoughts_repository.rs)

## Related docs

- [`../systems/storage.md`](../systems/storage.md), [`../systems/services.md`](../systems/services.md),
  [`../systems/cli.md`](../systems/cli.md)
- [`entity-rename.md`](entity-rename.md)
- [`../architecture/decisions/0013-entity-aliases.md`](../architecture/decisions/0013-entity-aliases.md)
- [`../architecture/decisions/0004-entity-reference-aliases.md`](../architecture/decisions/0004-entity-reference-aliases.md)
