# TUI Entity Filter

## Purpose

Let a user narrow the TUI's thought list down to those referencing a specific entity — or any entity
transitively reachable from it via child relations (its descendants) — chosen via a fuzzy-searchable
picker.

## Trigger

Pressing `/` while the TUI is in `Normal` mode.

## Participants

- `tui/state.rs` (`Mode::EntityPicker`)
- `tui/input.rs` (`handle_entity_picker_mode`)
- `tui/mod.rs` (`App::recompute_displayed_thoughts`, `App::reachable_names`)
- `services/entity_parser.rs` (`extract_entities`)
- `tui_input` crate (text input widget)
- `nucleo_matcher` crate (fuzzy matching)
- `storage/entity_relations_repository.rs` (indirectly — supplies the relation edges `App` loads at
  startup, see [`../systems/tui.md`](../systems/tui.md))

## Step-by-step flow

1. `/` in `Normal` mode opens `Mode::EntityPicker { input, matches, selected }`, seeded with all entity
   indices as initial `matches`.
2. Each keystroke (other than navigation/`Enter`/`Esc`) is forwarded to `tui_input::Input::handle_event`
   to update the query text, then `matches` is recomputed: `nucleo_matcher::Pattern::new(query,
   CaseMatching::Ignore, Normalization::Smart, AtomKind::Fuzzy)` scores every entity name, results are
   sorted descending by score, and `selected` resets to 0.
3. Up/Down move `selected` within the current `matches` list.
4. `Enter` sets `App::active_filter` to the selected entity's canonical name, computes
   `App::active_filter_reachable` via `reachable_names` (the selected entity plus every entity
   transitively reachable from it via child relations), and calls `recompute_displayed_thoughts`, which
   filters `thoughts` to those whose content references any entity in `active_filter_reachable` (via
   `entity_parser::extract_entities`) and re-sorts per `sort_order`, then returns to `Normal` mode.
5. `Esc` cancels the picker without changing `active_filter`, returning to `Normal` mode.

## Data and state changes

`App::active_filter`, `App::active_filter_reachable`, and `App::displayed_thoughts` are updated
in-memory; nothing is persisted to the database.

## Success behavior

`displayed_thoughts` contains only the indices of thoughts referencing the selected entity or any of its
descendants, in the current sort order; the list selection is clamped/reset to remain valid.

## Failure behavior

No explicit failure modes — an empty fuzzy match result simply yields an empty `matches` list, and `Enter`
with nothing selected is a no-op.

## External dependencies

None (both `tui_input` and `nucleo_matcher` are local, in-process crates, not external services).

## Invariants and assumptions

`recompute_displayed_thoughts` must be called any time `active_filter` changes — it is not automatically
kept in sync by any other mechanism.

## Security and privacy notes

Not applicable.

## Observability and debugging

If filtering seems to return the wrong thoughts, check whether the entity name being matched against is
the canonical name vs. what `extract_entities` returns (always the lowercased target name, per
[`../systems/services.md`](../systems/services.md)).

## Testing notes

Cover: fuzzy match narrowing as characters are typed, `Enter` applying the filter and recomputing the
displayed list, `Esc` leaving `active_filter` unchanged, clearing an active filter via `Esc` in `Normal`
mode.

## Source map

- [`src/tui/state.rs`](../../src/tui/state.rs)
- [`src/tui/input.rs`](../../src/tui/input.rs)
- [`src/tui/mod.rs`](../../src/tui/mod.rs)
- [`src/services/entity_parser.rs`](../../src/services/entity_parser.rs)
- [`src/storage/entity_relations_repository.rs`](../../src/storage/entity_relations_repository.rs)

## Related docs

- [`../systems/tui.md`](../systems/tui.md), [`../systems/services.md`](../systems/services.md)
- [`../architecture/decisions/0006-tui-viewer.md`](../architecture/decisions/0006-tui-viewer.md)
- [`../architecture/decisions/0012-entity-relations.md`](../architecture/decisions/0012-entity-relations.md)
