# Services

## Purpose

Pure business-logic helpers with no I/O or persistence dependencies — entity-reference parsing, entity
color styling, description-preview formatting, date parsing, and terminal color-mode detection — plus two
DB-touching helpers: `entity_resolution`, which ties `[bracket]` mention extraction to the persisted alias
registry, and `thought_writer`, which persists a new thought and its links. Reused by both the CLI and the
TUI.

## Questions this doc answers

- What syntax does an entity reference use, and how is it parsed?
- How are entities colored/styled in output?
- How are entity description previews generated?
- What date formats are accepted, and where is that decided?
- How is color output enabled/disabled?

## Scope

`src/services/color_mode.rs`, `date_parser.rs`, `entity_parser.rs`, `entity_styler.rs`,
`description_formatter.rs`, `entity_resolution.rs`, `thought_writer.rs`.

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
- `redirect_entity_references(text, old_name, new_target) -> String` — re-points references at a
  different entity while *keeping* the wording that was written: bare `[Old]` → `[Old](NewTarget)` and
  aliased `[Alias](old)` → `[Alias](NewTarget)`, collapsing to bare `[NewTarget]` when the display text
  already names the target. Used by entity merge (see [`flows/entity-merge.md`](../flows/entity-merge.md));
  the contrast with `rewrite_entity_references`'s wording-replacing behavior is deliberate and explained
  in [`../architecture/decisions/0014-entity-merge.md`](../architecture/decisions/0014-entity-merge.md).

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

`resolve_entity(conn, name) -> Result<Resolution, ThoughtError>` is the same logic without the printing:
it returns `Resolution::Ambiguous(AmbiguousMention)` for the caller to render. `resolve_or_create_entity`
is now a thin wrapper over it. **Anything that owns the terminal must use `resolve_entity`** — the
composer holds the tty in raw mode on the alternate screen, and an `eprintln!` there lands mid-frame with
no carriage return, which ratatui's diff-based redraw then leaves smeared for the rest of the session.
`AmbiguousMention::describe()` gives both paths identical wording.

**`thought_writer.rs`** — `create_thought(conn, content, date) -> Result<Created, ThoughtError>`,
the other storage-touching function here. Builds and validates a `Thought` (`date` of `None` timestamps it
with the current instant, `Some(date)` pins it to midnight UTC), then saves it and links each extracted
mention via `entity_resolution::resolve_entity` — all inside one transaction, so a failure part
way through leaves no orphan thought behind. Shared by `wet add` and the interactive composer, which is why
both cannot drift apart on what "adding a thought" means. Returns a `Created { id, entities, ambiguous }`: the new ID, the unique entity
names extracted from the content, and any mentions left unlinked because their alias was ambiguous.
Ambiguity is *returned* rather than printed so a caller that owns the terminal can render it itself —
see `entity_resolution` below.

**`date_parser.rs`** — `parse_date_from(input, today)` / `parse_date(input)`, the single definition of
what a date may look like anywhere in the app. Accepts, case-insensitively and trimmed: `YYYY-MM-DD`;
`today`/`t`, `yesterday`/`y`, `tomorrow`; signed offsets `-3d`, `-2w`, `-1m` (a leading `+` or no sign
moves forward); and weekday names in full or three-letter form, resolving to the most recent occurrence
at or before `today`. Month arithmetic clamps to the end of a shorter month, so `-1m` from March 31 is
the last day of February. Anything else is an `InvalidInput` naming the accepted forms, which are also
exported as `ACCEPTED_FORMS` for the composer's inline hint. `parse_date_from` takes `today` explicitly so
the behavior is testable without freezing the clock. Used by `wet add --date`, `wet edit --date`, and the
composer's date field.

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

Entity reference rewriting is the core of [`flows/entity-rename.md`](../flows/entity-rename.md) and
[`flows/entity-merge.md`](../flows/entity-merge.md).

## Data and state

`EntityStyler` holds an in-memory `color_map` for the duration of a single render pass — not persisted.

## Interfaces and entry points

`ColorMode::should_use_colors`, `entity_parser::{extract_entities, extract_unique_entities,
rewrite_entity_references, redirect_entity_references}`, `EntityStyler::{new, render_content}`,
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
- [`src/services/date_parser.rs`](../../src/services/date_parser.rs)
- [`src/services/entity_resolution.rs`](../../src/services/entity_resolution.rs)
- [`src/services/thought_writer.rs`](../../src/services/thought_writer.rs)

## Related docs

- [`cli.md`](cli.md), [`tui.md`](tui.md) — consumers.
- [`flows/entity-rename.md`](../flows/entity-rename.md)
- [`flows/add-thought-interactive.md`](../flows/add-thought-interactive.md)
- [Glossary: Entity Reference, Alias, Color Mode, Description Preview](../glossary.md)
- [`flows/entity-alias-resolution.md`](../flows/entity-alias-resolution.md)
- [`../architecture/decisions/0013-entity-aliases.md`](../architecture/decisions/0013-entity-aliases.md)
