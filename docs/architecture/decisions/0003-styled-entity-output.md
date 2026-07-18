---
status: Accepted
date: "2026-04-10"
---

# Styled Entity Output

## Context

`wet thoughts` output showed raw `[entity]` bracket markup, which is noisy to read. Entity references
needed to stand out visually without hardcoding a color scheme that breaks in non-interactive contexts
(pipes, redirects, `NO_COLOR`-respecting environments).

## Decision

Strip entity markup from displayed output and, in interactive terminals, render each entity name bold and
colored — the same entity gets a consistent color within a single execution, cycling through a 12-color
palette when the entity count exceeds it. Use the `owo-colors` crate (lightweight, no-alloc, supports
bold+color). Color assignment happens in order of first appearance within that run and is never persisted
— it may differ between separate invocations. Detect TTY via `stdout().is_terminal()` for `Auto` mode
(the default); a `--color` flag (`auto`/`always`/`never`) and `NO_COLOR`/`FORCE_COLOR` env vars can
override it, with CLI flag taking priority over environment variables over auto-detection. Markup is
always stripped, even in plain (uncolored) mode.

## Consequences

- Output stays readable and unambiguous whether or not the terminal supports color, without the caller
  needing to think about it.
- Because color assignment isn't persisted, the same entity may render in a different color on separate
  runs — accepted as a simplicity tradeoff, since there's no natural "entity's color" to persist without
  adding state.
- This same styling approach was later reused, independently, by the TUI (`tui/ui.rs`) with a different,
  hash-based color-assignment algorithm rather than the CLI's sequential one — see
  [`../../systems/services.md`](../../systems/services.md#common-pitfalls) for the resulting
  CLI/TUI color-consistency caveat.

## Alternatives considered

- **Persisted per-entity color assignment** — not pursued: would require a new column/table and doesn't
  clearly improve the user experience enough to justify the added state.
- **Always-on color with no TTY detection** — rejected: would corrupt output when piped to another tool or
  redirected to a file.

## Related code

- [`src/services/entity_styler.rs`](../../../src/services/entity_styler.rs)
- [`src/services/color_mode.rs`](../../../src/services/color_mode.rs)

## Related docs

- [`../../systems/services.md`](../../systems/services.md)
- [`../../systems/tui.md`](../../systems/tui.md)
