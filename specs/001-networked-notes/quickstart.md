# Quickstart: Networked Notes with Entity References

**Feature**: 001-networked-notes
**Last Updated**: 2025-12-30

## What is This?

Wetware's networked notes feature lets you capture quick thoughts and connect them through named entities. Use `[bracket notation]` to reference people, projects, or any concept you want to track across your notes.

## Quick Examples

### Add a simple note

```bash
wet add 'Remember to review PR #42'
```

Output:
```
✓ Note added
```

### Add a note with entity references

```bash
wet add 'Meeting with [Sarah] about [project-alpha]'
```

Output:
```
✓ Note added (2 entities: Sarah, project-alpha)
```

### List all your notes

```bash
wet notes
```

Output:
```
[2025-12-30 14:23] Remember to review PR #42
[2025-12-30 14:25] Meeting with [Sarah] about [project-alpha]
```

### Find notes about a specific entity

```bash
wet notes --on Sarah
```

Output:
```
[2025-12-30 14:25] Meeting with [Sarah] about [project-alpha]
```

### See all entities you've referenced

```bash
wet entities
```

Output:
```
project-alpha
Sarah
```

## Core Concepts

### Notes

Notes are short text entries you create with `wet add`. Each note:
- Has a timestamp (automatic)
- Can reference zero or more entities
- Is immutable once created (no editing)
- Can be up to 10,000 characters

### Entities

Entities are names you put in `[brackets]` within notes. They represent:
- People: `[Sarah]`, `[John Smith]`
- Projects: `[project-alpha]`, `[Q4-planning]`
- Topics: `[meeting]`, `[budget-review]`
- Anything you want to track!

**Key behaviors**:
- **Case-insensitive**: `[Sarah]`, `[sarah]`, and `[SARAH]` all refer to the same entity
- **First wins**: The first capitalization you use becomes the display form
- **Automatic tracking**: Entities are extracted and stored automatically

### Entity References

When you write `[entity-name]` in a note, Wetware:
1. Extracts the entity name
2. Creates or finds the entity (case-insensitive)
3. Links the note to that entity
4. Lets you filter notes by entity later

## Common Workflows

### Daily standup notes

```bash
# Monday
wet add 'Worked on [feature-X] with [Alice]'

# Tuesday
wet add 'Code review for [feature-X], blocked on [backend-API]'

# Wednesday
wet add 'Pair programming with [Alice] on [backend-API]'

# Review what happened with feature-X
wet notes --on feature-X
```

Output:
```
[2025-12-30 09:00] Worked on [feature-X] with [Alice]
[2025-12-31 09:00] Code review for [feature-X], blocked on [backend-API]
```

### Meeting notes

```bash
# Capture meeting notes
wet add 'Sprint planning: [Alice] owns [feature-X], [Bob] owns [feature-Y]'

# Later, find all notes about Alice
wet notes --on Alice
```

### Project tracking

```bash
# Add project-related notes
wet add '[project-alpha] kickoff meeting scheduled'
wet add '[project-alpha] design doc ready for review'
wet add '[project-alpha] backend API deployed to staging'

# See project timeline
wet notes --on project-alpha
```

## Entity Syntax Rules

### Valid entity syntax

✅ **Simple names**
```bash
wet add 'Call [Sarah]'
```

✅ **Multiple words (with spaces)**
```bash
wet add 'Review [quarterly budget]'
```

✅ **Hyphens and underscores**
```bash
wet add 'Deploy [project-alpha] to [staging-env]'
wet add 'Fix [bug_123]'
```

✅ **Numbers**
```bash
wet add 'Close [issue-42]'
```

### Invalid/ignored syntax

❌ **Empty brackets**
```bash
wet add 'This [] is ignored'
# Entity not extracted
```

❌ **Unclosed brackets**
```bash
wet add 'This [is broken'
# Entity not extracted
```

