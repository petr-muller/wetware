---
pr: petr-muller/wetware#227
title: "feat: add `wet entity rename` command"
head_sha: c249bd602d7d97a0e7e29f7ae1a124a9ad8a3226
base: main
reviewed_at: 2026-07-12T18:36:57Z
verdict: approve
refresh_log:
  - from: ae933381e0670b962e691b294683d42dbd6d944d
    to: c249bd602d7d97a0e7e29f7ae1a124a9ad8a3226
    summary: two commits (87ebdce, c249bd6) fixed the should-fix bracket-corruption finding and the README doc-gap nit; both resolved below.
---

## What this PR does
- Adds `wet entity rename <old-name> <new-name>`.
- Rewrites literal bracket references (`[Name]` and `[Alias](Name)`) to the new name across every entity description and every thought linked to the renamed entity, then updates the entity row itself.
- All three steps run in a single transaction; `thought_entities` link rows are untouched since they're keyed by entity ID.
- Rename onto a name already used by a *different* entity is rejected; casing-only self-rename is allowed (collision check compares IDs, not the `UNIQUE COLLATE NOCASE` name column).
- Spec added at `specs/010-entity-rename.md`.

Since previous review:
- `new-name` is now validated to reject `[`, `]`, `(`, `)` (commit 87ebdce), closing the bracket-corruption finding below.
- README gained a "Rename an entity" section with usage example (commit c249bd6).

## Findings

### [nit] redundant collision check
- where: `src/cli/entity_rename.rs:110-115` and `src/storage/entities_repository.rs:344-348`
- concern: "new name already exists" is checked once in the CLI layer before opening the transaction, and again inside `EntitiesRepository::rename`. Harmless for a single-user local CLI (no real TOCTOU exposure) but duplicated logic.

### [nit] error double-printed
- where: `src/cli/entity_rename.rs:103-104,113-114` vs `src/main.rs:72-75`
- concern: `execute()` `eprintln!`s a detailed error message and returns `Err`; `main.rs` then also prints `Error: {}` for the same `Err`. Pre-existing pattern already present in `entity_edit.rs`, not introduced by this PR, but compounds here.

## Resolved

### [should-fix] unescaped `new_name` can corrupt stored bracket syntax — FIXED in 87ebdce
- where: `src/cli/entity_rename.rs:32-35` (new validation), originally flagged at `src/services/entity_parser.rs:113,118`
- resolution: `execute()` now rejects any `new_name` containing `[`, `]`, `(`, or `)` before doing any lookup or transaction work, with a clear `InvalidInput` error. Spec (`specs/010-entity-rename.md`) and requirements updated to document the constraint and its rationale.
- excerpt: |
    if new_name.contains(['[', ']', '(', ')']) {
        return Err(ThoughtError::InvalidInput(
            "New entity name cannot contain '[', ']', '(', or ')' (these are reserved for entity reference syntax)"
                .to_string(),
        ));
    }

### [nit] README not updated — FIXED in c249bd6
- where: `README.md`
- resolution: added a "Rename an entity" section with a `wet entity rename rust Rustlang` example, consistent with the existing `entity edit` section style.

## Checked
- Transaction ordering: descriptions and thought content are rewritten using the pre-rename entity name/row before the row itself is renamed inside the same transaction — lookups stay valid throughout.
- `ThoughtsRepository::update` used by rename is a raw content update with no entity re-linking side effect, so it can't create duplicate entities or drop the just-renamed link (unlike the `wet edit` path which re-parses content).
- `rewrite_entity_references` does plain string comparison, not regex built from user input — no ReDoS/injection risk from entity names containing regex metacharacters.
- `thought_entities` links confirmed untouched by rename (keyed by entity ID) via `test_rename_preserves_link_by_id`.
- New `new_name` bracket/paren validation checked before any DB lookup or transaction, so a rejected rename can't have side effects.
- New validation is covered by `test_entity_rename_rejects_bracket_characters_in_new_name`, which exercises four bad-name variants (`Sarah]x`, `[Sarah`, `Sarah(x)`, `Sarah)`) and confirms original content is untouched by the rejected attempts.
- Ran `cargo nextest run` for the entity_rename tests at the new head (11/11 pass, up from 10 — the new validation test).
- Casing-only self-rename and no-op rename both covered by repository tests and behave correctly against the `UNIQUE COLLATE NOCASE` constraint.
- Rollback-on-failure covered by `test_rename_rollback_on_collision_leaves_everything_unchanged`.

## Open questions
- Any interest in deduplicating the collision check between `cli/entity_rename.rs` and `EntitiesRepository::rename`?
