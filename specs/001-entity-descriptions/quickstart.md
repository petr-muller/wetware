# Quickstart: Entity Descriptions

**Feature**: 001-entity-descriptions
**Date**: 2026-02-01

## Overview

This quickstart guide demonstrates how to use entity descriptions in the wetware CLI tool. Entity descriptions allow you to add multi-paragraph context to entities, with support for entity references.

## Prerequisites

- Wetware CLI installed (`wet` command available)
- Entities already created (descriptions can only be added to existing entities)

## Basic Usage

### Add Description with Inline Flag

Add a simple description to an existing entity:

```bash
wet entity edit rust --description "A systems programming language focused on safety and performance."
```

**Expected Output**:
```
Description updated for entity 'rust'
```

### Add Multi-Paragraph Description with Interactive Editor

Edit description in your preferred text editor:

```bash
wet entity edit rust
```

**What Happens**:
1. System launches your `$EDITOR` (or vim by default)
2. Editor opens with current description (if any)
3. Edit the description:
   ```
   Rust is a systems programming language that focuses on safety,
   speed, and concurrency. It achieves memory safety without using
   garbage collection.

   Rust is popular for building command-line tools, web services,
   and embedded systems. See [systems-programming] for context.
   ```
4. Save and exit editor
5. Description is saved to the entity

**Expected Output**:
```
Description updated for entity 'rust'
```

### Add Description from File

Prepare a description in a file:

```bash
cat > rust_description.txt << 'EOF'
Rust is a systems programming language that focuses on safety,
speed, and concurrency. It achieves memory safety without using
garbage collection.

Rust is popular for building command-line tools, web services,
and embedded systems. See @systems-programming for context.

Key features include:
- Zero-cost abstractions
- Ownership system
- Strong type system
- Excellent tooling (cargo, clippy, rustfmt)
EOF
```

Add description from file:

```bash
wet entity edit rust --description-file rust_description.txt
```

**Expected Output**:
```
Description updated for entity 'rust'
```

## Viewing Entities with Descriptions

### List All Entities

The `entities` command now shows description previews:

```bash
wet entities
```

**Expected Output** (80-character terminal):
```
rust - Rust is a systems programming language that focuses on safety, speed…
systems-programming - Low-level programming focused on operating systems…
wetware - A CLI tool for managing thoughts and entities. Built in Rust.
knowledge-management
```

**Notes**:
- Entities without descriptions (e.g., "knowledge-management") show no preview
- Previews show only the first paragraph
- Long descriptions are ellipsized to fit on one line
- Entity references in previews appear as plain text (no colors)

### Narrow Terminal Behavior

On narrow terminals (< 60 characters wide), previews are suppressed:

```bash
# Simulate narrow terminal
COLUMNS=50 wet entities
```

**Expected Output**:
```
rust
systems-programming
wetware
knowledge-management
```

## Entity References in Descriptions

### Plain Entity References

Use `[entity]` syntax to reference other entities:

```bash
wet entity edit rust --description "A systems language. See [programming] for context."
```

**Behavior**:
- If entity "programming" doesn't exist, it's automatically created
- Auto-created entities have no description
- Reference appears as plain text "programming" in previews

### Aliased Entity References

Use `[alias](entity)` syntax for custom display text:

```bash
wet entity edit rust --description "Popular for [CLI tools](command-line-tools) and web services."
```

**Behavior**:
- If entity "command-line-tools" doesn't exist, it's automatically created
- Preview shows "CLI tools" (the alias), not "command-line-tools"
- No color highlighting in preview

**Full Description**:
```
Popular for [CLI tools](command-line-tools) and web services.
```

**Preview Output**:
```
rust - Popular for CLI tools and web services.
```

## Removing Descriptions

### Clear Description with Empty String

Remove description by providing whitespace:

```bash
wet entity edit rust --description "   "
```

**Expected Output**:
```
Description removed for entity 'rust'
```

### Clear Description with Interactive Editor

1. Run `wet entity edit rust`
2. Delete all content in editor
3. Save and exit

**Expected Output**:
```
Description removed for entity 'rust'
```

## Advanced Examples

### Complex Multi-Paragraph Description

