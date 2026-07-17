# Documentation Style

These docs are written for both humans and AI agents. Good documentation is navigational and
explanatory, not exhaustive — it helps a reader know what matters, where the details live, and what
could break if the system changes.

## Writing

- Use clear, direct Markdown.
- Keep headings stable and matching the relevant template in [`templates/`](templates/), so a reader (or
  agent) can jump straight to a section by name across docs.
- Explain behavior, responsibilities, flows, invariants, and pitfalls. Don't restate implementation
  detail that's easier to read directly from the source.
- Avoid unsupported guesses. If something is hard to verify from the code, say so explicitly (e.g.
  "Unverified:" or "Assumed based on X, not confirmed").
- Stay concise enough to read in full before making a change to that system.

## Linking

- Link to related docs and source files using relative Markdown links from the current file's location
  (e.g. a doc in `docs/systems/` links to source with `../../src/...`).
- Every system and flow doc ends with a **Source map** linking to the files that matter most — not every
  file, unless the system is small.

## Glossary usage

- [`glossary.md`](glossary.md) entries use Title Case headings (e.g. "Entity Reference").
- In finished docs, prefer Title Case when referring to a glossary-defined concept if it improves clarity
  or distinguishes an application-specific meaning from the ordinary word. Lowercase is fine in code,
  commit messages, and informal notes.

## Keeping docs accurate

Documentation is only useful if it matches the code. When a change affects a system's responsibilities,
runtime behavior, data model, interfaces, error handling, security behavior, or invariants:

- Update the relevant `docs/systems/` and/or `docs/flows/` doc in the same change.
- Add or update an ADR under `docs/architecture/decisions/` if the change reflects a durable technical
  decision (see [`architecture/README.md`](architecture/README.md)).
- If docs and code disagree and it's not obvious which is right, flag the mismatch explicitly rather than
  guessing — either fix the doc, fix the code, or call it out for review.
