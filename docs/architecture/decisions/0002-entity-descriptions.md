---
status: Accepted
date: "2026-04-10"
---

# Entity Descriptions

## Context

Entities (people, projects, topics) needed more context than just a name — users wanted to attach
multi-paragraph notes to an entity, viewable both as a quick preview when listing entities and in full
detail elsewhere.

## Decision

Add a single nullable `description TEXT` column to the `entities` table. Descriptions support the same
entity-reference syntax as thought content (`[entity]` and `[alias](entity)`), and references to
non-existent entities within a description auto-create them, same as in thought content. `wet entity edit
<name>` sets/updates a description via three mutually exclusive input methods: inline `--description`,
`--description-file`, or an interactive `$EDITOR` session (fallback chain `$EDITOR` → vim → nano → vi) when
neither flag is given. A whitespace-only description clears/removes it. Descriptions can only be added to
existing entities, not at creation time.

For the `wet entities` listing, show a single-line ellipsized preview: split on the first blank line
(first paragraph only), strip entity markup to plain text (no color, no bold — this is a compact listing,
not styled detail output), and truncate at the last word boundary before the available width, appending a
Unicode ellipsis. The preview is suppressed if the terminal is narrower than 60 characters, or if the
entity name leaves less than 20 characters of space for the preview text itself.

## Consequences

- A simple, backward-compatible migration (one nullable column, checked via `pragma_table_info` before
  adding).
- The preview pipeline (extract-first-paragraph → strip-markup → collapse-whitespace → ellipsize) became
  a small reusable formatting module, since it doesn't belong in the entity model itself.
- Full, styled descriptions (all paragraphs, entity references rendered as in thought content) were
  deferred to a later feature — see [`0011-entity-show.md`](0011-entity-show.md).

## Alternatives considered

- **Separate descriptions table** — rejected: a description is a 1:1 attribute of an entity, not a
  collection; a nullable column is simpler and needs no join.
- **Placeholder text for entities without descriptions** (e.g. "(no description)") — rejected: entity
  names display alone, with no placeholder, keeping the common case (most entities undescribed) visually
  quiet.

## Related code

- [`src/services/description_formatter.rs`](../../../src/services/description_formatter.rs)
- [`src/storage/migrations/add_entity_descriptions_migration.rs`](../../../src/storage/migrations/add_entity_descriptions_migration.rs)
- [`src/cli/entity_edit.rs`](../../../src/cli/entity_edit.rs)

## Related docs

- [`../../systems/services.md`](../../systems/services.md)
- [`../../systems/cli.md`](../../systems/cli.md)
- [`0011-entity-show.md`](0011-entity-show.md)
