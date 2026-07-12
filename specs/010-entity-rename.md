# 010: Entity Rename

## Summary

Allow renaming an entity via `wet entity rename <old-name> <new-name>`. Since entity references are stored as literal bracket text embedded in thought content and entity descriptions (not as pure foreign-key placeholders), a rename rewrites every stored literal occurrence of the old name to the new name, in addition to updating the entity's own row. `thought_entities` links are keyed by entity ID and are never touched — they remain valid across the rename automatically.

## Requirements

- `wet entity rename <old-name> <new-name>` renames an existing entity, matched case-insensitively on `old-name`.
- Every thought linked to the entity has literal references to the old name rewritten to the new name:
  - Bare `[OldName]` becomes `[NewName]`.
  - Aliased `[Alias](OldName)` becomes `[Alias](NewName)` — only the target is rewritten, the alias/display text is left untouched.
- Every entity's description (including the renamed entity's own description) that references the old name is rewritten using the same rules.
- Text where the old name coincidentally appears as someone else's alias *display* text pointing at a different target (e.g. `[Sarah](project-alpha)` when renaming a `Sarah` entity) is left unchanged — it isn't a reference to this entity.
- The rename (row update + all content rewrites) is atomic: if any step fails, nothing is changed.
- Renaming to a name already used by a *different* entity fails with an error; no merge behavior.
- Renaming to the same name, or only changing casing (e.g. `Sarah` -> `Sarah Smith` vs. `Sarah` -> `SARAH`), succeeds.

## Decisions

- **Literal rewrite over alias-preserving rewrite**: on rename, stored text is updated to show the new name everywhere, rather than turning old occurrences into an implicit `[OldName](NewName)` alias. Simpler mental model — text always reflects the current entity name. Considered and rejected the alias-preserving approach (`[Sarah]` -> `[Sarah](Sarah Smith)`) since it would leave stale-looking display text and add complexity for the same end goal (keeping links intact).
- **Links are never re-derived on rename**: `thought_entities` rows are keyed by `entity_id`, not name, so they already remain valid without any action. The content rewrite exists purely to keep displayed/stored text in sync with the new name and to prevent duplicate-entity creation the next time an affected thought's content is re-parsed (e.g. via `wet edit`, which re-extracts entities by name from content on every content change).
- **Rewrite scope limited to entity's own thoughts**: only thoughts already linked to the entity (via `thought_entities`) are scanned for text rewriting, not every thought in the database.
- **All-entity description scan**: unlike thoughts, descriptions are scanned across *all* entities (not just the renamed one), since any entity's description text may reference any other entity.
- **No persisted alias table**: consistent with the existing design (spec 004) where aliases are purely per-occurrence text, not a stored alternate-name registry. Rename does not introduce one.
- **No entity-merge functionality**: renaming onto an existing different entity is rejected rather than merging the two entities' thoughts/descriptions together. Out of scope.
- **Self-rename (casing-only) allowed**: the `entities.name` column is `UNIQUE COLLATE NOCASE`, so a naive collision check against that column would false-positive when renaming `Sarah` -> `SARAH`. The collision check instead compares entity IDs, allowing a row to be renamed to a re-cased version of itself.

## CLI Interface

```
wet entity rename sarah "Sarah Smith"
```

Renames the entity currently known as "sarah" (case-insensitive match) to "Sarah Smith", rewriting all thought and description text that references it.

Errors:
```
wet entity rename nonexistent "New Name"
# Error: Entity 'nonexistent' not found

wet entity rename sarah john
# Error: Entity 'john' already exists
```

## Edge Cases

- Renaming an entity with no thoughts or descriptions referencing it: succeeds, only the entity row changes.
- Renaming to the exact same name (no-op): succeeds.
- Renaming with only a casing change (`Sarah` -> `SARAH`): succeeds, updates `canonical_name`.
- A thought referencing the entity via an aliased form (`[Alias](sarah)`) has only the target rewritten; the alias display text is preserved verbatim.
- A thought that happens to contain the old name as alias display text for an *unrelated* entity (`[Sarah](project-alpha)`) is left untouched.
- An unrelated thought that doesn't reference the entity at all is left byte-identical.
- Partial failure mid-rewrite (e.g. a storage error) rolls back the entire operation — no thought, description, or the entity row itself is left partially updated.
