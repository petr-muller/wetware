# 004: Entity Reference Aliases

## Summary

Allow entity references to use markdown-like alias syntax `[alias](entity)` for natural language in thoughts. When displayed, only the alias text is shown (in colored highlight), while the system links to the target entity for queries and filtering. Fully backward compatible with traditional `[entity]` syntax.

## Requirements

- Accept `[alias](entity)` syntax alongside traditional `[entity]` in thought text
- Display only the alias text in colored highlight when rendering thoughts
- Color assigned by target entity, not alias (same entity = same color regardless of alias used)
- For entity extraction/queries, return the target entity name (not alias)
- Alias and entity reference both trimmed of whitespace
- Empty alias `[](entity)` or empty reference `[alias]()` rejected (treated as plain text)
- Both syntaxes can coexist in the same thought
- No database migration needed: thoughts stored as raw text, entity extraction at runtime
- Backward compatible: existing `[entity]` syntax works identically to before

## Decisions

- **Single regex**: `\[([^\[\]]+)\](?:\(([^\(\)]+)\))?` handles both syntaxes. Group 1 = display text, optional group 2 = target entity.
- **No DB migration**: raw content stored as-is, extraction at runtime. Aliases are purely presentational.
- **Regex over parser combinator**: pattern is simple enough, no nesting needed.
- **Validation at parse time**: malformed syntax silently ignored (treated as plain text), matching existing behavior.
- **Migrated from lazy_static to std::sync::LazyLock** for the regex pattern.

## CLI Interface

`wet add` accepts both syntaxes in thought text:
```
wet add "I learned about [robots](robotics) today"
wet add "Reading about [robotics]"
wet add "Mixed: [robotics] and [robot](robotics)"
```

`wet thoughts` renders alias with entity color:
```
I learned about robots today          # "robots" shown in robotics' color
Reading about robotics                # "robotics" shown in its color
Mixed: robotics and robot             # both in same color (same target entity)
```

## Edge Cases

- `[x](x)` where alias equals entity: valid, accepted
- Multi-word aliases `[my robot project](robotics)`: supported
- Malformed `[alias](` without closing paren: treated as plain text
- Deduplication: `[robotics]` and `[robot](robotics)` in same thought -> one entity extracted ("robotics")
- Alias cannot contain brackets `[]` or parentheses `()`
