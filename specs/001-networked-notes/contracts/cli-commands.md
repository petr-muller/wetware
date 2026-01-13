# CLI Command Contracts: Networked Notes

**Feature**: 001-networked-notes
**Date**: 2025-12-30
**Status**: Complete

## Overview

This document defines the command-line interface contracts for the networked notes feature. All commands follow Unix conventions and maintain consistency with the existing `wet` CLI.

## General Conventions

### Exit Codes

| Code | Meaning      | When Used                         |
|------|--------------|-----------------------------------|
| 0    | Success      | Operation completed successfully  |
| 1    | User Error   | Invalid input, validation failure |
| 2    | System Error | Database unavailable, disk full   |

### Output Format

- **Standard Output** (stdout): Command results, list outputs
- **Standard Error** (stderr): Error messages, warnings
- **Encoding**: UTF-8
- **Line Endings**: Platform-specific (LF on Unix, CRLF on Windows)

### Common Behaviors

- Empty results show helpful messages (not silent failure)
- Errors include actionable guidance where possible
- All timestamps displayed in local timezone
- All commands respect `--help` flag

---

## Command: `wet add`

### Synopsis

```bash
wet add <text>
```

### Description

Add a new note with optional entity references. Text can contain zero or more entity references using `[entity-name]` syntax.

### Arguments

| Argument | Type   | Required | Description                        |
|----------|--------|----------|------------------------------------|
| `<text>` | String | Yes      | Note content (1-10,000 characters) |

### Entity Syntax

- Format: `[entity-name]`
- Case-insensitive matching (first occurrence preserved)
- Supports spaces: `[multi word entity]`
- Supports hyphens/underscores: `[project-alpha]`, `[task_123]`
- Malformed syntax (e.g., `[unclosed`) is ignored

### Examples

#### Basic note without entities

```bash
$ wet add 'Remember to buy groceries'
✓ Note added
```

#### Note with single entity

```bash
$ wet add 'Meeting with [Sarah] at 3pm'
✓ Note added (1 entity: Sarah)
```

#### Note with multiple entities

```bash
$ wet add 'Discussion about [project-alpha] with [Sarah] and [John]'
✓ Note added (3 entities: project-alpha, Sarah, John)
```

#### Note with duplicate entity references

```bash
$ wet add '[Sarah] and [sarah] are the same person'
✓ Note added (1 entity: Sarah)
```

### Success Output

**Format**: `✓ Note added [(<count> entities: <list>)]`

**Examples**:
- No entities: `✓ Note added`
- One entity: `✓ Note added (1 entity: Sarah)`
- Multiple: `✓ Note added (3 entities: project-alpha, Sarah, John)`

**Exit Code**: 0

### Error Cases

#### Empty content

```bash
$ wet add ''
Error: Note content cannot be empty
```
**Exit Code**: 1

#### Content too long

```bash
$ wet add '<10,001+ characters>'
Error: Note exceeds maximum length of 10000 characters (got 10045)
```
**Exit Code**: 1

#### Database unavailable

```bash
$ wet add 'Some note'
Error: Failed to save note: database unavailable at /path/to/wetware.db
Try: Check database file permissions
```
**Exit Code**: 2

### Contract Tests

```rust
#[test]
fn test_add_note_success() {
    let output = run_command(&["wet", "add", "Test note"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("✓ Note added"));
}

#[test]
fn test_add_note_with_entities() {
    let output = run_command(&["wet", "add", "Meeting with [Sarah]"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("1 entity: Sarah"));
}

#[test]
fn test_add_note_empty_content() {
    let output = run_command(&["wet", "add", ""]);
    assert_eq!(output.status, 1);
    assert!(output.stderr.contains("cannot be empty"));
}
```

---

## Command: `wet notes`

### Synopsis

```bash
wet notes [--on <entity>]
```

### Description

List all notes in chronological order (oldest first), or filter by entity reference.

### Arguments

| Argument        | Type   | Required | Description                                            |
|-----------------|--------|----------|--------------------------------------------------------|
| `--on <entity>` | String | No       | Filter notes containing this entity (case-insensitive) |

### Examples

#### List all notes

```bash
$ wet notes
[2025-12-30 14:23] Remember to buy groceries
[2025-12-30 14:25] Meeting with [Sarah] at 3pm
[2025-12-30 14:30] Discussion about [project-alpha] with [Sarah] and [John]
```

#### Filter by entity

```bash
$ wet notes --on Sarah
[2025-12-30 14:25] Meeting with [Sarah] at 3pm
[2025-12-30 14:30] Discussion about [project-alpha] with [Sarah] and [John]
```

#### Filter by entity (case-insensitive)

```bash
$ wet notes --on sarah
[2025-12-30 14:25] Meeting with [Sarah] at 3pm
[2025-12-30 14:30] Discussion about [project-alpha] with [Sarah] and [John]
```

