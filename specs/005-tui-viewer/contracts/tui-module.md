# TUI Module Contract

**Feature**: 005-tui-viewer
**Date**: 2026-03-09

## CLI Integration

### New subcommand: `wet tui`

Added to `Commands` enum in `src/cli/mod.rs`:

```rust
pub enum Commands {
    // ... existing variants
    Tui,
}
```

### CLI handler: `src/cli/tui.rs`

```rust
/// Launch the interactive TUI thought viewer.
pub fn execute(db_path: Option<&Path>) -> Result<(), ThoughtError>
```

Responsibilities:
- Open database connection
- Load thoughts and entities via repositories
- Initialize terminal (raw mode, alternate screen)
- Create `App` and run the event loop
- Restore terminal on exit (including on error/panic)

## TUI Module Public API: `src/tui/mod.rs`

```rust
/// Root application state for the TUI viewer.
pub struct App { /* ... */ }

impl App {
    /// Create a new App with loaded data.
    pub fn new(thoughts: Vec<Thought>, entities: Vec<Entity>) -> Self;

    /// Run the TUI event loop on the given terminal.
    pub fn run(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<(), ThoughtError>;
}
```

## State Module: `src/tui/state.rs`

```rust
/// Interaction mode of the TUI.
pub enum Mode {
    Normal,
    EntityPicker { /* fuzzy picker state */ },
    EntityDetail { /* popup state */ },
}

/// Sort order for the thought list.
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn toggle(&mut self);
    pub fn label(&self) -> &str;
}
```

## UI Module: `src/tui/ui.rs`

```rust
/// Render the full TUI frame.
pub fn render(app: &App, frame: &mut Frame);

/// Render the thought list in the main area.
fn render_thought_list(app: &App, frame: &mut Frame, area: Rect);

/// Render the fuzzy entity picker overlay.
fn render_entity_picker(app: &App, frame: &mut Frame, area: Rect);

/// Render the entity description modal popup.
fn render_entity_detail(app: &App, frame: &mut Frame, area: Rect);

/// Render the status bar (sort order, active filter, key hints).
fn render_status_bar(app: &App, frame: &mut Frame, area: Rect);
```

## Input Module: `src/tui/input.rs`

```rust
/// Handle a key event and update app state.
pub fn handle_key_event(app: &mut App, key: KeyEvent);

/// Handle key events in Normal mode.
fn handle_normal_mode(app: &mut App, key: KeyEvent);

/// Handle key events in EntityPicker mode.
fn handle_entity_picker_mode(app: &mut App, key: KeyEvent);

/// Handle key events in EntityDetail mode.
fn handle_entity_detail_mode(app: &mut App, key: KeyEvent);
```

## Error Handling

New variant added to `ThoughtError`:

```rust
#[error("TUI error: {0}")]
TuiError(String),
```

Used for terminal initialization/restoration failures and crossterm errors.

## Entity Color Mapping

Utility to bridge existing `owo_colors::AnsiColors` to `ratatui::style::Color`:

```rust
/// Map an owo-colors AnsiColor to a ratatui Color for TUI rendering.
pub fn ansi_to_ratatui_color(color: AnsiColors) -> ratatui::style::Color;
```

This may live in `src/services/entity_styler.rs` or in `src/tui/ui.rs` depending on whether it's useful outside the TUI module.
