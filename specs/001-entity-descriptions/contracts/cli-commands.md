# CLI Command Contracts: Entity Descriptions

**Feature**: 001-entity-descriptions
**Date**: 2026-02-01

## Overview

This document defines the command-line interface contracts for entity descriptions. These contracts specify command syntax, arguments, flags, output formats, and error behavior.

## Command: `wet entity edit`

### Purpose

Add or update a description for an existing entity using one of three input methods: inline text, interactive editor, or file input.

### Syntax

```bash
wet entity edit <ENTITY-NAME> [OPTIONS]
```

### Arguments

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<ENTITY-NAME>` | String | Yes | Name of the entity to edit (case-insensitive) |

### Options

| Flag | Type | Required | Mutually Exclusive | Description |
|------|------|----------|-------------------|-------------|
| `--description <TEXT>` | String | No | Yes (with editor, file) | Inline description text |
| `--description-file <PATH>` | Path | No | Yes (with editor, inline) | Path to file containing description |
| `--color <MODE>` | Enum | No | No | Color output mode: auto, always, never |

**Mutual Exclusivity**: Only one of `--description`, `--description-file`, or interactive editor (no flags) can be used per invocation.

### Behavior by Input Method

#### Method 1: Inline Text (`--description`)

```bash
wet entity edit rust --description "A systems programming language."
```

**Behavior**:
1. Validate entity exists (error if not found)
2. Trim whitespace from description text
3. If trimmed text is empty → remove description (set to NULL)
4. Else → extract entity references, auto-create entities, save description
5. Print success message

**Success Output**:
```
Description updated for entity 'rust'
```

**Empty/Whitespace Output**:
```
Description removed for entity 'rust'
```

#### Method 2: Interactive Editor (no flags)

```bash
wet entity edit rust
```

**Behavior**:
1. Validate entity exists (error if not found)
2. Determine editor: `$EDITOR` env var → vim → nano → vi
3. Create temp file with current description (if exists)
4. Launch editor with temp file
5. Wait for editor to exit
6. Read temp file contents
7. Delete temp file
8. Trim whitespace from description text
9. If trimmed text is empty → remove description
10. Else → extract entity references, auto-create entities, save description
11. Print success message

**Success Output**:
```
Description updated for entity 'rust'
```

**Editor Launch Output** (stderr, informational):
```
Launching editor: vim
```

**Editor Failure**:
```
Error: Failed to launch editor 'vim'
```

#### Method 3: File Input (`--description-file`)

```bash
wet entity edit rust --description-file desc.txt
```

**Behavior**:
1. Validate entity exists (error if not found)
2. Read file contents (error if file doesn't exist or can't be read)
3. Trim whitespace from file contents
4. If trimmed text is empty → remove description
5. Else → extract entity references, auto-create entities, save description
6. Print success message

**Success Output**:
```
Description updated for entity 'rust'
```

**File Not Found**:
```
Error: Description file 'desc.txt' not found
```

**File Read Error**:
```
Error: Failed to read description file 'desc.txt': Permission denied
```

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success (description updated or removed) |
| 1 | Entity not found |
| 2 | File not found or unreadable (--description-file) |
| 3 | Editor launch failed (interactive mode) |
| 4 | Mutual exclusivity violation (multiple input methods) |
| 5 | Database error |

### Error Messages

**Entity Not Found**:
```
Error: Entity 'foo' not found

Hint: Create the entity first by referencing it in a thought:
  wet add "Learning about @foo today"
```

**Mutual Exclusivity Violation**:
```
Error: Cannot use multiple input methods simultaneously

Usage:
  wet entity edit <name> --description "text"       # Inline
  wet entity edit <name> --description-file file    # From file
  wet entity edit <name>                            # Interactive editor
