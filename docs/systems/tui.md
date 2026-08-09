# TUI

## Purpose

The interactive terminal viewer (`wet tui`) for browsing, filtering, sorting, and deleting thoughts, and
the interactive composer (`wet add -i`) for entering new ones — both built on `ratatui` with a classic
Elm-style state/input/ui split.

## Questions this doc answers

- How is the TUI structured (state machine, event loop)?
- What does each `Mode` do?
- How does the composer's entity whisperer decide what to offer and when to close?
- Why might an entity's color differ between the CLI and the TUI?

## Scope

`src/tui/mod.rs`, `state.rs`, `input.rs`, `ui.rs`, `fuzzy.rs`, and `src/tui/compose/`.

## Non-scope

Loading the initial data set (`cli/tui.rs`, see [`cli.md`](cli.md)); entity-reference parsing itself (see
[`services.md`](services.md)).

## Key concepts

- **Mode** — see [glossary](../glossary.md#mode): `Normal`, `EntityPicker`, `ConfirmDelete`,
  `EntityDetail`.
- **Active Filter** — see [glossary](../glossary.md#active-filter).
- **Displayed Thoughts** — see [glossary](../glossary.md#displayed-thoughts).

## How the system works

Central state, `App` (`mod.rs`):

```rust
pub struct App {
    pub thoughts: Vec<Thought>,
    pub entities: Vec<Entity>,
    pub displayed_thoughts: Vec<usize>,   // filtered+sorted indices into `thoughts`
    pub list_state: ratatui::widgets::ListState,
    pub mode: Mode,
    pub sort_order: SortOrder,
    pub active_filter: Option<String>,
    pub active_filter_reachable: HashSet<String>,
    pub should_quit: bool,
    pub db_path: Option<PathBuf>,
    entity_children: HashMap<i64, Vec<i64>>,
}
```

`active_filter` holds the filter entity's display name (for the status bar); `active_filter_reachable`
holds the lowercase names of that entity and every entity transitively reachable from it via child
relations — this is the set actually tested against a thought's extracted entity names when filtering.
`entity_children` is a parent-id → child-ids adjacency map built once from the relation edges loaded at
startup (see [`../architecture/decisions/0012-entity-relations.md`](../architecture/decisions/0012-entity-relations.md)).

`state.rs` defines `Mode` as pure data, no logic:

```rust
enum Mode {
    Normal,
    EntityPicker { input: tui_input::Input, matches: Vec<usize>, selected: usize },
    ConfirmDelete { thought_index: usize },
    EntityDetail { entity_indices: Vec<usize>, scroll_offset: usize },
}
```

`App` methods (`mod.rs`):
- `App::new(thoughts, entities, sort_order)` — builds initial `displayed_thoughts` via
  `recompute_displayed_thoughts()`, selects index 0 if non-empty.
- `with_db_path(self, db_path)` — builder-style setter.
- `with_relations(self, relations: Vec<(i64, i64)>)` — builder-style setter; builds `entity_children` from
  `(child_id, parent_id)` edges loaded once at startup (see [`cli.md`](cli.md)'s `tui.rs` notes).
- `reachable_names(&self, root_idx: usize) -> HashSet<String>` — depth-first walk of `entity_children`
  starting at `entities[root_idx]`, returning the lowercase names of that entity and every descendant.
  Called once when an entity is picked (`Enter` in `EntityPicker` mode), not on every keystroke or every
  `recompute_displayed_thoughts` call.
- `delete_selected_thought(&mut self)` — only acts if `mode == ConfirmDelete`; deletes from the DB (opens
  its own connection, runs migrations) and from the in-memory `thoughts` list, then resets `mode` to
  `Normal`. No re-query of the database afterward. `ON DELETE CASCADE` handles `thought_entities` cleanup
  in the DB, same as the CLI's `delete.rs` (see [`cli.md`](cli.md)). Afterward,
  `recompute_displayed_thoughts` re-derives the list and clamps the selection to stay valid (the same
  index if possible, otherwise the previous one, or `None` if the list is now empty).
- `recompute_displayed_thoughts(&mut self)` — re-filters and re-sorts indices per `sort_order`; clamps/
  reselects the list selection safely. Filtering (when `active_filter` is set) extracts each thought's
  entity names via `entity_parser::extract_entities` and keeps the thought if any of those names appears
  in `active_filter_reachable` — i.e. the thought is tagged with the filter entity itself or any of its
  descendants.
- `selected_thought_entity_indices(&self)` — maps entities referenced in the currently-selected thought to
  indices in `App::entities`.
- `run(&mut self, terminal)` — the event loop: draw via `ui::render`, block on `event::read()`, dispatch
  key-press events to `input::handle_key_event`, repeat until `should_quit`.

`input.rs` — `handle_key_event(app, key)` dispatches by `app.mode` to one of four handlers:

- **Normal** — `q`/`Esc` quit (`Esc` clears an active filter first, if set); arrows/`PageUp`/`PageDown`/
  `Home`/`End` navigate the list; `s` toggles sort and recomputes; `/` opens `EntityPicker` (seeded with
  all entity indices); `Enter`/`d` opens `EntityDetail` for the selected thought's entities (no-op if
  none); `x` opens `ConfirmDelete` for the selected thought.
- **ConfirmDelete** — `y`/`Y` calls `delete_selected_thought()` (falls back to `Normal` silently on
  error); `n`/`N`/`Esc` cancels back to `Normal`.
- **EntityPicker** — `Esc` cancels; `Enter` sets `active_filter` to the selected entity's canonical name,
  computes `active_filter_reachable` via `reachable_names` (the entity plus every descendant), and
  recomputes; arrows move `selected` within `matches`; any other key forwards to
  `tui_input::Input::handle_event`, then recomputes fuzzy matches via `nucleo_matcher` (`Pattern::new`
  with `CaseMatching::Ignore`, `Normalization::Smart`, `AtomKind::Fuzzy`), scored/sorted descending,
  resetting `selected` to 0.
- **EntityDetail** — `Esc` closes; arrows adjust `scroll_offset` (saturating).

`ui.rs` — pure rendering, `render(app, frame)`: splits the screen into a thought list (min 3 rows) + a
1-row status bar, then overlays the active mode's popup (`ConfirmDelete`/`EntityPicker`/`EntityDetail`)
via `Clear` + a centered `Rect`. Also implements its own entity color assignment — see Common Pitfalls.
Its `styled_content_line` is `pub(crate)` and shared with the composer's preview.

`fuzzy.rs` — `score_all(query, haystacks)` / `match_indices(query, haystacks)` wrap `nucleo_matcher`
(`CaseMatching::Ignore`, `Normalization::Smart`, `AtomKind::Fuzzy`), sorted by descending score with
input order preserved on ties. Both the browser's entity picker and the composer's whisperer score
through it, so their matching behavior cannot drift apart. An empty query matches everything, in order.

### The composer (`src/tui/compose/`)

A second, independent ratatui app reached via `wet add -i` (or bare `wet add`), with the same
`mod`/`state`/`input`/`ui` split. See
[`../architecture/decisions/0015-interactive-thought-composer.md`](../architecture/decisions/0015-interactive-thought-composer.md)
for why it is separate from the browser rather than an `a` key inside it.

```rust
pub struct ComposeApp {
    pub content: tui_input::Input,
    pub date: tui_input::Input,
    pub focus: Field,                  // Date | Content
    pub candidates: Vec<Candidate>,    // canonical names *and* aliases
    pub known_lower: HashSet<String>,  // for the "+new" preview marker
    pub whisperer: Option<Whisperer>,
    pub status: Option<Status>,        // Saved { id, entities } | Error(String)
    pub saved_count: usize,
    pub should_quit: bool,
    pub pending_quit: bool,            // an Esc on unsaved text, awaiting confirmation
    conn: Connection,
}
```

Unlike `App`, which is handed pre-loaded data and reopens a connection per mutation, `ComposeApp` owns
one connection for its whole session and loads its own candidates — it writes repeatedly, and each save
must make newly created entities completable right away.

`compose/state.rs` holds pure data: `Field`, `Candidate { label, canonical, is_alias, description }`,
`Slot` (`Display` | `Target`), and `Whisperer { open_at, slot, matches, selected }`.
`Candidate::insertion(slot)` yields `[Name]` or `[alias](Name)` in the `Display` slot, and bare
`(Name)` in the `Target` slot.

`compose/input.rs` — `handle_key_event` picks one of two key maps. With the whisperer closed:
`Tab`/`BackTab` switch focus, `Enter` saves, `Esc`/`Ctrl-C` quit, anything else goes to the focused
input. With it open: arrows move the highlight, `Tab`/`Enter` accept, `Esc` dismisses while keeping the
typed text, anything else edits the content and re-runs `recompute_whisperer`.

`Esc` with no popup open quits only when the thought field is empty; with unsaved text it arms
`pending_quit` and reports it in the status line, and any other key disarms it. Discarding written text
is the composer's one destructive dismissal, so it is the one that asks first.

Two bindings sit above both maps and fire from anywhere: `Ctrl-C` quits, and `Alt-Left`/`Alt-Right` call
`ComposeApp::nudge_date(±1)`, shifting the date a day without moving focus — a one-day correction should
not cost a field round-trip. `tui_input` leaves `Alt`-arrows unbound (it claims the `Ctrl` variants for
word movement), so nothing is clobbered. `nudge_date` rewrites the field as an absolute `YYYY-MM-DD` so
repeated nudges compose; an empty field counts as today, and a field that does not parse is left alone
rather than overwriting a half-typed value.

`recompute_whisperer` is the whole popup lifecycle, and there are two triggers:

- **`[` opens the `Display` slot**, anchored at that bracket. Accepting writes a whole reference.
- **`(` typed straight after a `]` opens the `Target` slot**, anchored at the paren. This is the path for
  a reference whose display wording was written by hand — `met [my boss](…` — where the wording will
  never fuzzy-match anything. Accepting writes only `(Canonical)`, leaving the wording alone; picking a
  registered alias here contributes its canonical name, never `[my boss](alicia)`. `(` only triggers
  directly after a `]`, so ordinary parenthetical prose does not open the popup.

While open, the query is the content between the opening bracket and the cursor, scored via `fuzzy`. It
closes when the reference is finished or abandoned: the slot's own closing character in the query
(`]` for `Display`, `)` for `Target`), the cursor moving back to or past the opening bracket, or that
bracket being deleted. With no matches the popup says so and lets `Enter` fall through to a save, so
free-form entity names still work.

`compose/wrap.rs` — `wrap(text, cursor, width) -> Wrapped { rows, cursor_row, cursor_col }`, the
composer's word wrapping. `tui_input::Input` holds one logical line with no newlines, but a thought
easily runs past the right edge, so the content is laid out over visual rows here. Rows are half-open
character ranges that **tile the content exactly** — every character belongs to exactly one row, which
is what makes mapping a cursor index onto a row unambiguous. Breaks prefer the last space on a row
(kept at that row's end, so the ranges stay contiguous); a word longer than the field is broken hard.
Widths are display columns via `unicode-width`, not character counts, so wide glyphs cannot overflow
and be clipped. Rendering and cursor placement both consume one `Wrapped`, so they cannot disagree.

`compose/mod.rs`'s `selectable_candidates(slot)` filters what the whisperer may offer: in the target
slot it drops entities whose canonical name contains a paren, since `ENTITY_PATTERN`'s target group
cannot span one and accepting such a candidate would silently misparse into a new entity.

`compose/ui.rs` — a vertical layout of date field, content field, preview, status, and a two-line
syntax help footer. The date field re-parses on every frame and shows the resolved date or the accepted
forms in red; when the input is already an absolute `YYYY-MM-DD` — which it always is after a nudge —
it shows only the weekday rather than echoing the date back. The content and preview boxes **grow with
their text**, capped at `CONTENT_MAX_ROWS` / `PREVIEW_MAX_ROWS` so the footer is never pushed off
screen; past the content cap the field scrolls to keep the cursor's row in view. The preview wraps via
`Paragraph::wrap` and therefore calls `styled_content_line` with an effectively unlimited width — it
must not truncate, since wrapping is what shows the rest. The `+new:` marker gets its own reserved row
below the text rather than trailing it: appended inline it was the first thing wrapping dropped, so it
vanished on exactly the long thoughts where a typo is most likely. The preview runs the raw content through `ui::styled_content_line` and appends a
`+new:` list of mentions missing from `known_lower`. The whisperer popup tracks the cursor's column but
hangs below the preview, so the preview stays readable while completing.

## Important flows

- [`../flows/tui-entity-filter.md`](../flows/tui-entity-filter.md)
- [`../flows/add-thought-interactive.md`](../flows/add-thought-interactive.md)

Thought deletion (`x` + confirm overlay, vs. the CLI's unconfirmed `wet delete`) is covered inline above
and in [`cli.md`](cli.md#how-the-system-works), rather than as a standalone flow doc.

## Data and state

The TUI loads all thoughts, entities, and relation edges **once at startup** (`cli/tui.rs`); it does not
re-query the database during the session. Deletions mutate in-memory state directly and also delete from
the DB. A relation added or removed via the CLI mid-session is not reflected until the TUI restarts.

The composer likewise snapshots its candidates, but refreshes them after every save. An entity created by
a concurrent `wet` invocation is not offered until the next save.

## Interfaces and entry points

`App::new`, `App::with_db_path`, `App::with_relations`, `App::run`; launched via `wet tui`
([`cli.md`](cli.md)). `ComposeApp::new`, `ComposeApp::run`; launched via `wet add -i`.

## Dependencies

`errors`, `models::{Entity, SortOrder, Thought}`, `services::entity_parser`,
`storage::{connection, migrations, thoughts_repository, entity_relations_repository}`, `ratatui`,
`tui_input`, `nucleo_matcher`, `owo_colors`.

## Downstream effects

Deleting a thought here writes to the same database CLI commands use — a concurrent CLI invocation during
a TUI session could observe a delete after the fact, but there's no locking beyond SQLite's own.

## Invariants and assumptions

- `cli/tui.rs::execute` (not `App::run` itself) always calls `ratatui::restore()` after `App::run`
  returns, whether `run()` succeeded or errored — this relies on the caller doing so explicitly, not on a
  `Drop` impl on `App`, and doesn't cover a hard process panic (relies on `ratatui`'s own panic hook, if
  any, not an explicit `catch_unwind`). `cli/add.rs` carries the same contract for `ComposeApp::run`.
- `displayed_thoughts` must be recomputed any time `active_filter`, `sort_order`, or `thoughts` changes —
  it is not automatically kept in sync.
- `Whisperer::open_at` and `tui_input::Input::cursor` are both **codepoint** indices. Anything that
  slices content by them must convert to byte offsets, or it will split multibyte characters.
- `wrap::Row` ranges are also codepoint indices, and must stay contiguous and cover the whole content:
  `locate_cursor` assumes every character belongs to exactly one row.
- The rendered content rows and the cursor position must come from the **same** `wrap::Wrapped` value.
  Recomputing either independently reintroduces the drift the module exists to prevent.

## Error handling

`delete_selected_thought` returns `Result<(), ThoughtError>`; the `ConfirmDelete` key handler swallows an
error by falling back to `Normal` mode silently (no error message shown to the user in-TUI).

The composer is the opposite: `ComposeApp::save` records every failure in `status` and renders it in the
status line, keeping the content so it can be corrected.

## Security and privacy notes

Not applicable beyond [`storage.md`](storage.md)'s notes on local data sensitivity.

## Observability and debugging

If a delete silently appears to do nothing, check for a swallowed `Result` in
`handle_confirm_delete_mode` (see Error Handling above) — there's no visible error surface in the TUI
itself.

## Testing notes

`App`'s pure state-mutation methods (`recompute_displayed_thoughts`, `selected_thought_entity_indices`)
are testable without a terminal; `input.rs`'s handlers are testable by constructing `App` + `KeyEvent`
directly. Rendering (`ui.rs`) is harder to test and typically verified manually.

The composer is covered more thoroughly: `compose/mod.rs`'s `test_support` builds a `ComposeApp` over an
in-memory database seeded with an entity, an alias, and a description, and drives real keystrokes through
`handle_key_event`. `compose/ui.rs` renders through `ratatui::backend::TestBackend` and asserts on the
flattened buffer, including narrow-terminal and multibyte cases.

## Common pitfalls

- **`ui.rs`'s `entity_color`/`entity_color_index` use a separate, hash-based color-assignment algorithm**
  from `services::entity_styler::EntityStyler`'s sequential, order-of-appearance assignment. Both use the
  same 12-color palette values, but the assignment logic differs, so **an entity's TUI color is not
  guaranteed to match its CLI color** in the same session. See [`services.md`](services.md#common-pitfalls).
- The status bar advertises `?:Help` (`q:Quit  /:Filter  s:Sort  x:Delete  Enter:Details  ?:Help`), but no
  handler is wired to the `?` key anywhere in `input.rs` — it's a dead UI affordance, not a bug you
  introduced if you don't see a help overlay.

## Source map

- [`src/tui/mod.rs`](../../src/tui/mod.rs)
- [`src/tui/state.rs`](../../src/tui/state.rs)
- [`src/tui/input.rs`](../../src/tui/input.rs)
- [`src/tui/ui.rs`](../../src/tui/ui.rs)
- [`src/tui/fuzzy.rs`](../../src/tui/fuzzy.rs) — shared candidate scoring.
- [`src/tui/compose/mod.rs`](../../src/tui/compose/mod.rs) — composer state and event loop.
- [`src/tui/compose/state.rs`](../../src/tui/compose/state.rs)
- [`src/tui/compose/input.rs`](../../src/tui/compose/input.rs) — whisperer lifecycle.
- [`src/tui/compose/ui.rs`](../../src/tui/compose/ui.rs)
- [`src/tui/compose/wrap.rs`](../../src/tui/compose/wrap.rs) — word wrapping and cursor location.
- [`src/cli/tui.rs`](../../src/cli/tui.rs) — startup/data loading.
- [`src/cli/add.rs`](../../src/cli/add.rs) — composer startup/teardown.

## Related docs

- [`services.md`](services.md), [`storage.md`](storage.md), [`cli.md`](cli.md)
- [`../flows/tui-entity-filter.md`](../flows/tui-entity-filter.md)
- [`../flows/add-thought-interactive.md`](../flows/add-thought-interactive.md)
- [`../architecture/decisions/0006-tui-viewer.md`](../architecture/decisions/0006-tui-viewer.md)
- [`../architecture/decisions/0012-entity-relations.md`](../architecture/decisions/0012-entity-relations.md)
- [`../architecture/decisions/0015-interactive-thought-composer.md`](../architecture/decisions/0015-interactive-thought-composer.md)