### Success Output

**Format**: `[<timestamp>] <content>`

- Timestamp format: `YYYY-MM-DD HH:MM` (local timezone)
- One note per line
- Notes sorted chronologically (oldest first)
- Entity references preserved in original text

**Exit Code**: 0

### Empty Results

#### No notes exist

```bash
$ wet notes
No notes found. Add your first note with: wet add 'your note'
```

#### No notes match entity filter

```bash
$ wet notes --on nonexistent
No notes found referencing 'nonexistent'
Try: wet entities (to see all entities)
```

**Exit Code**: 0 (empty result is not an error)

### Error Cases

#### Database unavailable

```bash
$ wet notes
Error: Failed to retrieve notes: database unavailable
```
**Exit Code**: 2

### Contract Tests

```rust
#[test]
fn test_list_all_notes() {
    setup_with_notes(&["Note 1", "Note 2"]);
    let output = run_command(&["wet", "notes"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("Note 1"));
    assert!(output.stdout.contains("Note 2"));
}

#[test]
fn test_filter_notes_by_entity() {
    setup_with_notes(&["Meeting with [Sarah]", "Call [John]"]);
    let output = run_command(&["wet", "notes", "--on", "Sarah"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("[Sarah]"));
    assert!(!output.stdout.contains("[John]"));
}

#[test]
fn test_empty_notes_list() {
    let output = run_command(&["wet", "notes"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("No notes found"));
}
```

---

## Command: `wet entities`

### Synopsis

```bash
wet entities
```

### Description

List all unique entities referenced across all notes, displayed with their canonical capitalization (first occurrence).

### Arguments

None

### Examples

#### List entities

```bash
$ wet entities
John
project-alpha
Sarah
```

### Success Output

**Format**: One entity per line, sorted alphabetically (case-insensitive)

- Uses canonical capitalization (first occurrence)
- Alphabetically sorted
- No timestamps or counts

**Exit Code**: 0

### Empty Results

```bash
$ wet entities
No entities found. Add notes with entity references like: wet add 'Meeting with [Sarah]'
```

**Exit Code**: 0

### Error Cases

#### Database unavailable

```bash
$ wet entities
Error: Failed to retrieve entities: database unavailable
```
**Exit Code**: 2

### Contract Tests

```rust
#[test]
fn test_list_entities() {
    setup_with_notes(&["[Sarah] and [John]", "[project-alpha]"]);
    let output = run_command(&["wet", "entities"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("Sarah"));
    assert!(output.stdout.contains("John"));
    assert!(output.stdout.contains("project-alpha"));
}

#[test]
fn test_entities_canonical_capitalization() {
    setup_with_notes(&["[Sarah]", "[sarah]", "[SARAH]"]);
    let output = run_command(&["wet", "entities"]);
    let lines: Vec<&str> = output.stdout.lines().collect();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "Sarah");  // First occurrence preserved
}

#[test]
fn test_empty_entities_list() {
    setup_with_notes(&["Note without entities"]);
    let output = run_command(&["wet", "entities"]);
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("No entities found"));
}
```

---

## Global Options

All commands support these global options:

### `--help`

```bash
$ wet add --help
wet-add
Add a new note

USAGE:
    wet add <text>

ARGS:
    <text>    Note content (1-10,000 characters)
```

### `--version`

```bash
$ wet --version
wet 0.2.0
```

---

## Error Message Format

All error messages follow this structure:

```
Error: <clear description of what went wrong>
[Try: <actionable suggestion>]
```

### Examples

**Good Error Messages**:
```
Error: Note content cannot be empty
```

```
Error: Failed to save note: database unavailable at /path/to/wetware.db
Try: Check database file permissions
```

**Bad Error Messages** (avoid these):
```
Error: invalid input
```

```
Error: DB error
```

---

## Performance Contracts

| Command          | Max Latency | Notes                            |
|------------------|-------------|----------------------------------|
| `wet add`        | 100ms       | P95 latency for typical note     |
| `wet notes`      | 100ms       | P95 latency for <1000 notes      |
| `wet notes --on` | 500ms       | P95 latency for entity filtering |
| `wet entities`   | 100ms       | P95 latency for <500 entities    |

---

## Backwards Compatibility

- These commands are new and don't affect existing `wet` functionality
- If `wet add` previously existed with different behavior, this spec defines the new behavior
- Future versions may add options (e.g., `--format json`) without breaking existing usage

---

## Summary

This contract specification defines:

✅ All command syntaxes and arguments
✅ Success and error output formats
✅ Exit codes for all scenarios
✅ Empty result handling
✅ Performance requirements
✅ Contract test examples

All CLI commands are fully specified and ready for implementation.
