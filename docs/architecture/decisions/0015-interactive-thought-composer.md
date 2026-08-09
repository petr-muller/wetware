---
status: Accepted
date: "2026-08-09"
---

# Interactive thought composer with entity whisperer and relative dates

## Context

`wet add "content" --date YYYY-MM-DD` was the only way to record a thought, and it gave no help with
the two things that are hardest to get right at the moment of writing:

- **Entity mention syntax.** `[Entity]` and `[alias](Entity)` are documented in `--help` and in
  [`../../systems/services.md`](../../systems/services.md), but nothing at the point of entry shows
  which entities already exist. Because `entity_resolution::resolve_or_create_entity` creates an entity
  on any miss, a typo silently produces a near-duplicate that must later be cleaned up with
  `wet entity merge` (see [`0014-entity-merge.md`](0014-entity-merge.md)).
- **Dates.** Only strict `YYYY-MM-DD` was accepted, and the parsing was duplicated inline in
  `cli/add.rs` and `cli/edit.rs`. Backdating a thought by a couple of days meant looking up a calendar.

Thoughts are entered often and in small quantities, so the friction here compounds.

## Decision

Add an interactive composer, reached via `wet add -i` or bare `wet add` with no content argument,
built as `src/tui/compose/` alongside the existing browser TUI and following the same Elm-style
state/input/ui split established in [`0006-tui-viewer.md`](0006-tui-viewer.md).

The composer presents a date field, a content field, a live preview, and a persistent syntax help
footer. Typing `[` in the content field opens a fuzzy "whisperer" popup over every known entity
canonical name **and** alias; accepting a canonical name inserts `[Name]`, accepting an alias inserts
`[alias](Name)`.

The whisperer has a second trigger, for the case the first one cannot serve. `entity_parser`'s
`[wording](target)` syntax is free-form per-occurrence display text (see
[`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md)), so wording written by hand —
`met [my boss](…` — will never fuzzy-match a candidate. Typing `(` **directly after a `]`** therefore
opens the popup on the *target* slot instead, where accepting writes only `(Canonical)` and leaves the
wording alone. Without this, the one place a typo silently creates an unwanted entity is precisely the
place the whisperer was not firing. `(` elsewhere is ordinary prose and does not trigger it.

Text that matches nothing is left alone, so new entities can still be created by
typing freely — the preview marks those mentions `+new` so the choice is deliberate rather than
accidental. Saving clears the content but keeps the date, so a run of thoughts can be entered in one
sitting.

Two supporting decisions fall out of this:

- **Date parsing becomes a shared service**, `services::date_parser`, accepting `YYYY-MM-DD`,
  `today`/`yesterday`/`tomorrow`, signed offsets (`-3d`, `-2w`, `-1m`), and weekday names. It backs the
  composer's date field *and* `wet add --date` / `wet edit --date`, so the CLI and the composer can never
  drift apart on what a date looks like.
- **Thought creation becomes a shared service**, `services::thought_writer::create_thought`, wrapping
  the save-and-link sequence in a transaction. `wet add` previously did this non-transactionally, unlike
  `wet edit`; sharing the path with the composer fixed that asymmetry as a side effect.

## Consequences

- The syntax is discoverable at the moment it is needed, and near-duplicate entities from typos become
  visible before they are written rather than after.
- `Alt-Left`/`Alt-Right` shift the date by a day from either field. A separate date field is the right
  home for the *value*, but tabbing over and back to say "actually, yesterday" costs more than the edit
  is worth, so the common adjustment gets a binding that skips the round-trip entirely. The cost is one
  more thing to know about; the help footer carries it, and the two-field model is unchanged.
- Relative dates now work everywhere a date is accepted, not just in the composer. `--date` gained
  `allow_hyphen_values` so `--date -3d` parses as a value rather than an unknown flag; the cost is that
  `--date --editor` would consume `--editor` as a (rejected) date string.
- `wet add`'s `CONTENT` argument became optional. This is backward compatible for every existing
  invocation, but it means `wet add` with a typo'd flag now opens the composer instead of erroring.
- The composer needs a TTY. Without one it returns a plain `InvalidInput` explaining that content should
  be passed as an argument, rather than attempting to start ratatui against a pipe.
- Content is a single *logical* line — `tui_input::Input` holds no newlines, and thoughts are brief
  snippets by design, so multi-paragraph entry remains the job of `wet edit --editor` (see
  [`0005-edit-thoughts.md`](0005-edit-thoughts.md)). It is nonetheless **word-wrapped for display** over
  as many rows as it needs: a single rendered line silently hid everything past the right edge, which is
  unusable for anything but the shortest note. Wrapping lives in `tui/compose/wrap.rs` rather than
  leaning on `Paragraph::wrap`, because the cursor has to be placed at the wrapped position and deriving
  it from a second, independent wrap implementation would eventually disagree with what was drawn.
  `unicode-width` became a direct dependency for this; it was already compiled in the tree via ratatui,
  so it costs no additional build.
- The composer holds one database connection open for its whole session, unlike the browser `App`, which
  reopens one per mutation. It writes repeatedly, so a connection per keystroke-batch would be wasteful.
- Two helpers were lifted out of the browser TUI to be shared: `tui::fuzzy` (previously inline in the
  entity picker) and `tui::ui::styled_content_line`. The latter's truncation was byte-sliced and would
  panic on a multibyte character at the boundary — harmless in the browser, where content comes from the
  database, but the composer feeds it live keystrokes, so it was made char-boundary safe.

## Alternatives considered

- **An `a` key inside `wet tui`.** Rejected as the primary entry point: `wet tui` loads all thoughts,
  entities, and relations upfront and is built for browsing, so adding a write mode would entangle two
  concerns and require refreshing the loaded data set mid-session. A separate composer keeps both simple.
  Wiring one into `wet tui` later remains possible.
- **A date prefix inside the content line** (`2026-08-01 met with [Alice]`, stripped on save). Rejected as
  ambiguous — a thought may legitimately begin with a date — and undiscoverable.
- **Requiring the whisperer's selection**, i.e. only allowing existing entities. Rejected: creating an
  entity by simply mentioning it is the core of the bracket-markup design
  (see [`0001-networked-notes-schema.md`](0001-networked-notes-schema.md)); the `+new` marker gives
  visibility without taking that away.
- **An explicit completion key** (Ctrl-E) instead of triggering on `[`. Rejected as a worse default: `[`
  is already the character you type when you mean "an entity goes here", so it carries the intent
  unambiguously. Esc dismisses the popup for anyone who wants a literal bracket.
- **A date-parsing crate** such as `chrono-english` or `dateparser`. Rejected per the project's
  preference for lightweight, focused dependencies: the handful of forms wanted here is a small amount of
  code over `chrono`, which is already a dependency.

## Related code

- [`src/tui/compose/`](../../../src/tui/compose/) — the composer
- [`src/services/date_parser.rs`](../../../src/services/date_parser.rs)
- [`src/services/thought_writer.rs`](../../../src/services/thought_writer.rs)
- [`src/tui/fuzzy.rs`](../../../src/tui/fuzzy.rs)
- [`src/cli/add.rs`](../../../src/cli/add.rs)

## Related docs

- [`../../systems/tui.md`](../../systems/tui.md), [`../../systems/cli.md`](../../systems/cli.md),
  [`../../systems/services.md`](../../systems/services.md)
- [`../../flows/add-thought-interactive.md`](../../flows/add-thought-interactive.md)
- [`0006-tui-viewer.md`](0006-tui-viewer.md), [`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md),
  [`0013-entity-aliases.md`](0013-entity-aliases.md)
