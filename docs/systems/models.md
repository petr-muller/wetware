# Models

## Purpose

Domain types for wetware: `Thought`, `Entity`, and `SortOrder`. These are the plain data structures every
other system operates on.

## Questions this doc answers

- What fields does a Thought or Entity have?
- What validation applies when constructing one?
- Why is there both a `name` and a `canonical_name` on Entity?

## Scope

`Thought`, `Entity`, `SortOrder`, and their constructors/validation.

## Non-scope

Persistence (see [`storage.md`](storage.md)), entity-reference parsing/rendering (see
[`services.md`](services.md)), and CLI argument handling (see [`cli.md`](cli.md)).

## Key concepts

- **Thought** — a dated snippet of text. See [glossary](../glossary.md#thought).
- **Entity** — a named, describable thing a Thought can reference. See
  [glossary](../glossary.md#entity).
- **Canonical Name** — an Entity's originally-typed casing, kept separate from its lowercased lookup
  `name`. See [glossary](../glossary.md#canonical-name).
- **Sort Order** — `Ascending` or `Descending`. See [glossary](../glossary.md#sort-order).

## How the system works

`models/` has no dependencies on `cli`, `storage`, or any other layer besides `errors` and external
crates (`chrono`, `serde`) — this is a deliberate architectural rule (see
[`../architecture/README.md`](../architecture/README.md)) so domain types stay reusable and testable in
isolation.

- `Thought::new(content)` and `Thought::new_with_date(content, created_at)` validate content through a
  private `validate_content`: rejects empty/whitespace-only content (`ThoughtError::EmptyContent`) and
  content over 10,000 characters (`ThoughtError::ContentTooLong { max, actual }`).
- `Entity::new(name)` lowercases `name` for case-insensitive lookup while storing the original casing in
  `canonical_name`. `Entity::with_description(name, description)` additionally sets a description.
  `display_name()`, `has_description()`, and `description_or_empty()` are convenience accessors.
- `SortOrder` implements `Display`/`FromStr` (string forms `"ascending"`/`"descending"`), `toggle()`, and
  `label()` (human-readable "Oldest first" / "Newest first", used by the TUI status bar).

## Important flows

None — models are passive data types with no multi-step behavior of their own.

## Data and state

```rust
struct Thought { id: Option<i64>, content: String, created_at: DateTime<Utc> }
struct Entity { id: Option<i64>, name: String, canonical_name: String, description: Option<String> }
enum SortOrder { Ascending, Descending }
```

`id` is `None` until the value has been persisted and assigned a row ID by [`storage.md`](storage.md).

## Interfaces and entry points

`Thought::new`, `Thought::new_with_date`, `Entity::new`, `Entity::with_description`, `Entity::display_name`,
`Entity::has_description`, `Entity::description_or_empty`, `SortOrder::toggle`, `SortOrder::label`.

## Dependencies

`errors` (for `ThoughtError`), `chrono`, `serde`.

## Downstream effects

Every other system (`storage`, `services`, `cli`, `tui`) consumes these types directly. A field or
validation change here ripples everywhere.

## Invariants and assumptions

- Thought content is always non-empty and ≤10,000 characters by the time it reaches storage — enforced
  at construction, not at the storage layer.
- `Entity::name` is always the lowercased form of `canonical_name`.

## Error handling

Validation failures return `ThoughtError::EmptyContent` or `ThoughtError::ContentTooLong`; construction
never panics.

## Security and privacy notes

Not applicable — no security-sensitive behavior at this layer.

## Observability and debugging

Not applicable — these are plain data types with no I/O or logging.

## Testing notes

Unit tests cover the validation boundaries directly (empty content, exactly-10,000-char content,
10,001-char content) and the `SortOrder` string round-trip.

## Common pitfalls

- Don't confuse `Entity::name` (lowercased, used for lookups/uniqueness) with `canonical_name` (display
  form). Using the wrong one for a comparison silently breaks case-insensitive matching.

## Source map

- [`src/models/thought.rs`](../../src/models/thought.rs)
- [`src/models/entity.rs`](../../src/models/entity.rs)
- [`src/models/sort_order.rs`](../../src/models/sort_order.rs)
- [`src/models/mod.rs`](../../src/models/mod.rs)

## Related docs

- [`storage.md`](storage.md) — persistence for these types.
- [`services.md`](services.md) — entity-reference parsing and rendering built on top of these types.
- [Glossary](../glossary.md)
