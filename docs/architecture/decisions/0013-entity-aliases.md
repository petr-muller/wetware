---
status: Accepted
date: "2026-07-21"
---

# Entity Aliases

## Context

Entities have exactly one canonical name today, but people don't refer to the same real-world thing
consistently — a nickname, an abbreviation, or a former name is often what actually gets typed. Without a
persisted association, each of those alternate names either has to be the entity's literal, single stored
name, or ends up creating a separate, disconnected entity that never shows up when filtering or browsing
by the "real" name.

This is a distinct problem from the existing `[alias](target)` bracket syntax handled by
[`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md). That syntax is free-form,
per-occurrence *display* text — it lets a single sentence read naturally ("started \[ML\](machine-learning)
course") without changing what gets stored or looked up. ADR 0004 explicitly rejected building a persisted
alias registry, but only in that narrower, presentational context, where aliases are one-off and never
need to be reused across notes. This ADR is not reopening or contradicting that rejection — it addresses a
different requirement 0004 never claimed to solve: is `nickname` actually the same entity as
`entity-x`, durably and queryably, everywhere a name is looked up?

## Decision

Add a persisted alias registry: a new `entity_aliases` table associating zero or more alias strings with
each entity, managed via `wet entity alias <name> --alias <x>` / `wet entity unalias <name> --alias <x>`
(mirroring the `entity relate`/`entity unrelate` CLI shape). An alias is unique per entity
(`PRIMARY KEY (entity_id, alias)`), not globally — the same alias string can be registered for two
different entities, since real-world naming collisions across unrelated entities are common and
shouldn't be blocked outright.

Resolution is centralized in one function, `EntitiesRepository::resolve`, used everywhere an entity was
previously looked up by exact name: `wet thoughts --on`, `wet entity show`/`edit`/`rename`/`relate`/
`unrelate`, and `[bracket]` mentions extracted from thought/description text. It checks the canonical
name first (exact, case-insensitive) — a canonical-name match always wins outright, even if some other
entity also has that string registered as an alias, so an alias can never shadow a real entity name. Only
when there's no canonical match are aliases consulted: zero matches means unresolved, one match resolves
unambiguously, and more than one match (the same alias registered to different entities) is a first-class
error (`ThoughtError::AmbiguousAlias`) rather than a silently arbitrary pick — silently choosing one of
several candidates would be a worse failure mode than refusing to guess, since it could link data to the
wrong entity with no visible signal.

For `[bracket]` mentions specifically (`wet add`/`wet edit`/`entity edit`'s description references), an
ambiguous alias match prints a warning to stderr naming the candidates and skips linking that one mention,
without failing the whole command and without falling back to creating a new entity literally named after
the ambiguous string. Entity extraction from thought/description text has never been able to fail the
overall save before, and it shouldn't start here — a single ambiguous mention buried in otherwise-fine
content failing the entire save would be disproportionate. Falling back to a literal new entity was
rejected too: it would compound the ambiguity with a third, spurious entity and give the user no visible
signal that anything needed fixing. Skip-with-warning has precedent in `edit --editor`'s abnormal-exit
handling, which already treats a soft failure as warn-and-continue rather than a hard error.

`wet entity rename` gains one additional guard: renaming an entity to a name that's already registered as
an alias of a *different* entity is rejected (`ThoughtError::RenameCollidesWithAlias`), because
`resolve()`'s canonical-wins-first rule means the rename would otherwise permanently and silently shadow
that other entity's alias. Renaming to a name that is already the *same* entity's own alias is fine (no
collision). No guard is added for the old name becoming free after a rename — that already happens
unguarded for canonical names today, and aliases behave the same way for consistency: reuse of a freed-up
name later, by anyone, is ordinary, not a collision to prevent.

`wet entity show` gains an `Aliases: a, b, c` line, placed after the description and before
`Parents:`/`Children:` (identity facts before relational context), omitted when the entity has none.

## Consequences

- `ThoughtsRepository::list_by_entity`/`list_latest_by_entity` no longer seed their reachability CTE with
  `WHERE name = ?1` directly — they resolve the entity in Rust first (via `EntitiesRepository::resolve`)
  and seed the CTE with the resolved id, since SQL can't express the ambiguity-vs-not-found distinction
  inline. The not-found contract (empty results, not an error) is preserved exactly; only the new
  ambiguous-alias case adds a new error path.
- `add`/`edit`/`entity edit`'s entity-extraction loops gain their first-ever failure-adjacent behavior
  (the ambiguous-mention warning) via a new shared helper, `entity_resolution::resolve_or_create_entity`,
  rather than each duplicating resolve-then-branch logic independently.
- `entity_edit.rs`, `entity_rename.rs`, and other CLI commands that resolve an entity by a possibly-aliased
  name must use the *resolved* canonical name for any subsequent repository call that does its own
  canonical-only lookup (e.g. `EntitiesRepository::update_description`) — passing the raw, alias-typed
  argument through unchanged would otherwise fail with `EntityNotFound`.
- The TUI's entity picker remains alias-*unaware* for this change — it fuzzy-matches and selects from an
  in-memory `Vec<Entity>` loaded once at startup, by index, so the ambiguity problem this ADR addresses
  doesn't arise there today. Making aliases searchable in the picker is a deliberate, documented follow-up,
  not a gap.

## Alternatives considered

- **Global alias uniqueness** (one alias string, one entity, forever) — rejected: it over-constrains
  real-world naming collisions between unrelated entities (two different people nicknamed "Boss", for
  instance) that per-entity uniqueness handles naturally by surfacing ambiguity instead of blocking
  registration.
- **Picking the first alias match arbitrarily on ambiguity** — rejected: a silently wrong resolution
  (linking to the wrong one of several candidates) is worse than an explicit, actionable error.
- **Failing `add`/`edit` outright on an ambiguous mention** — rejected: entity extraction has always been
  a best-effort side effect of saving, never a gate; blocking an entire save over one ambiguous bracket
  reference is disproportionate to the problem.
- **A nullable `entities.alias_of` self-referential column instead of a join table** — rejected for the
  same reason [`0012-entity-relations.md`](0012-entity-relations.md) rejected a single `parent_id` column
  for relations: an entity may reasonably want more than one alias, and a join table generalizes that
  cleanly without a schema change later.

## Related code

- [`src/storage/migrations/entity_aliases_migration.rs`](../../../src/storage/migrations/entity_aliases_migration.rs)
- [`src/storage/entity_aliases_repository.rs`](../../../src/storage/entity_aliases_repository.rs)
- [`src/storage/entities_repository.rs`](../../../src/storage/entities_repository.rs) (`resolve`)
- [`src/storage/thoughts_repository.rs`](../../../src/storage/thoughts_repository.rs)
- [`src/services/entity_resolution.rs`](../../../src/services/entity_resolution.rs)
- [`src/cli/entity_alias.rs`](../../../src/cli/entity_alias.rs)
- [`src/cli/entity_rename.rs`](../../../src/cli/entity_rename.rs)
- [`src/errors/thought_error.rs`](../../../src/errors/thought_error.rs) (`AmbiguousAlias`,
  `RenameCollidesWithAlias`)

## Related docs

- [`../../systems/storage.md`](../../systems/storage.md)
- [`../../systems/services.md`](../../systems/services.md)
- [`../../systems/cli.md`](../../systems/cli.md)
- [`../../flows/entity-alias-resolution.md`](../../flows/entity-alias-resolution.md)
- [`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md)
- [`0012-entity-relations.md`](0012-entity-relations.md)
