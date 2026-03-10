# Data Model: Interactive TUI Thought Viewer

**Feature**: 005-tui-viewer
**Date**: 2026-03-09

## Existing Domain Models (unchanged)

### Thought
- `id: Option<i64>` — database primary key
- `content: String` — thought text, may contain entity references
- `created_at: DateTime<Utc>` — creation timestamp

### Entity
- `id: Option<i64>` — database primary key
- `name: String` — lowercase normalized name (for matching)
- `canonical_name: String` — original capitalization (for display)
- `description: Option<String>` — optional multi-paragraph description

## New TUI State Types

### App (root state)
- `thoughts: Vec<Thought>` — all thoughts loaded from database
- `entities: Vec<Entity>` — all entities loaded from database
- `displayed_thoughts: Vec<usize>` — indices into `thoughts` for current view (after filtering/sorting)
- `list_state: ListState` — ratatui list selection/scroll state
- `mode: Mode` — current interaction mode
- `sort_order: SortOrder` — current sort direction
- `active_filter: Option<String>` — entity name currently filtering by (None = show all)
- `should_quit: bool` — exit flag

### Mode (enum)
- `Normal` — browsing thought list, standard key bindings active
- `EntityPicker { input: Input, matches: Vec<usize> }` — fuzzy entity picker overlay open
  - `input`: tui-input state for the search field
  - `matches`: indices into `entities` matching current input, sorted by fuzzy score
  - `selected: usize`: currently highlighted match in the picker list
- `EntityDetail { entity_indices: Vec<usize> }` — entity description popup showing
  - `entity_indices`: indices into `entities` for entities referenced in selected thought
  - `scroll_offset: usize`: scroll position within the description popup

### SortOrder (enum)
- `Ascending` — oldest first (default)
- `Descending` — newest first

## State Transitions

```
Normal
  ├── `/` → EntityPicker (open fuzzy picker)
  ├── `s` → Normal (toggle SortOrder, recompute displayed_thoughts)
  ├── `Enter` or `d` → EntityDetail (show descriptions for selected thought's entities)
  ├── `q` or `Esc` → Quit (if no active filter)
  ├── `Esc` → Normal (clear active_filter if set)
  ├── `↑/↓/PgUp/PgDn/Home/End` → Normal (scroll/select in list)
  └── `?` → (help indicator already visible in status bar)

EntityPicker
  ├── typing → EntityPicker (update input, recompute fuzzy matches)
  ├── `↑/↓` → EntityPicker (move selection in picker list)
  ├── `Enter` → Normal (apply selected entity as filter, recompute displayed_thoughts)
  └── `Esc` → Normal (close picker, no filter change)

EntityDetail
  ├── `↑/↓` → EntityDetail (scroll within description popup)
  └── `Esc` → Normal (close popup)
```

## Data Flow

1. **Startup**: Load all thoughts and entities from SQLite via existing repositories
2. **Display**: Compute `displayed_thoughts` from `thoughts` based on `active_filter` and `sort_order`
3. **Filter**: When entity selected in picker, set `active_filter` and recompute `displayed_thoughts`
4. **Sort**: When toggled, flip `sort_order` and recompute `displayed_thoughts`
5. **Entity detail**: Parse selected thought's content for entity references, look up matching entities
