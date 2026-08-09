# Add a Thought Interactively

## Purpose

Record a new thought through a guided full-screen composer that teaches the entry syntax as you type:
relative dates resolved live, fuzzy completion over existing entities and aliases, and a preview that
flags mentions which would create a brand-new entity.

## Trigger

`wet add -i`, or `wet add` with no `CONTENT` argument. An optional `--date` pre-fills the date field.

## Participants

- `cli/add.rs` — dispatches between the direct and interactive paths, owns terminal setup/teardown
- `tui/compose/` — `ComposeApp` state, key handling, rendering
- `tui/fuzzy.rs` — candidate scoring, shared with the browser TUI's entity picker
- `services::date_parser` — resolves the date field
- `services::thought_writer` — persists the thought and links its mentions
- `storage::{entities_repository, entity_aliases_repository}` — the completion candidates

## Step-by-step flow

1. `cli/add.rs::execute` sees no `CONTENT` and calls `compose_interactively`.
2. If stdin or stdout is not a terminal, it returns an `InvalidInput` error naming the non-interactive
   alternative, and stops — ratatui is never started.
3. A connection is opened and migrated, then `ComposeApp::new` loads the completion candidates:
   every entity's canonical name plus each of its registered aliases. Aliases carry the canonical name
   they point at. The lowercased set of all of them becomes `known_lower`.
4. `ratatui::init()` takes over the terminal and `ComposeApp::run` enters a blocking draw/`event::read`
   loop, the same shape as the browser TUI's `App::run`.
5. Each keystroke goes to `compose::input::handle_key_event`, which picks one of two key maps depending
   on whether the whisperer popup is open.
6. **Whisperer closed**: `Tab`/`Shift-Tab` switches focus between the date and content fields, `Enter`
   saves, `Ctrl-C` quits, everything else goes to the focused `tui_input::Input`. `Esc` quits too, but
   only straight away when the thought field is empty: with unsaved text it arms a confirmation
   (`pending_quit`) and says so in the status line, and any edit disarms it again. Discarding written
   text is the one destructive dismissal in the composer, so it is the one that asks.
   `Alt-Left`/`Alt-Right` shift the date a day earlier or later from wherever the focus is, so a
   one-day correction never costs a trip to the date field and back; the field is rewritten as an
   absolute `YYYY-MM-DD` so repeated presses compose.
7. **Typing `[` in the content field** opens the whisperer on the reference's display slot, anchored at
   that bracket's character index. The query is the text between the bracket and the cursor; it is scored
   against every candidate label by `fuzzy::match_indices`. An empty query offers everything.
8. **Typing `(` straight after a `]`** opens the whisperer on the *target* slot instead, titled
   `Entities → links to`. This covers the case where the display wording was written by hand
   (`met [my boss](…`) and so will never match a candidate; the target still gets completion, which is
   where a typo would otherwise create an unwanted entity. A `(` anywhere else is ordinary prose and does
   not trigger the popup.
9. **Whisperer open**: arrows move the highlight; `Tab`/`Enter` accepts; `Esc` dismisses the popup while
   keeping the typed text. Accepting in the display slot replaces the partial `[query` with
   `[Canonical]`, or `[alias](Canonical)` when a registered alias was chosen. Accepting in the target
   slot replaces `(query` with `(Canonical)` — a registered alias contributes only the entity it points
   at, since the wording is already written. Either way the cursor lands just past the inserted text.
   A space is appended so typing continues without reaching for one; it is skipped when the following
   text already begins with whitespace or with punctuation that should hug the reference
   (`[Alice].`, `[Alice]'s`). `ComposeApp::save` trims, so that convenience space never reaches storage.

   Entities whose canonical name contains a parenthesis are not offered in the target slot at all:
   `ENTITY_PATTERN`'s target group cannot span a nested paren, so `[my thing](Wetware (project))` would
   degrade to the traditional form and silently create an entity named after the display text. They stay
   reachable through the display slot, which tolerates parens.
10. The popup closes on its own when the reference is finished or abandoned: the slot's own closing
    character in the query (`]` for the display slot, `)` for the target slot), the cursor moving back to
    or past the opening bracket, or that bracket being deleted.
11. With no matches, the popup says so rather than blocking: `Enter` falls through to a save, so a
    brand-new entity name can be typed freely.
12. Every frame re-renders the preview from the raw content, resolving markup to display text and
    showing a `+new:` marker for mentions absent from `known_lower`. The marker occupies its own
    reserved row rather than trailing the text, so wrapping can never drop it — appended inline it
    disappeared exactly on the long thoughts where a typo is most likely. Both the content field and
    the preview word-wrap over as many rows as they need (`tui/compose/wrap.rs`), growing up to a cap
    and then scrolling to keep the cursor's row visible — text past the right edge is wrapped, never
    hidden.
13. `Enter` calls `ComposeApp::save`, which resolves the date field and delegates to
    `thought_writer::create_thought`.
14. On success the content field is cleared, the date field is kept, the save counter increments, and
    the candidates are reloaded so entities the save just created are immediately completable. The
    composer stays open.
