# 003: Styled Entity Output

## Summary

Render entity references in `wet thoughts` output with bold colored text instead of raw markup. Each unique entity gets a consistent color within a single execution. Styling auto-disables when output is piped or redirected, with explicit override via `--color` flag.

## Requirements

- Entity markup (`[entity]`) is stripped from output; only the entity name is displayed
- Entities rendered bold + colored in interactive terminals
- Same entity always gets the same color within one execution
- 12 distinct colors available; colors reuse when entity count exceeds 12
- TTY detection: styling auto-disabled when piping or redirecting
- `--color` flag: `auto` (default), `always`, `never`
- `NO_COLOR` env var disables colors; `FORCE_COLOR` forces them
- Priority: CLI flag > environment variables > auto-detection
- Applies to all `wet thoughts` variants (filtered and unfiltered)

## Decisions

- **owo-colors** crate for terminal styling: lightweight, no-alloc, supports bold + colors.
- **Color assignment**: colors assigned in order of first entity appearance, cycling when exhausted.
- **No persistence**: color assignments are ephemeral per execution, may differ between runs.
- **Markup always stripped**: even in plain mode, `[entity]` brackets are removed.

## CLI Interface

```
wet thoughts                   # Styled output (if TTY)
wet thoughts --color=always    # Force styled output (even when piped)
wet thoughts --color=never     # Force plain output (even in terminal)
wet thoughts | cat             # Auto-detects pipe, outputs plain text
```

Styled output (conceptual):
```
[1] 2026-01-15 10:30 - Meeting with Sarah about project-alpha
                              ^^^^^ bold+cyan  ^^^^^^^^^^^^^  bold+green
```

## Edge Cases

- Zero entities in output: normal display, no colored elements
- 25+ entities: colors cycle/repeat across entities
- Terminal doesn't support colors: falls back to plain output
