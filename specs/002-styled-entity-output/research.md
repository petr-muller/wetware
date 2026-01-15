# Research: Styled Entity Output

**Feature Branch**: `002-styled-entity-output`
**Date**: 2026-01-15
**Status**: Complete

## Research Questions

1. Which Rust library should be used for terminal styling (colors, bold)?
2. How should TTY detection be implemented?
3. How should the `--color` CLI flag be implemented?

---

## Decision 1: Terminal Styling Library

### Decision: **owo-colors**

### Rationale

- **Zero allocation, zero-cost**: No runtime overhead for styled output
- **No-std compatible**: Lightweight and focused
- **Drop-in colored replacement**: Simple, intuitive API
- **Built-in supports-colors feature**: Integrates TTY detection and environment variable handling (`NO_COLOR`, `FORCE_COLOR`)
- **Recommended by Rust CLI best practices**: Per [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html)
- **Active maintenance**: MSRV of Rust 1.70, well-maintained

### Alternatives Considered

| Library     | Pros                    | Cons                                         | Why Rejected                                   |
|-------------|-------------------------|----------------------------------------------|------------------------------------------------|
| `colored`   | Simple API, widely used | Allocations per styled string                | Less efficient, owo-colors is zero-cost        |
| `crossterm` | Full terminal control   | Much heavier, overkill for just colors       | Only need styling, not full terminal control   |
| `termcolor` | Mature                  | Complex API, targets deprecated Windows APIs | API complexity, Windows compatibility concerns |
| `anstyle`   | Interoperability        | Lower-level, needs adapters                  | Adds complexity for simple use case            |

### Usage Pattern

```rust
use owo_colors::OwoColorize;

// Simple styling
let styled = "entity-name".bold().cyan();

// With dynamic colors
use owo_colors::colors::*;
let styled = "entity-name".bold().color(Cyan);
```

### Dependency Addition

```toml
[dependencies]
owo-colors = { version = "4", features = ["supports-colors"] }
```

---

## Decision 2: TTY Detection

### Decision: **std::io::IsTerminal** (standard library)

### Rationale

- **Standard library**: No additional dependency needed
- **Stable since Rust 1.70**: Well-tested and maintained
- **Simple API**: Single trait method `is_terminal()`
- **Cross-platform**: Works on Unix and Windows
- **Recommended approach**: Both `atty` and `is-terminal` crates recommend using std in their documentation

### Alternatives Considered

| Approach                   | Pros       | Cons                                             | Why Rejected                                     |
|----------------------------|------------|--------------------------------------------------|--------------------------------------------------|
| `atty` crate               | Simple API | Unmaintained, recommends std::io::IsTerminal     | Deprecated                                       |
| `is-terminal` crate        | Reliable   | Unnecessary dependency when std has same feature | Extra dependency                                 |
| owo-colors supports-colors | Integrated | May want direct control in some cases            | Use std for explicit TTY check, owo for env vars |

### Usage Pattern

```rust
use std::io::IsTerminal;

fn should_use_colors() -> bool {
    std::io::stdout().is_terminal()
}
```

### Integration with owo-colors

The `supports-colors` feature of owo-colors provides `Stream::Stdout.supports_color()` which checks:
1. TTY detection
2. `NO_COLOR` environment variable
3. `FORCE_COLOR` environment variable
4. CI environment detection
5. Terminal type detection

This can be used alongside explicit std::io::IsTerminal for fine-grained control.

---

## Decision 3: CLI Color Flag Implementation

### Decision: **Custom implementation with clap ValueEnum**

### Rationale

- **Simple, self-contained**: No additional dependency
- **Full control**: Can match exact behavior needed for entity styling
- **Constitution compliance**: Simplicity & YAGNI - avoid unnecessary dependencies
- **clap integration**: Uses clap's built-in ValueEnum derive for clean argument parsing

### Alternatives Considered

| Approach           | Pros                  | Cons                                             | Why Rejected                      |
|--------------------|-----------------------|--------------------------------------------------|-----------------------------------|
| `colorchoice-clap` | Ready-made solution   | Extra dependency, focused on clap's own coloring | Overkill for our use case         |
| `clap-color-flag`  | Pre-built integration | Another dependency                               | YAGNI - simple enum is sufficient |
| `concolor-clap`    | Full-featured         | Heavy, more than needed                          | Over-engineered for simple flag   |

### Implementation Design

```rust
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ColorMode {
    /// Always use colors regardless of terminal detection
    Always,
    /// Automatically detect terminal and use colors when appropriate
    #[default]
    Auto,
    /// Never use colors
    Never,
}

#[derive(Parser)]
struct Cli {
    /// Control color output
    #[arg(long, value_enum, default_value_t = ColorMode::Auto)]
    color: ColorMode,
}
```

### Behavior Matrix

| ColorMode | TTY Detected | Result |
|-----------|--------------|--------|
| Always    | Yes          | Styled |
| Always    | No           | Styled |
| Auto      | Yes          | Styled |
| Auto      | No           | Plain  |
| Never     | Yes          | Plain  |
| Never     | No           | Plain  |

---

## Color Palette Selection

### Decision: 12 ANSI Colors

Use standard ANSI 16-color palette subset for maximum terminal compatibility:

```rust
const ENTITY_COLORS: [Color; 12] = [
    Color::Cyan,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Red,
    Color::BrightCyan,
    Color::BrightGreen,
    Color::BrightYellow,
    Color::BrightBlue,
    Color::BrightMagenta,
    Color::BrightRed,
];
```

### Rationale

- **12 colors**: Meets FR-005 requirement of 10-15 distinct colors
- **No white/black**: Avoids contrast issues with terminal backgrounds
- **ANSI 16-color compatible**: Works on virtually all terminals
- **Visually distinct**: Each color easily distinguishable

### Color Assignment Strategy

- Hash entity name to index: `entity_name.bytes().fold(0, |acc, b| acc.wrapping_add(b as usize)) % ENTITY_COLORS.len()`
- Store mapping in `HashMap<String, Color>` for consistency within execution
- Deterministic within execution, but may vary between executions (acceptable per spec)

---

## Summary of Dependencies

### New Dependencies to Add

```toml
[dependencies]
owo-colors = { version = "4", features = ["supports-colors"] }
```

### Standard Library Features Used

- `std::io::IsTerminal` - TTY detection
- `std::collections::HashMap` - Color assignment tracking

### No Additional Dependencies Needed For

- TTY detection (std)
- CLI flag parsing (already have clap)

---

## Sources

- [Rain's Rust CLI recommendations - Managing colors in Rust](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html)
- [std::io::IsTerminal documentation](https://doc.rust-lang.org/beta/std/io/trait.IsTerminal.html)
- [Detect TTY in Rust](https://alexwlchan.net/til/2024/detect-tty-in-rust/)
- [owo-colors crate](https://lib.rs/crates/owo-colors)
- [colorchoice-clap crate](https://docs.rs/colorchoice-clap/latest/colorchoice_clap/)
