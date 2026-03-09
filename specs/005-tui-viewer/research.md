# Research: Interactive TUI Thought Viewer

**Feature**: 005-tui-viewer
**Date**: 2026-03-09

## TUI Framework Selection

**Decision**: ratatui with crossterm backend

**Rationale**: ratatui is the dominant Rust TUI framework (successor to tui-rs), used by major projects (helix, gitui, bottom). It provides immediate-mode rendering with full control over the event loop, which suits a simple read-only viewer. Built-in support for styled text spans (`Line`/`Span`), scrollable lists (`List`/`ListState`), and popup overlays (via `Clear` + layered rendering).

**Alternatives considered**:
- **cursive**: Retained-mode framework. More opinionated, smaller ecosystem. Less flexible for custom rendering needs like inline entity highlighting.
- **termwiz**: Too low-level for this use case, would require reimplementing standard widgets.

## Application Architecture Pattern

**Decision**: The Elm Architecture (TEA) — Model/View/Update

**Rationale**: TEA is the simplest pattern for a read-only viewer with clear state transitions. The app has a small, well-defined state (thought list, sort order, filter, active overlay) and a limited set of user actions. No need for the complexity of component architecture or async patterns.

**Pattern**:
- `App` struct holds all state (Model)
- `render()` function draws UI from state (View)
- `handle_key()` maps key events to state mutations (Update)
- Synchronous event loop: draw → read event → update state → repeat

## Text Input Widget

**Decision**: tui-input

**Rationale**: Lightweight single-line text input widget, sufficient for the fuzzy filter input in the entity picker. `tui-textarea` is heavier and designed for multi-line editing, which is unnecessary here.

## Fuzzy Matching

**Decision**: nucleo-matcher

**Rationale**: High-performance fuzzy matching library from the helix-editor project. ~6x faster than skim's fuzzy-matcher, with full Unicode support. Only the matcher crate is needed (not the full `nucleo` async picker), since we just need to score and rank entity names against user input.

**Alternative**: `fuzzy-matcher` (skim) — simpler API but ASCII-only case insensitivity and slower performance.

## Entity Highlighting in TUI

**Decision**: Adapt existing `EntityStyler` color assignments to ratatui `Style` objects

**Rationale**: The existing `EntityStyler` in `src/services/entity_styler.rs` assigns consistent colors to entities using a 12-color palette of `owo_colors::AnsiColors`. For the TUI, we need to map these same color assignments to ratatui `Style` objects with matching foreground colors. This ensures visual consistency between CLI output and TUI display.

**Approach**: Create a mapping function from `owo_colors::AnsiColors` to `ratatui::style::Color`. The entity parser (`ENTITY_PATTERN` regex) is reused directly for identifying entity references in thought content.

## Modal Popup Implementation

**Decision**: Layered rendering with `Clear` widget

**Rationale**: ratatui supports popups by rendering the main UI first, then rendering a `Clear` widget followed by the popup content in a centered `Rect`. This is the standard pattern documented in ratatui's popup example. No additional crate needed.

## New Dependencies Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| ratatui | latest (0.29+) | TUI framework with crossterm backend |
| crossterm | latest (0.29+) | Terminal events and raw mode |
| tui-input | latest (0.11+) | Single-line text input widget |
| nucleo-matcher | latest (0.3+) | Fuzzy string matching |

Note: Check crates.io for exact latest versions at implementation time.
