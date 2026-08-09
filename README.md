# Wetware

A simple CLI tool for managing networked notes with entity references.

## Features

- Add quick notes via command line, or interactively with entity completion and syntax help
- Edit existing thoughts: correct content, update date, or both
- Reference entities using `[entity-name]` or `[alias](entity-name)` syntax
- Relative dates everywhere a date is accepted: `today`, `yesterday`, `-3d`, `mon`
- Filter notes by entity
- Case-insensitive entity matching with first-occurrence capitalization
- Add multi-paragraph descriptions to entities
- View entity descriptions as previews when listing entities

## Installation

Build from source:

```bash
cargo build --release
```

The binary will be available at `target/release/wetware`.

## Usage

### Add a note

```bash
wet add "Meeting with [Sarah] about [project-alpha]"
```

Backdate it with `--date`, which accepts `YYYY-MM-DD` as well as `today`, `yesterday`, `tomorrow`,
offsets like `-3d` / `-2w` / `-1m`, and weekday names:

```bash
wet add "Retro notes on [project-alpha]" --date yesterday
```

### Add a note interactively

Run `wet add` with no text (or `wet add -i`) to open the composer:

```bash
wet add -i
```

It gives you a date field that shows what your input resolves to as you type, and a thought field where
typing `[` opens a fuzzy search over the entities you already have — including their aliases. Pick one
and it inserts `[Name]`, or `[alias](Name)` if you chose an alias.

To link some wording of your own to an entity, type the wording, then `](` — the search reopens on the
link target and fills in just the entity name:

```
met [my boss](      →  search reopens  →  met [my boss](Alice)
```

A live preview shows how the thought will read, and flags any mention that would create a brand-new
entity, so typos don't quietly become duplicates. You can still type any name you like; nothing forces
you to pick from the list.

`Alt+←` and `Alt+→` shift the date a day earlier or later without leaving the thought field, so
backdating something to yesterday doesn't cost a trip to the date field and back.

`Tab` switches fields (and accepts a completion when the popup is open), `Enter` saves and clears the
thought field so you can keep going, and `Esc` exits.

### Edit an existing thought

Correct the text of thought with ID 3 (IDs shown in `wet thoughts` output as `[id]`):

```bash
wet edit 3 "Corrected content with [Alice]"
```

Edit a thought using your `$EDITOR` (content pre-populated):

```bash
wet edit 3 --editor
```

Update the date of a thought (date only, content unchanged):

```bash
wet edit 3 --date 2026-01-15
```

Update both content and date in a single command:

```bash
wet edit 3 "Updated content" --date 2026-01-15
```

Entity associations are automatically recalculated whenever content changes.

### List all notes

```bash
wet thoughts
```

### Filter notes by entity

```bash
wet thoughts --on Sarah
```

### List all entities

```bash
wet entities
```

Entities with descriptions will show a preview:

```
rust - Rust is a systems programming language that focuses on safety…
wetware - A CLI tool for managing thoughts and entities.
```

### Add or edit entity descriptions

Add a description inline:

```bash
wet entity edit rust --description "Rust is a systems programming language."
```

Add a description from a file:

```bash
wet entity edit rust --description-file description.txt
```

Edit a description interactively (opens $EDITOR):

```bash
wet entity edit rust
```

Remove a description (use empty string or whitespace):

```bash
wet entity edit rust --description ""
```

### Rename an entity

```bash
wet entity rename rust Rustlang
```

Rewrites every stored reference to the entity's old name (in thought content and entity descriptions) to the new name. Existing links are preserved.

### Merge entities

```bash
wet entity merge rustlang --into rust
```

Folds the first entity into the second, for when the same thing ended up recorded under two names. Thoughts, description, aliases and relations all move onto the survivor, which is then the only one left.

References keep the wording originally written — `[rustlang]` becomes `[rustlang](rust)` — so past thoughts still read as you wrote them while pointing at the surviving entity. Unlike a rename, this is not reversible.

The merged-away name is not registered as an alias of the survivor, so a *future* `[rustlang]` would create a fresh entity. To prevent that:

```bash
wet entity alias rust --alias rustlang
```

## Database

By default, notes are stored in `default.db` inside wetware's data directory (`~/.local/share/wetware/` on
Linux/XDG systems). You can override the entire data directory with `WETWARE_DATA_DIR`, or just the
database path with `WETWARE_DB`:

```bash
export WETWARE_DB=/path/to/my/notes.db
wet add "My note"
```

See [`docs/systems/storage.md`](docs/systems/storage.md) for full detail.

## Development

Run tests:

```bash
cargo nextest run
```

Check code coverage:

```bash
cargo tarpaulin
```

## License

See LICENSE file for details.
