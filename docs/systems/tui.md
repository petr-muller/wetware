# TUI

## Purpose

The interactive terminal viewer (`wet tui`) for browsing, filtering, sorting, and deleting thoughts —
built on `ratatui` with a classic Elm-style state/input/ui split.

## Questions this doc answers

- How is the TUI structured (state machine, event loop)?
- What does each `Mode` do?
- Why might an entity's color differ between the CLI and the TUI?

## Scope

`src/tui/mod.rs`, `state.rs`, `input.rs`, `ui.rs`.

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
    pub should_quit: bool,
    pub db_path: Option<PathBuf>,
}
```

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
- `delete_selected_thought(&mut self)` — only acts if `mode == ConfirmDelete`; deletes from the DB (opens
  its own connection, runs migrations) and from the in-memory `thoughts` list, then resets `mode` to
  `Normal`. No re-query of the database afterward.
- `recompute_displayed_thoughts(&mut self)` — re-filters (by `active_filter`, via
  `entity_parser::extract_entities`) and re-sorts indices per `sort_order`; clamps/reselects the list
  selection safely.
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
- **EntityPicker** — `Esc` cancels; `Enter` applies `active_filter` = the selected entity's canonical name
  and recomputes; arrows move `selected` within `matches`; any other key forwards to
  `tui_input::Input::handle_event`, then recomputes fuzzy matches via `nucleo_matcher` (`Pattern::new`
  with `CaseMatching::Ignore`, `Normalization::Smart`, `AtomKind::Fuzzy`), scored/sorted descending,
  resetting `selected` to 0.
- **EntityDetail** — `Esc` closes; arrows adjust `scroll_offset` (saturating).

`ui.rs` — pure rendering, `render(app, frame)`: splits the screen into a thought list (min 3 rows) + a
1-row status bar, then overlays the active mode's popup (`ConfirmDelete`/`EntityPicker`/`EntityDetail`)
via `Clear` + a centered `Rect`. Also implements its own entity color assignment — see Common Pitfalls.

## Important flows

- [`flows/tui-entity-filter.md`](../flows/tui-entity-filter.md)
- [`flows/delete-thought.md`](../flows/delete-thought.md)

## Data and state

The TUI loads all thoughts and entities **once at startup** (`cli/tui.rs`); it does not re-query the
database during the session. Deletions mutate in-memory state directly and also delete from the DB.

## Interfaces and entry points

`App::new`, `App::with_db_path`, `App::run`; launched via `wet tui` ([`cli.md`](cli.md)).

## Dependencies

`errors`, `models::{Entity, SortOrder, Thought}`, `services::entity_parser`,
`storage::{connection, migrations, thoughts_repository}`, `ratatui`, `tui_input`, `nucleo_matcher`,
`owo_colors`.

## Downstream effects

Deleting a thought here writes to the same database CLI commands use — a concurrent CLI invocation during
a TUI session could observe a delete after the fact, but there's no locking beyond SQLite's own.

## Invariants and assumptions

- `App::run` always calls `ratatui::restore()` on return from `cli/tui.rs`, whether `run()` succeeded or
  errored — but this relies on the caller doing so explicitly, not on a `Drop` impl on `App` itself, and
  doesn't cover a hard process panic (relies on `ratatui`'s own panic hook, if any, not an explicit
  `catch_unwind`).
- `displayed_thoughts` must be recomputed any time `active_filter`, `sort_order`, or `thoughts` changes —
  it is not automatically kept in sync.

## Error handling

`delete_selected_thought` returns `Result<(), ThoughtError>`; the `ConfirmDelete` key handler swallows an
error by falling back to `Normal` mode silently (no error message shown to the user in-TUI).

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
- [`src/cli/tui.rs`](../../src/cli/tui.rs) — startup/data loading.

## Related docs

- [`services.md`](services.md), [`storage.md`](storage.md), [`cli.md`](cli.md)
- [`flows/tui-entity-filter.md`](../flows/tui-entity-filter.md), [`flows/delete-thought.md`](../flows/delete-thought.md)
- [`../architecture/decisions/0006-tui-viewer.md`](../architecture/decisions/0006-tui-viewer.md)