```bash
wet entity edit knowledge-management << 'EOF'
Knowledge management (KM) is the process of capturing, organizing,
and sharing information. Tools like [wetware] help individuals manage
their personal knowledge graphs.

Effective KM systems support entity references, making it easy to
link related concepts. For example, [Zettelkassel](zettelkasten)
is a popular KM methodology that uses index cards.

Modern digital KM tools often include:
- Bidirectional links
- Graph visualization
- Full-text search
- Markdown support
EOF
```

**Preview Output**:
```
knowledge-management - Knowledge management (KM) is the process of capturing…
```

**Entities Auto-Created**:
- `wetware` (if didn't exist)
- `zettelkasten` (if didn't exist)

### Description with Multiple Entity References

```bash
wet entity edit rust --description "Rust combines ideas from [functional-programming] and [systems-programming]. It's used by [Mozilla](mozilla) and [AWS](amazon-web-services) for critical infrastructure."
```

**Preview Output**:
```
rust - Rust combines ideas from functional-programming and systems…
```

**Entities Auto-Created**:
- `functional-programming`
- `systems-programming`
- `mozilla`
- `amazon-web-services`

## Edge Cases

### Whitespace-Only Description

```bash
wet entity edit rust --description "


"
```

**Behavior**: Treated as empty, description is removed

**Expected Output**:
```
Description removed for entity 'rust'
```

### Description Starting with Entity Reference

```bash
wet entity edit rust --description "[mozilla] created Rust as a research project. Now it's a community-driven language."
```

**Preview Output**:
```
rust - mozilla created Rust as a research project. Now it's a…
```

### Very Long Single-Paragraph Description

```bash
wet entity edit rust --description "Rust is a multi-paradigm systems programming language focused on safety, especially safe concurrency, supporting both functional and imperative programming styles with a syntax similar to C++ but providing memory safety without garbage collection and guaranteeing thread safety through its ownership system that enforces memory safety at compile time."
```

**Preview Output** (80-character terminal):
```
rust - Rust is a multi-paradigm systems programming language focused on…
```

## Troubleshooting

### Editor Doesn't Launch

**Problem**: `wet entity edit rust` fails with "Editor launch failed"

**Solutions**:
1. Set `$EDITOR` environment variable:
   ```bash
   export EDITOR=nano
   wet entity edit rust
   ```

2. Install vim or nano:
   ```bash
   # Debian/Ubuntu
   sudo apt install vim

   # macOS
   brew install vim
   ```

### Entity Doesn't Exist

**Problem**: `Error: Entity 'foo' not found`

**Solution**: Descriptions can only be added to existing entities. Create entity first by referencing it in a thought:

```bash
wet add "Learning about [foo] today"
wet entity edit foo --description "Description here"
```

### File Not Found

**Problem**: `Error: Description file 'desc.txt' not found`

**Solution**: Ensure file path is correct:

```bash
# Absolute path
wet entity edit rust --description-file /home/user/desc.txt

# Relative path
wet entity edit rust --description-file ./desc.txt
```

### Description Too Long for Preview

**Not a problem**: This is expected behavior. The `entities` command shows only a preview. Full descriptions can be viewed with `wet entity show <name>` (future feature).

## Tips

1. **Use Interactive Editor for Long Descriptions**: The `--description` flag works well for short text, but interactive editor is better for multi-paragraph content

2. **Leverage Entity References**: Use entity references to link related concepts, building your knowledge graph naturally

3. **First Paragraph Matters**: Since previews show only the first paragraph, put the most important information there

4. **Check Terminal Width**: If previews look truncated, check your terminal width (resize window for better display)

5. **Consistent Entity Naming**: Entity references are case-insensitive, but use consistent capitalization for readability

## What's Next?

After adding descriptions to entities:

1. **View Your Knowledge Graph**: Use `wet entities` to see all entities with previews
2. **Link Thoughts to Entities**: Use `wet add "text with [entity]"` to create connections
3. **Build Context**: Add descriptions to frequently-referenced entities
4. **Navigate Relationships**: Entity references in descriptions create implicit relationships

For more details, see:
- [Data Model Documentation](./data-model.md)
- [Implementation Plan](./plan.md)
- [Feature Specification](./spec.md)
