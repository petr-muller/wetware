---
status: Accepted
date: "2026-07-12"
---

# Entity Show

## Context

`wet entities` shows only a single-line, plain-text, first-paragraph preview per entity (see
[`0002-entity-descriptions.md`](0002-entity-descriptions.md)). There was no way to view an entity's full
description or its recently-linked thoughts without grepping the database directly.

## Decision

`wet entity show <name>` (case-insensitive match) displays a detail view: the entity's canonical name as a
header, its full description if present (all paragraphs, not just the first — the ellipsized-preview
formatter from `wet entities` is not used here), rendered with the same entity-reference styling rules as
thought content (consistent per-entity colors, bold, aliased references shown as their alias display
text) — then up to 5 most recently created thoughts linked to the entity, newest first, in the same
`[id] date - content` format `wet thoughts` uses, also styled. A dedicated repository query,
`ThoughtsRepository::list_latest_by_entity`, orders by `created_at DESC` and limits to 5 in SQL, rather
than fetching all of an entity's thoughts and truncating in the CLI layer. One `EntityStyler` instance is
shared across both the description and the thoughts list so entity color assignment stays consistent
within the single invocation — matching how `wet thoughts` assigns colors consistently across everything
it prints. If the entity has no description, that section is omitted entirely (no placeholder); if it has
no linked thoughts, a message is shown instead of an empty list. Respects the same `--color` semantics as
`wet thoughts`.

## Consequences

- `wet entity show` and `wet entities` intentionally use different description-rendering paths: the
  listing's compact, plain, first-paragraph-only preview vs. this command's full, styled detail view.
  They are not meant to converge — see [`0002-entity-descriptions.md`](0002-entity-descriptions.md) for
  the listing's rationale.
- Doing the newest-first-limit-5 logic in SQL (rather than in Rust after a full fetch) keeps the query
  cheap regardless of how many thoughts an entity accumulates over time.
- Sharing one `EntityStyler` instance across the whole command's output is now the established pattern for
  any future multi-section styled output — deviating from it (e.g. a fresh styler per section) would
  reintroduce the CLI-internal color-consistency problem this command was careful to avoid.

## Alternatives considered

- **Reusing `description_formatter::generate_preview` for the description here** — rejected: that
  formatter is specifically for compact, first-paragraph, unstyled listing previews, which is the opposite
  of what a detail view needs.
- **Fetching all of an entity's thoughts and truncating client-side** — rejected in favor of a
  `LIMIT`-based SQL query, since it avoids pulling unbounded data for entities with long histories just to
  discard most of it.

## Related code

- [`src/cli/entity_show.rs`](../../../src/cli/entity_show.rs)
- [`src/storage/thoughts_repository.rs`](../../../src/storage/thoughts_repository.rs) (`list_latest_by_entity`)

## Related docs

- [`../../systems/cli.md`](../../systems/cli.md)
- [`0002-entity-descriptions.md`](0002-entity-descriptions.md)
- [`0003-styled-entity-output.md`](0003-styled-entity-output.md)
