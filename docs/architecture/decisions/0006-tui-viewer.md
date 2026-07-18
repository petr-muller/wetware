---
status: Accepted
date: "2026-04-10"
---

# TUI Thought Viewer

## Context

Browsing thoughts through repeated `wet thoughts --on <entity>` CLI invocations was slow for exploratory
use. An interactive, scrollable viewer with filtering and detail views was wanted, without adding write
complexity to a first version.

## Decision

Build a read-only interactive TUI (`wet tui`) using `ratatui` + `crossterm` (cross-platform, well
maintained), structured around The Elm Architecture (state → event → update → render). Fuzzy entity
filtering uses `tui-input` for the text widget and `nucleo-matcher` for scoring. All thoughts and entities
are loaded upfront at startup — no database modifications from the TUI in this initial version, and a
single-threaded synchronous event loop, both judged sufficient for personal-scale data volumes. Features:
scrollable thought list with entity highlighting (reusing entity styling), `/` opens a fuzzy entity picker
that filters the list, `s` toggles sort order (composable with an active filter), `Enter`/`d` opens a
modal showing full entity descriptions for the selected thought's references, `q`/`Esc` quits or dismisses
overlays.

Note: a later feature ([`0008-delete-thoughts.md`](0008-delete-thoughts.md)) added a delete capability to
the TUI, superseding the "read-only" aspect of this decision for that one operation — see that ADR.

## Consequences

- The Elm-style split (state/input/ui) kept rendering, input handling, and state mutation independently
  testable.
- Loading all data upfront means the TUI never re-queries mid-session — simple, but means a concurrent
  external change to the database (e.g. a `wet delete` in another terminal) isn't reflected until restart.
- Reusing existing entity-styling infrastructure conceptually, rather than by direct code reuse, led to
  the TUI implementing its own separate color-assignment algorithm — see
  [`../../systems/services.md`](../../systems/services.md#common-pitfalls).

## Alternatives considered

- **A different TUI framework** — `ratatui`/`crossterm` chosen for being well-maintained and
  cross-platform; no other framework is recorded as seriously evaluated.
- **Live re-querying instead of upfront load** — rejected for the initial version as unnecessary
  complexity given personal-scale data volumes and single-user, single-process usage.

## Related code

- [`src/tui/mod.rs`](../../../src/tui/mod.rs)
- [`src/tui/state.rs`](../../../src/tui/state.rs)
- [`src/tui/input.rs`](../../../src/tui/input.rs)
- [`src/tui/ui.rs`](../../../src/tui/ui.rs)

## Related docs

- [`../../systems/tui.md`](../../systems/tui.md)
- [`../../flows/tui-entity-filter.md`](../../flows/tui-entity-filter.md)
- [`0008-delete-thoughts.md`](0008-delete-thoughts.md)