```

**Database Error**:
```
Error: Failed to update description: database is locked
```

### Entity Reference Auto-Creation

When description contains entity references:

```bash
wet entity edit rust --description "See @programming and [the guide](@rust-guide)"
```

**Behavior**:
1. Parse description for entity references
2. Extract unique entity names: `programming`, `rust-guide`
3. For each entity: `find_or_create()` in database
4. Auto-created entities have no description (NULL)
5. No output about auto-created entities (silent creation)

**Why Silent?**: Consistent with existing thought creation behavior

### Examples

**Example 1: Add Simple Description**
```bash
$ wet entity edit rust --description "A systems programming language."
Description updated for entity 'rust'
$ echo $?
0
```

**Example 2: Remove Description**
```bash
$ wet entity edit rust --description "   "
Description removed for entity 'rust'
$ echo $?
0
```

**Example 3: Interactive Editor**
```bash
$ export EDITOR=nano
$ wet entity edit rust
Launching editor: nano
[... user edits in nano ...]
Description updated for entity 'rust'
$ echo $?
0
```

**Example 4: File Input**
```bash
$ cat desc.txt
Rust is a systems programming language.
See @programming for context.

$ wet entity edit rust --description-file desc.txt
Description updated for entity 'rust'
$ echo $?
0
```

**Example 5: Entity Not Found**
```bash
$ wet entity edit nonexistent --description "test"
Error: Entity 'nonexistent' not found

Hint: Create the entity first by referencing it in a thought:
  wet add "Learning about @nonexistent today"
$ echo $?
1
```

---

## Command: `wet entities` (Modified)

### Purpose

List all entities in alphabetical order, with optional description previews.

### Syntax

```bash
wet entities [OPTIONS]
```

### Arguments

None (command takes no positional arguments)

### Options

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--color <MODE>` | Enum | No | Color output mode: auto, always, never |

**Note**: No new options added for this feature. Preview display is automatic based on terminal width.

### Behavior

**Algorithm**:
1. Fetch all entities from database (ordered by name, ascending)
2. Detect terminal width
3. For each entity:
   - If terminal width < 60: print entity name only
   - Else if entity has no description: print entity name only
   - Else: print entity name + " - " + preview
4. Generate preview:
   - Extract first paragraph (split on `\n\n`)
   - Strip entity reference markup (render as plain text)
   - Collapse newlines to spaces
   - Ellipsize to fit terminal width

### Output Format

#### Wide Terminal (≥ 60 characters)

```
<entity-name> - <preview-text>…
<entity-name-2>
<entity-name-3> - <preview-text-3>…
```

**Format Rules**:
- Entity name in original capitalization (canonical_name)
- " - " separator (space-dash-space) between name and preview
- Preview is first paragraph, entity references as plain text
- "…" (Unicode ellipsis U+2026) if truncated
- No "…" if preview fits without truncation
- Entities without descriptions show name only (no " - ")

**Example**:
```
rust - Rust is a systems programming language that focuses on safety…
systems-programming
wetware - A CLI tool for managing thoughts and entities. Built in Rust.
```

#### Narrow Terminal (< 60 characters)

```
<entity-name>
<entity-name-2>
<entity-name-3>
```

**Format Rules**:
- Entity name only (canonical_name)
- No previews displayed
- One entity per line

**Example**:
```
rust
systems-programming
wetware
```

#### Empty Database

```
No entities found.
```

### Preview Generation Rules

**Rule 1: First Paragraph Only**
- Split description on `\n\n` (double newline)
- Use first element only
- Ignore subsequent paragraphs

**Rule 2: Strip Entity Reference Markup**
- `@entity` → `entity`
- `[alias](@entity)` → `alias`
- No color highlighting
- Plain text only

**Rule 3: Collapse Newlines**
- Replace all single newlines (`\n`) with spaces
- Multiple spaces collapsed to single space
- Trim leading/trailing whitespace

**Rule 4: Ellipsize to Fit**
- Calculate available width: `terminal_width - entity_name.len() - 3`
  - 3 chars for " - "