15. `Esc` (confirmed, if there was unsaved text) sets `should_quit`, the loop ends, and `cli/add.rs`
    calls `ratatui::restore()` and prints how many thoughts were added. The count is printed *before*
    any `run` error is propagated: those saves are already committed, and a terminal failure must not
    leave the user unsure whether their work landed.

## Data and state changes

Each save inserts one row into `thoughts` and one row into `thought_entities` per resolved mention,
creating rows in `entities` for names that match nothing — all inside a single transaction (see
[`../systems/storage.md`](../systems/storage.md)). Nothing is written before `Enter`; abandoning the
composer with `Esc` leaves the database untouched.

An empty date field means "now" and stores a full timestamp; a filled one is pinned to midnight UTC on
the resolved date. This mirrors `wet add` with and without `--date`.

## Success behavior

Each `Enter` reports `Saved thought #N (K entities) · M saved this session` in the status line, and the
composer is ready for the next thought. On exit, `wet` prints the session total on stdout.

## Failure behavior

- **No terminal** — an `InvalidInput` error before any terminal setup, pointing at `wet add "..."`.
- **Unparseable date** — the date field renders `→ unrecognized` with the accepted forms, in red, as you
  type. `Enter` refuses to save and repeats the message in the status line; the content is untouched.
- **Empty or oversized content** — `Thought::new`'s validation error lands in the status line and the
  content is preserved so it can be corrected.
- **Ambiguous alias** — the mention is skipped rather than failing the save (see
  [`entity-alias-resolution.md`](entity-alias-resolution.md)), and the composer reports it in its own
  status line: `Saved thought #N, but 'sar' matches multiple entities (…)`. The composer calls
  `entity_resolution::resolve_entity`, which *returns* the ambiguity, not `resolve_or_create_entity`,
  which prints it. Printing would be wrong here: stderr and the alternate screen are the same tty, so
  the warning lands mid-frame with no carriage return, and because ratatui redraws only diffs the smear
  survives for the rest of the session.
- **Storage failure mid-save** — the transaction rolls back, so no partially-linked thought survives;
  the error appears in the status line and the composer stays open.

## External dependencies

A terminal supporting ratatui/crossterm. The database, per
[`../systems/storage.md`](../systems/storage.md).

## Invariants and assumptions

- `cli/add.rs` always calls `ratatui::restore()` after `run` returns, whether it succeeded or errored —
  the same explicit-teardown contract as `cli/tui.rs`, not a `Drop` impl.
- `Whisperer::open_at` and `tui_input::Input::cursor` are both **codepoint** indices; mixing them with
  byte offsets would split multibyte characters.
- The candidate list and `known_lower` are only refreshed at startup and after a save. An entity created
  by a concurrent `wet` invocation is not offered until the next save.
- Accepting a completion rebuilds the whole `Input`, so any pending selection state in it is discarded.

## Security and privacy notes

Nothing beyond [`../systems/storage.md`](../systems/storage.md)'s notes on local data sensitivity.

## Observability and debugging

Save failures surface in the composer's status line, unlike the browser TUI's silently-swallowed delete
errors. Warnings printed to stderr by `entity_resolution` are hidden by the alternate screen and only
appear after exit. To reproduce composer behavior without a terminal, drive `handle_key_event` directly —
that is what the unit tests do.

## Testing notes

- `src/tui/compose/input.rs` — key-handling tests covering the whisperer lifecycle (open, narrow, accept
  canonical vs. alias, all four dismissal paths) and the field key map.
- `src/tui/compose/ui.rs` — `TestBackend` render assertions for date resolution, the `+new` marker, the
  popup contents, box growth and its cap, scrolling to the cursor, and narrow/multibyte terminals.
- `src/tui/compose/wrap.rs` — wrapping in isolation: space vs. hard breaks, contiguous rows, cursor
  location including row boundaries, and wide glyphs.
- `src/tui/compose/mod.rs` — save semantics, including candidate reload after a save.
- `tests/contract/test_add_command.rs` — that `wet add` without a terminal fails with a clear message,
  and that relative `--date` values reach the CLI.

## Source map

- [`src/cli/add.rs`](../../src/cli/add.rs)
- [`src/tui/compose/mod.rs`](../../src/tui/compose/mod.rs)
- [`src/tui/compose/input.rs`](../../src/tui/compose/input.rs)
- [`src/tui/compose/ui.rs`](../../src/tui/compose/ui.rs)
- [`src/tui/compose/state.rs`](../../src/tui/compose/state.rs)
- [`src/tui/compose/wrap.rs`](../../src/tui/compose/wrap.rs)
- [`src/services/date_parser.rs`](../../src/services/date_parser.rs)
- [`src/services/thought_writer.rs`](../../src/services/thought_writer.rs)

## Related docs

- [`../systems/cli.md`](../systems/cli.md), [`../systems/tui.md`](../systems/tui.md),
  [`../systems/services.md`](../systems/services.md)
- [`edit-thought.md`](edit-thought.md), [`entity-alias-resolution.md`](entity-alias-resolution.md)
- [`../architecture/decisions/0015-interactive-thought-composer.md`](../architecture/decisions/0015-interactive-thought-composer.md)
