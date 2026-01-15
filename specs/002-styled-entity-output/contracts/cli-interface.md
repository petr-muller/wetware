# CLI Interface Contract: Styled Entity Output

**Feature Branch**: `002-styled-entity-output`
**Date**: 2026-01-15

## Global Flag Addition

### `--color` Flag

**Scope**: Global flag available on all commands (affects output styling)

**Definition**:
```
--color <MODE>
    Control color output

    Possible values:
    - always: Always use colors regardless of terminal detection
    - auto:   Automatically detect terminal and use colors when appropriate [default]
    - never:  Never use colors
```

**Behavior**:
- Affects entity styling in `wet thoughts` output
- May affect future commands that render entities

## Command: `wet thoughts`

### Current Output Format (Before)

```
[<id>] <timestamp> - <raw_content_with_markup>
```

Example:
```
[1] 2026-01-15 10:30:45 - Meeting with [Sarah] about [project-alpha]
[2] 2026-01-15 11:15:22 - Email [Sarah] the report
```

### New Output Format (After)

**Plain mode** (non-TTY or `--color=never`):
```
[<id>] <timestamp> - <content_without_markup>
```

Example:
```
[1] 2026-01-15 10:30:45 - Meeting with Sarah about project-alpha
[2] 2026-01-15 11:15:22 - Email Sarah the report
```

**Styled mode** (TTY with `--color=auto` or `--color=always`):
```
[<id>] <timestamp> - <content_with_styled_entities>
```

Where entities are rendered with:
- Bold text
- Consistent color per unique entity
- No bracket markup

Visual representation (ANSI codes shown as descriptions):
```
[1] 2026-01-15 10:30:45 - Meeting with [BOLD+CYAN]Sarah[RESET] about [BOLD+GREEN]project-alpha[RESET]
[2] 2026-01-15 11:15:22 - Email [BOLD+CYAN]Sarah[RESET] the report
```

## Output Behavior Matrix

| Condition | `--color` value | Output |
|-----------|-----------------|--------|
| Interactive terminal | `auto` (default) | Styled entities |
| Interactive terminal | `always` | Styled entities |
| Interactive terminal | `never` | Plain text |
| Pipe (`\| command`) | `auto` (default) | Plain text |
| Pipe (`\| command`) | `always` | Styled entities |
| Pipe (`\| command`) | `never` | Plain text |
| Redirect (`> file`) | `auto` (default) | Plain text |
| Redirect (`> file`) | `always` | Styled entities (ANSI codes in file) |
| Redirect (`> file`) | `never` | Plain text |

## Entity Styling Rules

### Color Consistency

Within a single command execution:
- Same entity name → same color
- Case-insensitive matching (Sarah = sarah = SARAH)

### Color Assignment

- 12 distinct colors available
- Colors assigned in order of first entity appearance
- When entities exceed 12, colors cycle/repeat

### Markup Removal

| Input Content | Output Text |
|---------------|-------------|
| `[Sarah]` | `Sarah` |
| `[project-alpha]` | `project-alpha` |
| `[multi word entity]` | `multi word entity` |
| `plain text` | `plain text` |

## Exit Codes

No changes to exit code behavior. Feature is display-only.

## Environment Variables

The styling system respects standard environment variables:
- `NO_COLOR`: If set (any value), disables colors (equivalent to `--color=never`)
- `FORCE_COLOR`: If set (any value), forces colors (equivalent to `--color=always`)

Priority: CLI flag > environment variables > auto-detection

## Affected Subcommands

| Command | Entity Styling Applied |
|---------|----------------------|
| `wet thoughts` | Yes |
| `wet thoughts --on <entity>` | Yes |
| `wet thoughts --since <date>` | Yes |
| `wet thoughts --until <date>` | Yes |
| `wet entities` | No (not in scope) |
| `wet add` | No (input command) |

## Test Contracts

### TC-001: Plain Output When Piped
```bash
$ wet thoughts | cat
[1] 2026-01-15 10:30:45 - Meeting with Sarah about project-alpha
```
- No ANSI escape codes in output
- Entity brackets removed

### TC-002: Force Colors When Piped
```bash
$ wet thoughts --color=always | cat
# Output contains ANSI escape codes
```

### TC-003: No Colors in Terminal
```bash
$ wet thoughts --color=never
[1] 2026-01-15 10:30:45 - Meeting with Sarah about project-alpha
```
- No ANSI escape codes
- Entity brackets removed

### TC-004: Entity Color Consistency
```bash
$ wet thoughts
# Same entity "Sarah" appears with identical color in all occurrences
```

### TC-005: Markup Removal
```bash
$ wet thoughts --color=never
# All [entity] brackets removed from output
# Entity text preserved
```