- If preview.len() ≤ available_width: display as-is
- Else: truncate at last space before limit, append "…"
- If no space found: hard truncate at limit - 1, append "…"

**Rule 5: Minimum Preview Length**
- If available_width < 20: don't show preview (name only)
- Prevents unreadable truncated previews

### Exit Codes

| Code | Condition |
|------|-----------|
| 0 | Success (entities listed or "No entities found") |
| 5 | Database error |

### Error Messages

**Database Error**:
```
Error: Failed to retrieve entities: database is locked
```

### Examples

**Example 1: Wide Terminal with Descriptions**
```bash
$ wet entities
rust - Rust is a systems programming language that focuses on safety…
systems-programming - Low-level programming for operating systems and…
wetware - A CLI tool for managing thoughts and entities. Built in Rust.
```

**Example 2: Mixed (Some with Descriptions, Some Without)**
```bash
$ wet entities
knowledge-management
rust - Rust is a systems programming language that focuses on safety…
wetware
```

**Example 3: Narrow Terminal**
```bash
$ COLUMNS=50 wet entities
knowledge-management
rust
wetware
```

**Example 4: Empty Database**
```bash
$ wet entities
No entities found.
```

**Example 5: Entity References in Preview**

**Setup**:
```bash
$ wet entity edit rust --description "Used by @mozilla and [AWS](@amazon-web-services)."
```

**Output**:
```bash
$ wet entities
amazon-web-services
mozilla
rust - Used by mozilla and AWS.
```

**Note**: References rendered as plain text, no colors

---

## Global Options

All commands inherit global options from the main `wet` CLI:

### `--color <MODE>`

**Values**: `auto`, `always`, `never`

**Behavior**:
- `auto`: Use colors if stdout is a TTY
- `always`: Always use colors (even when piping)
- `never`: Never use colors

**Default**: `auto`

**Note**: Entity descriptions feature doesn't add colors to previews (plain text only), but this flag still affects entity names and other output.

---

## Contract Testing

### Test Coverage Requirements

Each contract must have:

1. **Success Cases**: Command succeeds with valid inputs
2. **Error Cases**: Command fails gracefully with invalid inputs
3. **Edge Cases**: Boundary conditions (empty, whitespace, long text)
4. **Output Validation**: Verify exact output format
5. **Exit Code Validation**: Verify correct exit codes

### Example Test Cases

**Contract Test: `wet entity edit` with inline description**

```rust
#[test]
fn test_entity_edit_inline_description() {
    let db = setup_temp_db();
    create_entity(&db, "rust");

    let output = run_wet_command(&[
        "entity", "edit", "rust",
        "--description", "A systems programming language."
    ]);

    assert_eq!(output.status, 0);
    assert_eq!(output.stdout, "Description updated for entity 'rust'\n");

    let entity = find_entity(&db, "rust");
    assert_eq!(entity.description, Some("A systems programming language.".to_string()));
}
```

**Contract Test: `wet entities` with preview**

```rust
#[test]
fn test_entities_shows_preview() {
    let db = setup_temp_db();
    create_entity_with_description(&db, "rust", "A systems programming language.");
    create_entity(&db, "wetware"); // No description

    let output = run_wet_command(&["entities"]);

    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("rust - A systems programming language."));
    assert!(output.stdout.contains("wetware\n")); // Name only, no preview
}
```

---

## Backward Compatibility

### Breaking Changes

None. All changes are additive:

- New command: `wet entity edit` (doesn't conflict with existing commands)
- Modified command: `wet entities` (output format extends, doesn't break)
- Existing entities without descriptions display as before (name only)

### Migration Path

1. Deploy updated CLI binary
2. Run database migration (adds `description` column)
3. Users can immediately use `wet entity edit`
4. `wet entities` command works with or without descriptions

**No user action required**: Existing workflows continue unchanged.
