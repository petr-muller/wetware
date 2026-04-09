# 002: Entity Descriptions

## Summary

Allow entities to have multi-paragraph plain text descriptions with entity references. Descriptions are displayed as ellipsized previews in the `wet entities` listing, fitting each entity on a single terminal line.

## Requirements

- `wet entity edit <name>` adds/updates a description on an existing entity
- Three input methods: `--description "text"` (inline), `--description-file <path>` (file), or no flags (interactive editor)
- Input methods are mutually exclusive
- Descriptions support entity references using the same syntax as thoughts (`[entity]` and `[alias](entity)`)
- Entity references in descriptions that point to non-existent entities auto-create those entities
- Whitespace-only descriptions clear/remove the description
- `wet entities` shows ellipsized description previews alongside entity names
- Previews show only the first paragraph (split on `\n\n`), with entity references rendered as plain text (no color/bold)
- Preview suppressed when terminal width < 60 characters
- Preview suppressed when available space for preview text < 20 characters
- Entities without descriptions display name only (no placeholder)
- Descriptions can only be added to existing entities, not during creation

## Decisions

- **Single column addition**: `description TEXT` (nullable) added to `entities` table. Simple migration.
- **First-paragraph preview**: split on double newline, use first part only. Keeps previews focused.
- **Ellipsization**: truncate at last word boundary before width limit, append `...` (Unicode ellipsis). Hard truncate if no space found.
- **Preview format**: `<entity-name> - <preview>...` with ` - ` separator (3 chars).
- **Editor fallback chain**: `$EDITOR` -> vim -> nano -> vi.

## CLI Interface

```
wet entity edit <name>                         # Open editor with current description
wet entity edit <name> --description "text"    # Set description inline
wet entity edit <name> --description-file f    # Set description from file
```

`wet entities` output (wide terminal):
```
rust - Rust is a systems programming language that focuses on safety...
systems-programming
wetware - A CLI tool for managing thoughts and entities. Built in Rust.
```

`wet entities` output (narrow terminal, < 60 chars):
```
rust
systems-programming
wetware
```

## Edge Cases

- Description containing only whitespace -> treated as empty, removes description
- Editor crash/abnormal exit -> no changes made, warning displayed
- Entity references in preview rendered without markup or color
- Very long entity names reduce available preview space; if < 20 chars available, preview is suppressed
