# CLI Contract: `wet edit`

**Branch**: `004-edit-thoughts` | **Date**: 2026-02-19

## Command Signature

```
wet edit <ID> [CONTENT] [--date <DATE>] [--editor]
```

### Arguments

| Argument | Type | Required | Description |
|---|---|---|---|
| `ID` | positive integer | yes | Numeric ID of the thought to edit (visible in `wet` listing as `[id]`) |
| `CONTENT` | string | no | New text for the thought. Mutually exclusive with `--editor`. |
| `--date <DATE>` | string (YYYY-MM-DD) | no | New date for the thought. Can be combined with `CONTENT` or `--editor`. |
| `--editor` | flag | no | Open thought in interactive editor. Mutually exclusive with `CONTENT`. |

**Constraint**: At least one of `CONTENT`, `--date`, or `--editor` must be provided. Providing none is a usage error.

**Constraint**: `CONTENT` and `--editor` are mutually exclusive.

---

## Scenarios & Expected Behaviour

### Edit text via direct argument

```
$ wet edit 3 "Met with [Alice] and [Bob] about the roadmap"
Thought 3 updated.
```

- Thought 3 content is replaced.
- Entity associations are recalculated: `Alice` and `Bob` are linked; any previous entities not in the new text are unlinked.

---

### Edit date only

```
$ wet edit 3 --date 2026-01-15
Thought 3 updated.
```

- Thought 3 `created_at` is set to 2026-01-15 00:00:00 UTC.
- Text and entity associations are unchanged.

---

### Edit text and date in a single command

```
$ wet edit 3 "Retrospective notes for [Project-Y]" --date 2026-01-31
Thought 3 updated.
```

- Both content and date are updated atomically.
- Entity associations recalculated from new content.

---

### Edit text via interactive editor

```
$ wet edit 3 --editor
[editor opens with current thought content pre-populated]
[user saves and closes]
Thought 3 updated.
```

- Editor opens with existing content. On save, content is replaced and entity associations recalculated.

---

### No changes detected (editor closed without saving)

```
$ wet edit 3 --editor
[user closes editor without changes]
No changes made to thought 3.
```

- Thought is not modified. Exit code 0.

---

### Thought not found

```
$ wet edit 999 "some text"
Error: Thought with ID 999 not found.
```

- Exit code: non-zero (1).
- No changes made.

---

### Empty content rejected

```
$ wet edit 3 ""
Error: Thought content cannot be empty.
```

- Exit code: non-zero (1).
- No changes made.

---

### Invalid date format

```
$ wet edit 3 --date "not-a-date"
Error: Invalid date format 'not-a-date'. Expected YYYY-MM-DD.
```

- Exit code: non-zero (1).
- No changes made.

---

### No arguments (usage error)

```
$ wet edit 3
Error: At least one of CONTENT, --date, or --editor must be provided.
```

- Exit code: non-zero (2 for usage errors).

---

### Editor crash / abnormal exit

```
$ wet edit 3 --editor
[editor crashes]
Warning: Editor exited abnormally. No changes made to thought 3.
```

- Thought is not modified. Exit code 0 (treated as no-change, not error).

---

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success (thought updated, or no changes detected) |
| 1 | Runtime error (thought not found, validation failed, storage error) |
| 2 | Usage error (missing required arguments, mutually exclusive flags) |

---

## Integration with Existing Commands

The `wet edit` subcommand is parallel in structure to `wet add`:

| `wet add` | `wet edit` |
|---|---|
| `wet add "content"` | `wet edit <id> "content"` |
| `wet add --editor` | `wet edit <id> --editor` |
| (no date flag) | `wet edit <id> --date YYYY-MM-DD` |

The listing command (`wet thoughts`) is unchanged. IDs are already shown as `[id]` in its output.
