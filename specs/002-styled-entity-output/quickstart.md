# Quickstart: Styled Entity Output Implementation

**Feature Branch**: `002-styled-entity-output`
**Date**: 2026-01-15

## Prerequisites

- Rust 2024 edition toolchain installed
- Repository cloned and on branch `002-styled-entity-output`
- Understanding of existing codebase structure (see exploration in plan.md)

## Step 1: Add Dependency

Add to `Cargo.toml`:

```toml
[dependencies]
owo-colors = { version = "4", features = ["supports-colors"] }
```

Run `cargo build` to verify dependency resolution.

## Step 2: Create ColorMode Enum

Create `src/services/color_mode.rs`:

```rust
use clap::ValueEnum;

/// Controls color output behavior for styled entity rendering
#[derive(Clone, Copy, Debug, Default, ValueEnum, PartialEq, Eq)]
pub enum ColorMode {
    /// Always use colors regardless of terminal detection
    Always,
    /// Automatically detect terminal and use colors when appropriate
    #[default]
    Auto,
    /// Never use colors
    Never,
}

impl ColorMode {
    /// Determine if colors should be used based on mode and terminal state
    pub fn should_use_colors(&self) -> bool {
        use std::io::IsTerminal;
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }
}
```

## Step 3: Create Entity Styler Service

Create `src/services/entity_styler.rs`:

```rust
use owo_colors::{OwoColorize, AnsiColors};
use std::collections::HashMap;

const ENTITY_COLORS: [AnsiColors; 12] = [
    AnsiColors::Cyan,
    AnsiColors::Green,
    AnsiColors::Yellow,
    AnsiColors::Blue,
    AnsiColors::Magenta,
    AnsiColors::Red,
    AnsiColors::BrightCyan,
    AnsiColors::BrightGreen,
    AnsiColors::BrightYellow,
    AnsiColors::BrightBlue,
    AnsiColors::BrightMagenta,
    AnsiColors::BrightRed,
];

/// Manages consistent color assignment for entities within a single execution
pub struct EntityStyler {
    color_map: HashMap<String, AnsiColors>,
    next_color: usize,
    use_colors: bool,
}

impl EntityStyler {
    pub fn new(use_colors: bool) -> Self {
        Self {
            color_map: HashMap::new(),
            next_color: 0,
            use_colors,
        }
    }

    /// Get or assign a color for the given entity
    fn get_color(&mut self, entity: &str) -> AnsiColors {
        let key = entity.to_lowercase();
        if let Some(&color) = self.color_map.get(&key) {
            return color;
        }
        let color = ENTITY_COLORS[self.next_color % ENTITY_COLORS.len()];
        self.color_map.insert(key, color);
        self.next_color += 1;
        color
    }

    /// Render thought content with styled entities
    pub fn render_content(&mut self, content: &str) -> String {
        // Implementation using entity_parser to find and replace entities
        // ...
    }
}
```

## Step 4: Update CLI Structure

Modify `src/cli/mod.rs` to add global `--color` flag:

```rust
use crate::services::color_mode::ColorMode;

#[derive(Parser)]
#[command(name = "wet")]
pub struct Cli {
    /// Control color output
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, global = true)]
    pub color: ColorMode,

    #[command(subcommand)]
    pub command: Commands,
}
```

## Step 5: Update Thoughts Command

Modify `src/cli/thoughts.rs` to use styled rendering:

```rust
use crate::services::entity_styler::EntityStyler;

pub fn execute(args: &Thoughts, color_mode: ColorMode) -> Result<(), ThoughtError> {
    let use_colors = color_mode.should_use_colors();
    let mut styler = EntityStyler::new(use_colors);

    for thought in thoughts {
        let styled_content = styler.render_content(&thought.content);
        println!(
            "[{}] {} - {}",
            thought.id.unwrap_or(0),
            thought.created_at.format("%Y-%m-%d %H:%M:%S"),
            styled_content
        );
    }
    Ok(())
}
```

## Step 6: Update Services Module

Add exports in `src/services/mod.rs`:

```rust
pub mod color_mode;
pub mod entity_styler;
```

## Step 7: Write Tests

### Unit Tests (`src/services/entity_styler.rs`)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_entity_same_color() {
        let mut styler = EntityStyler::new(true);
        let color1 = styler.get_color("Sarah");
        let color2 = styler.get_color("sarah");  // case-insensitive
        assert_eq!(color1, color2);
    }

    #[test]
    fn test_different_entities_different_colors() {
        let mut styler = EntityStyler::new(true);
        let color1 = styler.get_color("Sarah");
        let color2 = styler.get_color("John");
        assert_ne!(color1, color2);
    }

    #[test]
    fn test_colors_cycle_after_palette_exhausted() {
        let mut styler = EntityStyler::new(true);
        let colors: Vec<_> = (0..15)
            .map(|i| styler.get_color(&format!("entity{}", i)))
            .collect();
        // 13th entity (index 12) should wrap to first color
        assert_eq!(colors[0], colors[12]);
    }
}
```

### Contract Tests (`tests/contract/test_thoughts_command.rs`)

```rust
#[test]
fn test_thoughts_output_strips_entity_markup() {
    let temp_db = setup_temp_db();
    run_wet_command(&["add", "Meeting with [Sarah]"], Some(&temp_db));

    let result = run_wet_command(&["thoughts", "--color=never"], Some(&temp_db));

    assert!(result.stdout.contains("Meeting with Sarah"));
    assert!(!result.stdout.contains("[Sarah]"));
}

#[test]
fn test_thoughts_piped_has_no_ansi_codes() {
    // Test using shell pipe
    // Verify no escape codes in output
}
```

## Build & Test Commands

```bash
# Build
cargo build

# Run all tests
cargo nextest run

# Run specific test
cargo nextest run test_same_entity_same_color

# Check formatting
cargo fmt --check

# Run linter
cargo clippy

# Manual testing
cargo run -- thoughts
cargo run -- thoughts --color=never
cargo run -- thoughts --color=always | cat
```

## Key Files to Modify/Create

| File                                      | Action | Purpose                   |
|-------------------------------------------|--------|---------------------------|
| `Cargo.toml`                              | Modify | Add owo-colors dependency |
| `src/services/mod.rs`                     | Modify | Export new modules        |
| `src/services/color_mode.rs`              | Create | ColorMode enum definition |
| `src/services/entity_styler.rs`           | Create | Entity styling service    |
| `src/cli/mod.rs`                          | Modify | Add global --color flag   |
| `src/cli/thoughts.rs`                     | Modify | Use styled rendering      |
| `tests/contract/test_thoughts_command.rs` | Modify | Add styled output tests   |

## Verification Checklist

- [ ] `cargo build` succeeds
- [ ] `cargo nextest run` passes all tests
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt --check` passes
- [ ] `wet thoughts` shows styled entities in terminal
- [ ] `wet thoughts | cat` shows plain text without escape codes
- [ ] `wet thoughts --color=never` shows plain text
- [ ] `wet thoughts --color=always | cat` shows escape codes
- [ ] Same entity has consistent color throughout output
