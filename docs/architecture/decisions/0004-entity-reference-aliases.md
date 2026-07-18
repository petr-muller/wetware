---
status: Accepted
date: "2026-04-10"
---

# Entity Reference Aliases

## Context

The traditional `[entity-name]` reference syntax forces thought text to read awkwardly whenever the
natural phrasing doesn't match the entity's exact name (e.g. writing "robots" in prose while the entity is
named `robotics`). A markdown-like alias syntax was wanted, without breaking existing content.

## Decision

Extend the entity-reference syntax to `[alias](entity)`, alongside the existing bare `[entity]` form, in a
single regex: `` \[([^\[\]]+)\](?:\(([^()]+)\))? `` — group 1 is the display text, optional group 2 is the
target entity. When rendering, only the alias text is shown, colored/bolded by the *target* entity (so the
same entity gets the same color regardless of which alias referenced it). For extraction/queries, the
target entity name is always returned, never the alias text. Both alias and target are trimmed of
whitespace; an empty alias (`[](entity)`) or empty target (`[alias]()`) is rejected and treated as plain
text, matching existing malformed-syntax behavior. No database migration is needed — thought content is
stored as raw text regardless of syntax, with extraction happening at read time.

## Consequences

- Fully backward compatible: existing `[entity]` content parses identically to before (group 2 simply
  absent).
- No schema change and no separate alias-registry table — aliases are purely per-occurrence presentational
  text, not a stored alternate name.
- The regex-based approach set the pattern later reused (and extended) by
  [`0010-entity-rename.md`](0010-entity-rename.md)'s reference-rewriting logic.

## Alternatives considered

- **Parser combinator instead of a single regex** — rejected: the syntax still has no nesting, so a regex
  remains sufficient.
- **Persisted alias registry** (mapping alias strings to entities) — rejected: aliases are meant to be
  free-form, per-occurrence text, not a fixed vocabulary; a registry would add complexity without a clear
  use case.

## Related code

- [`src/services/entity_parser.rs`](../../../src/services/entity_parser.rs)

## Related docs

- [`../../systems/services.md`](../../systems/services.md)
- [`0010-entity-rename.md`](0010-entity-rename.md)