❌ **Nested brackets**
```bash
wet add 'This [[inner]] is weird'
# Only [inner] is extracted
```

### Case sensitivity

Entities are case-insensitive but preserve your first usage:

```bash
# First use - sets display form
wet add 'Meeting with [Sarah]'

# Later uses - same entity, different casing
wet add '[sarah] approved the PR'
wet add '[SARAH] is on vacation'

# All three notes reference the same entity
wet notes --on Sarah
```

Output shows all three notes, and `wet entities` shows "Sarah" (first form).

## Tips & Best Practices

### Keep notes atomic

✅ **Good**: One thought per note
```bash
wet add '[Sarah] approved feature-X PR'
wet add '[Bob] requested changes on feature-Y'
```

❌ **Less useful**: Multiple unrelated thoughts
```bash
wet add 'Sarah approved feature-X, Bob requested changes on feature-Y, also need to buy groceries'
```

### Use consistent entity naming

✅ **Good**: Pick one form and stick with it
```bash
wet add 'Meeting with [Sarah]'
wet add '[Sarah] sent the proposal'
```

❌ **Confusing**: Different forms for the same person
```bash
wet add 'Meeting with [Sarah]'
wet add '[Sarah Johnson] sent the proposal'
# These are treated as different entities!
```

### Leverage case-insensitivity

Don't worry about exact capitalization when searching:

```bash
wet notes --on sarah       # Works
wet notes --on Sarah       # Works
wet notes --on SARAH       # Works
```

### Use descriptive entity names

✅ **Good**: Clear, searchable names
```bash
wet add 'Review [Q4-budget-proposal]'
wet add 'Deploy [backend-api-v2]'
```

❌ **Less useful**: Vague names
```bash
wet add 'Review [doc]'
wet add 'Deploy [thing]'
```

## Advanced Usage

### Multiple entity references

```bash
wet add 'Discussed [project-alpha] and [project-beta] with [Sarah] and [John]'
```

All four entities are extracted and linked to this note.

### Finding all connections

```bash
# What projects did Sarah work on?
wet notes --on Sarah | grep '\[project-'

# What did we discuss in meetings?
wet notes --on meeting
```

### Discovering your knowledge graph

```bash
# See all entities you've referenced
wet entities

# Pick an interesting entity
wet notes --on <entity-name>
```

## Troubleshooting

### "No notes found"

**Cause**: No notes have been added yet.

**Solution**:
```bash
wet add 'My first note'
```

### "No entities found"

**Cause**: None of your notes have entity references.

**Solution**: Add brackets around entity names:
```bash
wet add 'Meeting with [Sarah]'
```

### "No notes found referencing 'X'"

**Cause**: No notes contain `[X]`.

**Solutions**:
1. Check spelling: `wet entities` (to see all entities)
2. Try different capitalization: `wet notes --on x` (case-insensitive)
3. The entity might not exist: add a note referencing it

### Note too long

**Error**: `Note exceeds maximum length of 10000 characters`

**Solution**: Break your note into smaller pieces:
```bash
wet add 'Part 1: [topic] summary...'
wet add 'Part 2: [topic] details...'
```

## Command Reference

### `wet add <text>`

Add a new note.

**Example**:
```bash
wet add 'Your note text here with optional [entities]'
```

### `wet notes`

List all notes chronologically.

**Example**:
```bash
wet notes
```

### `wet notes --on <entity>`

Filter notes by entity (case-insensitive).

**Example**:
```bash
wet notes --on Sarah
```

### `wet entities`

List all unique entities.

**Example**:
```bash
wet entities
```

## What's Next?

Now that you know the basics:

1. **Start capturing**: Use `wet add` to record thoughts as they come
2. **Build connections**: Reference people, projects, and topics with `[brackets]`
3. **Discover patterns**: Use `wet notes --on` to find related notes
4. **Explore your graph**: Use `wet entities` to see what you're tracking

Happy note-taking! 📝
