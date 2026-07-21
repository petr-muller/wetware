# Services

## Purpose

Pure business-logic helpers with no I/O or persistence dependencies — entity-reference parsing, entity
color styling, description-preview formatting, and terminal color-mode detection — plus one small
DB-touching helper, `entity_resolution`, that ties `[bracket]` mention extraction to the persisted alias
registry. Reused by both the CLI and the TUI.

## Questions this doc answers

- What syntax does an entity reference use, and how is it parsed?
- How are entities colored/styled in output?
- How are entity description previews generated?
- How is color output enabled/disabled?

## Scope

`src/services/color_mode.rs`, `entity_parser.rs`, `entity_styler.rs`, `description_formatter.rs`,
`entity_resolution.rs`.

## Non-scope

Where this output is actually printed (see [`cli.md`](cli.md)) or rendered in a TUI frame (see
[`tui.md`](tui.md) — which has its own, separate color-assignment logic, see Common Pitfalls below).

## Key concepts

- **Entity Reference** — `[entity]` (traditional) or `[alias](entity)` (aliased, alias text displayed,
  `entity` resolved). See [glossary](../glossary.md#entity-reference).
- **Color Mode** — `Always` / `Auto` (default) / `Never`. See [glossary](../glossary.md#color-mode).
- **Description Preview** — see [glossary](../glossary.md#description-preview).

## How the system works

**`color_mode.rs`** — `ColorMode` (clap `ValueEnum`): `Always`, `Auto` (default), `Never`.
`should_use_colors(&self) -> bool` — for `Auto`, checks `stdout().is_terminal()`.

**`entity_parser.rs`** — the core entity-reference regex/extraction logic, shared by every system that
needs to find or rewrite entity references in text:

- `static ENTITY_PATTERN: LazyLock<Regex>` = `` \[([^\[\]]+)](?:\(([^()]+)\))? `` — matches both
  `[entity]` and `[alias](entity)`.
- `extract_entities(text) -> Vec<String>` — returns the *target* entity name for each reference (for
  aliased syntax, the parenthesized target, not the alias text).
- `extract_unique_entities(text) -> Vec<String>` — case-insensitive dedup, preserving first-occurrence
  order and casing.
- `rewrite_entity_references(text, old_name, new_name) -> String` — rewrites bare `[Old]` → `[New]` and
  aliased `[Alias](old)` → `[Alias](New)`, leaving alias display text and unrelated references untouched.
  Used by entity rename (see [`flows/entity-rename.md`](../flows/entity-rename.md)).

Note: this module's "aliased syntax" (`[alias](entity)`) is unrelated to the persisted alias registry
described below and in [`storage.md`](storage.md) — it's free-form, per-occurrence *display* text that
never touches `entity_aliases`, and `extract_entities` always resolves it to the parenthesized *target*
name directly, without consulting the registry. See
[`../architecture/decisions/0004-entity-reference-aliases.md`](../architecture/decisions/0004-entity-reference-aliases.md)
and [`../architecture/decisions/0013-entity-aliases.md`](../architecture/decisions/0013-entity-aliases.md)
for the distinction.

**`entity_resolution.rs`** — `resolve_or_create_entity(conn, name) -> Result<Option<i64>, ThoughtError>`,
the one function in this module that touches storage. Used by `add`/`edit`/`entity edit` wherever they
used to unconditionally `find_or_create` an extracted `[bracket]` name: it resolves `name` against
canonical names and registered aliases first (`EntitiesRepository::resolve`), only falling back to
creating a brand-new literal entity when nothing matches. If `name` is an alias registered to more than
one entity, it prints a warning to stderr and returns `Ok(None)` — the mention is skipped (not linked to
any entity, no new entity created) without failing the caller's overall command. See
[`../flows/entity-alias-resolution.md`](../flows/entity-alias-resolution.md).

**`entity_styler.rs`** — `EntityStyler { color_map, next_color, use_colors }`. Cycles through a 12-color
palette (excluding black/white). `EntityStyler::new(use_colors)`, `render_content(&mut self, content) ->
String` — strips entity markup and, if `use_colors`, colors+bolds each entity span. Color assignment is
**sequential, by order of first appearance within a single render pass** — the same entity gets a
consistent color across a single command's output, but color assignment is not persisted across runs.

**`description_formatter.rs`** — formats an entity description into a single-line preview for `wet
entities` listings. Pipeline: `extract_first_paragraph` (split on blank line) → `strip_entity_markup`
(strip `[..]`/`(..)`, keep display text) → `collapse_newlines` (normalize whitespace) →
`ellipsize_at_word_boundary(text, max_length)`. `get_terminal_width()` (via the `terminal_size` crate,
defaults to 80 if unavailable). `generate_preview(description, entity_name, terminal_width)` orchestrates
the pipeline and returns `""` if the available width is below `MIN_PREVIEW_WIDTH` (20 chars).

## Important flows

Entity reference rewriting is the core of [`flows/entity-rename.md`](../flows/entity-rename.md).

## Data and state

`EntityStyler` holds an in-memory `color_map` for the duration of a single render pass — not persisted.

## Interfaces and entry points

`ColorMode::should_use_colors`, `entity_parser::{extract_entities, extract_unique_entities,
rewrite_entity_references}`, `EntityStyler::{new, render_content}`,
`description_formatter::{generate_preview, get_terminal_width}`.

## Dependencies

`errors` (indirectly), `regex`, `owo-colors`, `terminal_size`. No dependency on `storage` or `cli` for
`color_mode`/`entity_parser`/`entity_styler`/`description_formatter` — this is what makes those services
reusable by the TUI as well. `entity_resolution` is the one exception: it depends on `storage` directly
(`EntitiesRepository`, `EntityAliasesRepository`) since resolving a name against the alias registry
requires a database read.

## Downstream effects

Both [`cli.md`](cli.md) and [`tui.md`](tui.md) depend on `entity_parser` for extracting/filtering entity
references. Only the CLI currently uses `EntityStyler` directly — the TUI reimplements its own styling
(see Common Pitfalls).

## Invariants and assumptions

- `extract_entities` always returns the *target* name, never the alias display text — callers that need
  the alias text must parse it themselves.
- The 12-color palette excludes black/white to remain visible against both light and dark terminal
  backgrounds.

## Error handling

These functions are infallible (no `Result` return) — malformed markup (e.g. unmatched brackets) simply
doesn't match the regex and is treated as plain text.

## Security and privacy notes

Not applicable.

## Observability and debugging

If entity references aren't being recognized, check them against `ENTITY_PATTERN` directly — bracket or
paren characters inside a name will break matching (this is also why `entity_rename.rs` in `cli.md`
rejects new names containing `[`, `]`, `(`, `)`).

## Testing notes

`entity_parser` and `description_formatter` have unit tests covering the regex edge cases (aliased vs.
bare references, nested-looking brackets) and the preview pipeline's truncation boundaries.

## Common pitfalls

- **`tui/ui.rs` has its own, separate hash-based entity color-assignment function** (`entity_color`/
  `entity_color_index`), not `EntityStyler`. It uses the same 12-color palette values but a different
  assignment algorithm (hash-based vs. sequential-by-appearance) — so an entity's color in TUI output is
  **not guaranteed to match** its color in CLI output for the same run. See [`tui.md`](tui.md#common-pitfalls).
  This is a known inconsistency, not an intentional design choice.

## Source map

- [`src/services/entity_parser.rs`](../../src/services/entity_parser.rs)
- [`src/services/entity_styler.rs`](../../src/services/entity_styler.rs)
- [`src/services/description_formatter.rs`](../../src/services/description_formatter.rs)
- [`src/services/color_mode.rs`](../../src/services/color_mode.rs)

## Related docs

- [`cli.md`](cli.md), [`tui.md`](tui.md) — consumers.
- [`flows/entity-rename.md`](../flows/entity-rename.md)
- [Glossary: Entity Reference, Alias, Color Mode, Description Preview](../glossary.md)
- [`flows/entity-alias-resolution.md`](../flows/entity-alias-resolution.md)
- [`../architecture/decisions/0013-entity-aliases.md`](../architecture/decisions/0013-entity-aliases.md)
