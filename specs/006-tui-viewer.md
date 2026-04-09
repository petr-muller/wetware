# 006: Interactive TUI Thought Viewer

## Summary

Read-only interactive TUI for browsing thoughts, launched via `wet tui`. Provides a scrollable thought list with entity highlighting, fuzzy entity picker for filtering, sort order toggling, and entity description modals.

## Requirements

- `wet tui` launches the interactive viewer
- Thoughts displayed in a scrollable list showing date and content
- Entity references highlighted with colors (reusing existing entity styling)
- Keyboard navigation: arrow keys, Page Up/Down, Home/End for scrolling
- `q` or `Esc` exits the TUI and restores terminal state
- `/` opens a fuzzy entity picker overlay listing all entities
- Fuzzy search narrows the entity list as user types
- Selecting an entity filters the thought list to matching thoughts
- `Esc` in picker closes it without applying filter; filter can be cleared to show all thoughts
- `s` toggles sort order between ascending and descending by date
- Current sort order indicated in the UI
- Sort and filter are composable (sorting preserves active filter)
- `Enter` or `d` on a selected thought opens a centered modal popup showing entity descriptions
- Modal shows descriptions for all entities referenced in the selected thought
- Entities without descriptions indicated as such in the modal
- `Esc` dismisses the modal
- Help indicator/key legend visible in the UI
- Empty database and no-match filter results show informative messages
- Default sort: ascending (oldest first), matching existing `wet thoughts` behavior

## Decisions

- **ratatui + crossterm**: TUI framework and terminal backend. Cross-platform, well-maintained.
- **The Elm Architecture (TEA)**: state management pattern for the TUI.
- **tui-input**: crate for text input widget in the fuzzy picker.
- **nucleo-matcher**: crate for fuzzy matching in the entity picker.
- **Read-only**: no database modifications from the TUI. All data loaded upfront.
- **Single-threaded synchronous event loop**: sufficient for personal-scale data.

## CLI Interface

```
wet tui       # Launch the interactive TUI viewer
```

Key bindings:
| Key | Action |
|-----|--------|
| `q`, `Esc` | Quit TUI (or dismiss overlay/modal) |
| Up/Down | Scroll thought list |
| PgUp/PgDn | Page through thought list |
| `/` | Open entity filter picker |
| `s` | Toggle sort order (asc/desc) |
| `Enter`/`d` | Show entity descriptions for selected thought |

## Edge Cases

- Empty database: informative empty state message
- Very narrow terminal: layout adapts, content truncated as needed
- Terminal resize: TUI redraws to fit new dimensions
- Very long thought content: truncated in list view with indicator
- Entity descriptions containing entity references: highlighted consistently in modal
