# 011: Entity Show

## Summary

Allow viewing a single entity's detail via `wet entity show <name>`: the entity's full description, styled the same way entity references are styled in thought content (consistent colors, aliases rendered as their display text), followed by the 5 most recent thoughts linked to the entity.

## Requirements

- `wet entity show <name>` displays an entity matched case-insensitively on `name`.
- Errors with `Entity '<name>' not found` if no such entity exists.
- Displays the entity's canonical name as a header.
- If the entity has a description, displays it in full (all paragraphs, not just the first) with entity references rendered using the same styling rules as thought content: consistent per-entity colors, bold text, aliased references (`[alias](entity)`) shown as their alias display text rather than the target entity name. Colors are consistent between the description and the thoughts list below (same entity gets the same color in both).
- If the entity has no description, the description section is omitted entirely (no placeholder text).
- Displays up to 5 most recently created thoughts linked to the entity, newest first, in the same `[id] date - content` format used by `wet thoughts`, with entity references styled the same way.
- If the entity has no linked thoughts, displays a message instead of the list.
- Respects the same `--color` flag semantics as `wet thoughts` (auto/always/never).

## Decisions

- **Full description, not a preview**: unlike `wet entities` (spec 002), which shows a single-line ellipsized first-paragraph preview with plain (unstyled) entity references, `wet entity show` is a detail view and renders the complete description with full entity styling. `services::description_formatter::generate_preview` is not used here.
- **Newest-first, limit 5**: a dedicated repository query (`ThoughtsRepository::list_latest_by_entity`) orders by `created_at DESC` and limits in SQL, rather than fetching all of an entity's thoughts and truncating in the CLI layer.
- **Shared `EntityStyler` instance**: one styler is used across both the description and the thoughts list so that entity color assignment stays consistent within a single invocation, matching how `wet thoughts` assigns colors consistently across all printed thoughts.

## CLI Interface

```
wet entity show rust
```

```
Rust

Rust is a systems programming language focused on safety, particularly memory
safety, while maintaining performance. See also related work in ML.

Latest thoughts:
[42] 2026-07-10 - Started reading the Rust book again
[31] 2026-05-02 - Rust's borrow checker finally clicked today
```

(`ML` above renders as a styled alias for a `machine-learning` entity reference in the description, e.g. from `[ML](machine-learning)`.)

No description:
```
wet entity show wetware
```
```
wetware

Latest thoughts:
[12] 2026-06-01 - Added entity show command
```

Entity with no thoughts:
```
Latest thoughts:
No thoughts found for entity: wetware
```

Errors:
```
wet entity show nonexistent
# Error: Entity 'nonexistent' not found
```

## Edge Cases

- Entity has a description but zero linked thoughts: description prints in full, followed by the "no thoughts" message.
- Entity has thoughts but no description: description section omitted, thoughts list still prints.
- Entity has fewer than 5 thoughts: all of them print, newest first.
- Entity has more than 5 thoughts: only the 5 most recent print.
- Description spans multiple paragraphs: all paragraphs are printed (no first-paragraph truncation, unlike the `wet entities` preview).
- Description or thought content contains aliased entity references (`[alias](entity)`): the alias display text is shown, colored by the target entity, matching `wet thoughts` rendering rules.
