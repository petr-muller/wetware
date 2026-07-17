# Glossary

Domain-specific terms used across wetware. See [`STYLE.md`](STYLE.md) for how these terms are used
(Title Case) in other docs.

## Thought

A short, dated snippet of text — the core unit wetware stores. Has an ID, `content`, and a `created_at`
timestamp. Content must be non-empty and at most 10,000 characters. See
[`systems/models.md`](systems/models.md), [`systems/storage.md`](systems/storage.md).

## Entity

A named thing a Thought can reference (a person, project, topic, etc.), with an optional multi-paragraph
description. Has a lowercased `name` (used for case-insensitive lookup) and a `canonical_name` (the
originally-typed casing, used for display). See [`systems/models.md`](systems/models.md).

## Canonical Name

The display form of an Entity's name, preserving whatever casing was first used to create it — distinct
from the lowercased `name` field used for lookups and uniqueness. See
[`systems/models.md`](systems/models.md).

## Entity Reference

The bracket markup used inside Thought content and Entity descriptions to link to an Entity: `[entity]`
(traditional) or `[alias](entity)` (aliased — see below). Parsed by a shared regex in
[`systems/services.md`](systems/services.md).

## Alias

The display text in an aliased Entity Reference `[alias](entity)` — shown in place of the entity's name,
while the reference still resolves to and links the named entity. See
[`architecture/decisions/0004-entity-reference-aliases.md`](architecture/decisions/0004-entity-reference-aliases.md).

## Description Preview

The single-line, ellipsized summary of an Entity's description shown in `wet entities` listings —
derived from the description's first paragraph. See [`systems/services.md`](systems/services.md).

## Sort Order

Whether Thoughts are displayed oldest-first (`Ascending`) or newest-first (`Descending`, the default).
Configurable via `wet config thoughts.order`. See [`systems/config.md`](systems/config.md).

## Data Directory

The directory holding wetware's persistent state (`config.toml` and the SQLite database), resolved from
`WETWARE_DATA_DIR`, or the OS data directory in release builds. Debug builds require an explicit
override. See [`systems/storage.md`](systems/storage.md).

## Migration

An idempotent, additive schema change applied to the SQLite database on every command invocation (there
is no migration-version table — idempotency is the safety net). See
[`systems/storage.md`](systems/storage.md).

## Repository

The storage-layer pattern used for `EntitiesRepository` and `ThoughtsRepository` — collections of static
functions taking a `&Connection` and performing one persistence operation each, rather than stateful
objects. See [`systems/storage.md`](systems/storage.md).

## Mode

The TUI's current interaction state — `Normal`, `EntityPicker`, `ConfirmDelete`, or `EntityDetail` —
which key presses are dispatched to. See [`systems/tui.md`](systems/tui.md).

## Active Filter

The Entity name (if any) currently narrowing the TUI's Thought list, set via the `EntityPicker` mode. See
[`systems/tui.md`](systems/tui.md).

## Displayed Thoughts

The filtered and sorted list of indices into the TUI's full Thought list, recomputed whenever the Active
Filter or Sort Order changes. See [`systems/tui.md`](systems/tui.md).

## Color Mode

Whether output is styled: `Always`, `Never`, or `Auto` (the default — styles only when stdout is a
terminal). See [`systems/services.md`](systems/services.md).
