# Architecture

## Purpose

Cross-system structure and durable technical decisions for wetware. Individual module behavior lives in
[`../systems/`](../systems/); the rationale behind specific decisions lives in
[`decisions/`](decisions/) as ADRs. This document covers the shape of the system as a whole.

## Layered architecture

```
src/
├── cli/          CLI commands (clap subcommands)
├── models/       Domain types (Thought, Entity, SortOrder)
├── services/     Business logic, no I/O (entity_parser, entity_styler, description_formatter, color_mode)
├── storage/      SQLite persistence (repositories, migrations, connection)
├── input/        User input handling (editor integration)
├── tui/          Interactive TUI viewer (state, ui, input)
├── errors/       ThoughtError, used across the whole crate
├── config.rs     TOML config file
├── lib.rs        Library root
└── main.rs       Binary entry point
```

**Dependency direction rule**: `models/` depends on nothing but `errors/` and external crates — it must
never depend on `cli/`, `storage/`, `services/`, or `tui/`. `services/` depends on `models/`/`errors/`
only, no I/O, which is what makes it reusable from both `cli/` and `tui/`. `storage/` is the only layer
that talks to SQLite; everything else goes through its repositories. `cli/` and `tui/` are the two
consumers that tie `models/`, `services/`, and `storage/` together for a given interface.

See [`../systems/`](../systems/) for what each layer actually does.

## Core data model

```sql
CREATE TABLE thoughts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL CHECK(length(trim(content)) > 0 AND length(content) <= 10000),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_thoughts_created_at ON thoughts(created_at);

CREATE TABLE entities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE CHECK(length(trim(name)) > 0),
    canonical_name TEXT NOT NULL,
    description TEXT
);

CREATE TABLE thought_entities (
    thought_id INTEGER NOT NULL,
    entity_id INTEGER NOT NULL,
    PRIMARY KEY (thought_id, entity_id),
    FOREIGN KEY (thought_id) REFERENCES thoughts(id) ON DELETE CASCADE,
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);
CREATE INDEX idx_thought_entities_entity ON thought_entities(entity_id);
CREATE INDEX idx_thought_entities_thought ON thought_entities(thought_id);

CREATE TABLE entity_relations (
    child_id INTEGER NOT NULL,
    parent_id INTEGER NOT NULL,
    PRIMARY KEY (child_id, parent_id),
    FOREIGN KEY (child_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES entities(id) ON DELETE CASCADE,
    CHECK (child_id != parent_id)
);
CREATE INDEX idx_entity_relations_parent ON entity_relations(parent_id);
CREATE INDEX idx_entity_relations_child ON entity_relations(child_id);
```

A normalized shape: Thoughts and Entities are independent rows, linked many-to-many through
`thought_entities`. Entity references inside `thoughts.content` and `entities.description` are stored as
literal bracket-markup text (`[entity]` / `[alias](entity)`), re-parsed at read/write time rather than
kept as a separate structured representation — see
[`decisions/0001-networked-notes-schema.md`](decisions/0001-networked-notes-schema.md) and
[`decisions/0004-entity-reference-aliases.md`](decisions/0004-entity-reference-aliases.md) for why.
`entity_relations` is a directed parent/child edge table forming a DAG over entities — reachability
(descendants of a given entity) is computed with a recursive CTE at read time, not a materialized
closure table — see [`decisions/0012-entity-relations.md`](decisions/0012-entity-relations.md).

Full detail (repository methods, migration mechanics) is in [`../systems/storage.md`](../systems/storage.md).

## Cross-cutting concerns

- **Errors** — a single `ThoughtError` enum (`errors/`) is the `Result` error type across the entire
  crate. See [`../systems/errors.md`](../systems/errors.md).
- **Color mode** — `Always`/`Auto`/`Never`, shared between `wet thoughts`/`wet entity show` (CLI) styling
  and (independently) TUI styling. See [`../systems/services.md`](../systems/services.md) for the known
  CLI/TUI color-assignment inconsistency.
- **Data directory resolution** — XDG-based, with a debug-build panic guard to keep dev/test runs from
  touching real user data. See [`../systems/storage.md`](../systems/storage.md) and
  [`decisions/0007-data-directory.md`](decisions/0007-data-directory.md).

## Decisions index

All accepted architectural decisions are recorded as ADRs in [`decisions/`](decisions/), numbered in the
order they were made:

| ADR | Decision |
|---|---|
| [0001](decisions/0001-networked-notes-schema.md) | Normalized three-table schema + bracket entity references |
| [0002](decisions/0002-entity-descriptions.md) | Multi-paragraph entity descriptions with ellipsized previews |
| [0003](decisions/0003-styled-entity-output.md) | Styled (colored/bold) entity rendering with TTY auto-detection |
| [0004](decisions/0004-entity-reference-aliases.md) | `[alias](entity)` reference syntax |
| [0005](decisions/0005-edit-thoughts.md) | `wet edit` with atomic entity re-association |
| [0006](decisions/0006-tui-viewer.md) | ratatui/TEA-based interactive TUI |
| [0007](decisions/0007-data-directory.md) | XDG data directory with debug-build panic guard |
| [0008](decisions/0008-delete-thoughts.md) | `wet delete` (CLI, no confirm) + TUI confirm-delete |
| [0009](decisions/0009-config-command.md) | `wet config` with hardcoded per-key match arms |
| [0010](decisions/0010-entity-rename.md) | Literal text rewrite on rename, ID-keyed links untouched |
| [0011](decisions/0011-entity-show.md) | `wet entity show` detail view, full description + 5 latest thoughts |
| [0012](decisions/0012-entity-relations.md) | Directed entity parent/child relations (DAG) with recursive-CTE reachability |

Only ADRs with `status: Accepted` reflect current guidance — see each ADR's frontmatter.

## Related docs

- [`../systems/`](../systems/) — how each layer actually works.
- [`../flows/`](../flows/) — cross-system behavior traces.
- [`../glossary.md`](../glossary.md)
