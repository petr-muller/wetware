---
status: Accepted
date: "2026-07-28"
---

# Entity Merge

## Context

Entities are created implicitly, by name, the first time a bracket reference mentions them (see
[`0001-networked-notes-schema.md`](0001-networked-notes-schema.md)). That makes duplicates easy to
produce: `[k8s]` and `[kubernetes]`, or `[Alice]` and `[Alice Smith]`, become two independent entities,
each accumulating its own thoughts, description, aliases and relations.

[`0010-entity-rename.md`](0010-entity-rename.md) deliberately left this unsolved: renaming onto an
existing name is a hard error, and merge was declared out of scope with its questions unanswered ("whose
description wins? do both entities' thought histories combine?"). This ADR answers them.

## Decision

`wet entity merge <entity-name> --into <target>` folds the first entity into the second. Both names are
resolved alias-aware; merging an entity into itself is rejected (`SelfMerge`). Everything below happens in
one transaction — see [`../../flows/entity-merge.md`](../../flows/entity-merge.md) for the ordering.

**Stored references keep their original wording.** Bare `[Alice]` becomes `[Alice](Bob)`, and
`[Al](Alice)` becomes `[Al](Bob)` — the display text survives, the target changes. This is the
alias-preserving rewrite that ADR 0010 *rejected* for rename, and the asymmetry is deliberate: a rename
says "this thing is now called something else", so old text is stale and should be updated; a merge says
"these two names were always the same thing", and the wording that was actually written at the time is
history worth keeping. Rewriting `[Alice]` to `[Bob]` would silently edit what past thoughts say.
A reference whose display text already names the target collapses to bare `[Bob]` rather than the
redundant `[Bob](Bob)`.

**Both histories combine.** `thought_entities` rows are re-pointed from the source onto the target with
`INSERT OR IGNORE`, so a thought that referenced both entities keeps a single link. This is the one place
merge does more work than rename: rename never touches the link table because IDs don't change, whereas
here the source's ID is about to disappear.

**Both descriptions are kept**, the source's appended to the target's after a blank line, rather than
picking a winner. The source's aliases and parent/child relations transfer to the target; edges that would
become self-relations or close a cycle once the two entities collapse are dropped rather than erroring.
The source row is then deleted, and `ON DELETE CASCADE` clears whatever wasn't copied.

**The merged-away name is not auto-registered as an alias of the survivor.** Merge is not stated as an
opinion about what the old name means going forward, so it doesn't quietly claim it.

**No confirmation prompt**, matching every other `wet` command
(see [`0008-delete-thoughts.md`](0008-delete-thoughts.md)).

**An entity whose name contains `(` or `)` cannot be a merge target**, and the command rejects it up
front, pointing at `wet entity rename` as the fix. Such names arise through ordinary use — group 1 of
`ENTITY_PATTERN` permits parentheses, so `wet add "[Alice (HR)]"` creates one — but the target group is
`[^()]+`, so `[Bob](Alice (HR))` reads back as a bare `[Bob]`. That desynchronizes stored text from the
link table, and since `wet edit` re-extracts entities by name, the next edit of an affected thought would
recreate the merged-away entity and silently undo the merge for it. Rejecting was chosen over falling back
to rename-style wording replacement for those targets: the fallback would let the merge succeed, but by
quietly abandoning the wording-preservation property that is the whole point of the decision above, in the
one case the user cannot see coming. `entity_rename.rs` already guards the same character class.

Merge reports what it moved, rewrote, and **dropped** — including relation edges discarded as
self-relations or cycles. Dropping them is right, but doing so silently *and* irreversibly is not.

## Consequences

- Duplicate entities are now fixable without losing anything: no thought, description, alias or relation
  is dropped by a merge.
- Stored text accumulates more explicit `[display](target)` markup over time. Rendering is unaffected —
  `entity_styler` already displays group 1 and colors by group 2 — but the raw text a user sees in
  `wet edit` gets busier with each merge.
- Because the old name isn't aliased to the survivor, typing `[Alice]` in a *future* thought creates a
  fresh `Alice` entity. Users who want the old name to keep resolving run
  `wet entity alias <target> --alias <old name>` afterwards. This is one extra step, but the alternative
  silently makes a name mean something the user never said it meant.
- Merge is irreversible and, like rename, a potentially large write (every entity's description plus every
  thought reachable from the source, in one transaction).
- Appending descriptions can produce awkward prose when both entities described the same thing. The user
  edits it afterwards; the system does not attempt to reconcile the two texts.
- A parenthesized entity name is now a second-class merge participant: usable as a source, never as a
  target, requiring a rename first. This is a real usability wart, accepted because the alternative is a
  silent data-integrity bug. It would disappear if reference syntax ever gained escaping.

## Alternatives considered

- **Rewrite `[Alice]` to `[Bob]`** (reusing `rewrite_entity_references` verbatim, exactly what rename
  does) — rejected: zero new code, but it rewrites the visible wording of thoughts the user wrote, which
  is a different and more invasive claim than re-pointing a link.
- **Falling back to that rewrite for parenthesized targets** instead of rejecting them — rejected, see
  the target-name rule above: it trades a visible error for an invisible semantic downgrade.
- **Discard the source's description** — rejected: silently loses text the user wrote, and there is no
  undo.
- **Register the source's name as an alias of the target automatically** — considered and rejected as a
  default; it is a one-line follow-up command when wanted, and doing it unasked commits the user to an
  interpretation of the old name. Worth revisiting as an opt-in flag if it turns out to be what people
  always want.
- **Requiring `--yes` or printing a dry-run first** — rejected for consistency with the rest of the CLI,
  where destructive commands act immediately and only the TUI confirms.
- **Merging on rename collision** (making `wet entity rename` fall back to a merge) — rejected: renaming
  and merging are different intents, and a typo'd rename should not silently destroy an entity.

## Related code

- [`src/cli/entity_merge.rs`](../../../src/cli/entity_merge.rs)
- [`src/services/entity_parser.rs`](../../../src/services/entity_parser.rs) (`redirect_entity_references`)
- [`src/storage/entities_repository.rs`](../../../src/storage/entities_repository.rs)
  (`repoint_thought_links`, `delete`)

## Related docs

- [`../../flows/entity-merge.md`](../../flows/entity-merge.md),
  [`../../flows/entity-rename.md`](../../flows/entity-rename.md)
- [`0010-entity-rename.md`](0010-entity-rename.md), [`0013-entity-aliases.md`](0013-entity-aliases.md)
- [`0001-networked-notes-schema.md`](0001-networked-notes-schema.md),
  [`0004-entity-reference-aliases.md`](0004-entity-reference-aliases.md)
