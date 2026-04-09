# 005: Edit Existing Thoughts

## Summary

Add a `wet edit` subcommand to correct an existing thought's text (inline or via editor) and/or update its date, with entity associations atomically recalculated. Parallel in structure to `wet add`.

## Requirements

- `wet edit <ID> "new text"` replaces thought content via CLI argument
- `wet edit <ID> --editor` opens thought in interactive editor with current text pre-populated
- `wet edit <ID> --date YYYY-MM-DD` changes the thought's date without re-entering text
- Text and date can be edited in a single command: `wet edit <ID> "text" --date YYYY-MM-DD`
- At least one of text, `--date`, or `--editor` must be provided
- `CONTENT` and `--editor` are mutually exclusive
- When text changes, entity associations are fully recalculated: new entities added, removed entities unlinked
- New entities introduced by editing are auto-created (same as `wet add`)
- All edit operations are atomic: if any step fails, the thought remains unchanged (SQLite transaction)
- Empty text is rejected
- Non-existent thought ID returns clear error
- Editor closed without changes: no modification, user informed
- Editor crash/abnormal exit: no changes, warning displayed

## Decisions

- **No schema changes**: existing `thoughts.id`, `content`, `created_at` fields suffice.
- **Atomic via SQLite transaction**: update text, update date, unlink old entities, link new entities all in one transaction.
- **Reuses existing infrastructure**: editor launch, entity parser, entity find-or-create all reused from `wet add` and `wet entity edit`.
- **New error variant**: `ThoughtNotFound(i64)` added to `ThoughtError`.

## CLI Interface

```
wet edit <ID> "new text"                          # Edit text directly
wet edit <ID> --editor                            # Edit text in editor
wet edit <ID> --date 2026-01-15                   # Change date only
wet edit <ID> "new text" --date 2026-01-15        # Edit both at once
```

Success: `Thought <ID> updated.`
No changes: `No changes made to thought <ID>.`
Not found: `Error: Thought with ID <ID> not found.`

Exit codes: 0 (success/no-change), 1 (runtime error), 2 (usage error).

## Edge Cases

- Editing removes all entity references: all previous associations are removed
- Editing introduces references to new entities: those entities are auto-created
- Editor unavailable or not configured: error with clear message
- Invalid date format: error with expected format hint
