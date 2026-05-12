# 008: Delete Thoughts

## Summary

Allow deleting thoughts by ID, both from the CLI and the TUI.

## Requirements

- CLI: `wet delete <ID>` deletes a thought immediately (no confirmation)
- TUI: `x` key opens a confirmation overlay; `y` confirms deletion, `n`/`Esc` cancels
- Deleting a thought removes its `thought_entities` junction rows (handled by ON DELETE CASCADE)
- Deleting a nonexistent ID returns an error
- After TUI deletion, selection adjusts to stay valid (same index or previous if at end)

## Decisions

- **No CLI confirmation:** Deleting a single thought by explicit ID is low-risk; no interactive prompt needed
- **TUI confirmation overlay:** Destructive action in a browsing UI warrants a y/n confirmation step
- **TUI keybinding:** `x` (vim-style remove); `d` is already taken by entity detail
- **Database access in TUI:** Store `db_path: PathBuf` in `App`, open connection on demand for mutations

## CLI Interface

```
wet delete <ID>
```

Output on success: `Deleted thought <ID>.`
Output on not found: `Error: Thought with id <ID> not found`

## TUI Interface

- Normal mode: press `x` with a thought selected → enters `ConfirmDelete` mode
- `ConfirmDelete` mode shows a centered overlay with thought content and date
- `y`/`Y` confirms deletion, `n`/`N`/`Esc` cancels back to Normal mode
- Status bar hints include `x:Delete`

## Edge Cases

- `wet delete <ID>` with nonexistent ID → ThoughtNotFound error
- TUI `x` with no thoughts (empty list) → no-op, stays in Normal mode
- TUI deletion of last thought in filtered view → selection becomes None
- TUI deletion when only one thought exists → list becomes empty, selection None
