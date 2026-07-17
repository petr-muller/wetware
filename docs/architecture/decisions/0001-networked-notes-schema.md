---
status: Accepted
date: "2026-04-10"
---

# Networked Notes Schema

## Context

wetware needed a way to capture short notes ("thoughts") that can reference named entities (people,
projects, topics), forming a network of related information, without free-text duplication of entity
names across notes.

## Decision

Store thoughts and entities as independent rows in a normalized three-table schema — `thoughts`,
`entities`, and a `thought_entities` many-to-many junction table — rather than embedding entity data
inside thought rows. Entity references inside thought content use a bracket syntax, `[entity-name]`,
parsed with a simple regex (`\[([^\[\]]+)\]`) rather than a parser combinator, since the syntax has no
nesting requirements. Entities are auto-created the first time they're referenced. Entity name matching
is case-insensitive (`COLLATE NOCASE` on `entities.name`), with a separate `canonical_name` column
preserving the first-used display casing.

The full schema now lives in [`../README.md`](../README.md#core-data-model) as architectural fact; this
ADR keeps only the rationale.

## Consequences

- Efficient many-to-many queries (filter thoughts by entity, list an entity's thoughts) via indexed joins
  through `thought_entities`.
- Entity identity is ID-based, so later features (descriptions, rename) can change how an entity is
  displayed or described without touching every thought that mentions it — see
  [`0010-entity-rename.md`](0010-entity-rename.md).
- Regex-based parsing means malformed syntax (unclosed brackets, nested brackets) is simply not matched
  and silently treated as plain text, rather than erroring — a deliberate simplicity/robustness tradeoff.

## Alternatives considered

- **Denormalized/tag-string storage** (e.g. a comma-separated entity list on each thought row) — rejected:
  makes case-insensitive, indexed entity lookups and joins harder, and duplicates entity display-name
  data across every thought that references it.
- **Parser combinator for entity extraction** — rejected: the bracket syntax has no nesting or recursive
  structure, so a full parser combinator would be unnecessary complexity for what a single regex handles.

## Related code

- [`src/storage/migrations/networked_notes_migration.rs`](../../../src/storage/migrations/networked_notes_migration.rs)
- [`src/services/entity_parser.rs`](../../../src/services/entity_parser.rs)

## Related docs

- [`../../systems/storage.md`](../../systems/storage.md)
- [`../README.md`](../README.md)
