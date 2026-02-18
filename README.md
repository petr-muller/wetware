# Wetware

A simple CLI tool for managing networked notes with entity references.

## Features

- Add quick notes via command line
- Reference entities using `[entity-name]` or `[alias](entity-name)` syntax
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

### List all notes

```bash
wet notes
```

### Filter notes by entity

```bash
wet notes --on Sarah
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

## Database

By default, notes are stored in `wetware.db` in the current directory. You can specify a custom database location using the `WETWARE_DB` environment variable:

```bash
export WETWARE_DB=/path/to/my/notes.db
wet add "My note"
```

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
