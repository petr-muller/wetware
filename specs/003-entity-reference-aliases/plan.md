# Implementation Plan: Entity Reference Aliases

**Branch**: `003-entity-reference-aliases` | **Date**: 2026-01-31 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-entity-reference-aliases/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Enable users to reference entities using natural language aliases with markdown-like syntax `[alias](entity)`. When entering a thought like "I learned about [robots](robotics)", the user sees only "robots" in colored highlight when viewing the thought, while the system internally links to the "robotics" entity. This maintains backward compatibility with traditional `[entity]` syntax while making thoughts more grammatically natural.

**Technical Approach**: Extend the existing regex-based entity parser to recognize both traditional `[entity]` and aliased `[alias](entity)` patterns. Modify the entity styler to render only the alias portion while maintaining the same colored highlight styling. Store both the display alias and the target entity reference for proper entity-based queries and filtering.

## Technical Context

**Language/Version**: Rust 2024 edition (matching Cargo.toml)
**Primary Dependencies**:
- CLI: `clap` 4.5.54 (argument parsing)
- Persistence: `rusqlite` 0.32.1 (SQLite integration)
- Parsing: `regex` 1.11 + `lazy_static` 1.5 (entity pattern matching)
- Styling: `owo-colors` 4 (colored terminal output)
- Errors: `thiserror` 2.0 (error type derivation)
- Dates: `chrono` 0.4.43 (timestamp handling)

**Storage**: SQLite database (thoughts table stores content as-is, entity extraction happens at runtime)
**Testing**: `cargo nextest` (test runner), `tempfile` 3.14 (dev dependency for test databases)
**Target Platform**: Linux CLI (binary: `wet`)
**Project Type**: Single project (src/ structure with models, services, storage, cli layers)
**Performance Goals**: Regex pattern matching must handle 10,000 character thoughts efficiently (current max)
**Constraints**:
- Must maintain backward compatibility with existing `[entity]` syntax
- No database migration required (thoughts stored as-is)
- Same colored highlight styling as traditional entity references

**Scale/Scope**:
- Single-user CLI tool
- Existing codebase: ~2000 LOC across 20+ Rust files
- Core modification scope: entity_parser.rs (pattern matching), entity_styler.rs (rendering)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Verify compliance with Wetware Constitution (`.specify/memory/constitution.md`):

**I. Test-First Development (90%+ Coverage)**
- [x] Test strategy defined (unit, integration, contract)
  - Unit tests: regex pattern matching, alias parsing, validation
  - Integration tests: end-to-end thought save/retrieve with aliased entities
  - Contract tests: CLI behavior with new syntax
- [x] 90%+ coverage target achievable for this feature
  - Following existing pattern: entity_parser.rs has 100% coverage, entity_styler.rs has comprehensive tests
- [x] TDD approach planned for new functionality
  - Tests written first for new regex patterns and rendering logic

**II. Layer Separation**
- [x] Changes respect CLI / Domain / Persistence / Input layer boundaries
  - CLI layer: no changes (uses existing entity parsing)
  - Domain layer: Thought model unchanged (content stored as-is)
  - Services layer: entity_parser.rs and entity_styler.rs modifications only
  - Persistence layer: no changes (no migration needed)
- [x] No direct dependencies from Domain to CLI or Persistence implementation
  - Thought model remains framework-agnostic
- [x] Persistence abstraction maintained (interface-based)
  - No storage changes required

**III. Strong Typing & Error Handling**
- [x] Error types identified for new functionality
  - Validation errors: empty alias, empty reference, invalid characters in alias
  - Entity resolution errors: non-existent entity reference
  - Malformed syntax errors: unclosed brackets/parentheses
- [x] Result/Option types used (no panics in business logic)
  - Validation returns Result<(), ThoughtError>
  - Pattern matching uses Option for captures
- [x] Error context propagation planned
  - Use existing ThoughtError enum, add new variants if needed

**IV. Observability & Documentation**
- [x] Public API documentation plan (rustdoc)
  - Document new pattern matching behavior in entity_parser.rs
  - Update examples in docstrings to show aliased syntax
  - Add rustdoc examples for both syntaxes
- [x] Logging strategy for significant operations
  - No new logging needed (parsing is pure function, no I/O)
- [x] Architecture decision rationale documented
  - Decision to store raw content (no DB migration) documented in plan
  - Rationale for regex-based approach vs. parser combinator

**V. Simplicity & YAGNI**
- [x] Solution is simplest that meets current requirements
  - Extends existing regex pattern rather than introducing new parser
  - Reuses existing color assignment and styling logic
  - No new data structures or abstractions
- [x] No premature optimization or abstraction
  - Single regex pattern handles both syntaxes
  - No caching or pre-compilation beyond existing lazy_static
- [x] Complexity justified if present (see Complexity Tracking below)
  - No violations

## Project Structure

### Documentation (this feature)

```text
specs/003-entity-reference-aliases/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── models/
│   ├── mod.rs
│   ├── entity.rs        # No changes
│   └── thought.rs       # No changes (content stored as-is)
├── services/
│   ├── mod.rs
│   ├── entity_parser.rs # MODIFIED: Add aliased entity pattern matching
│   ├── entity_styler.rs # MODIFIED: Render alias portion only
│   ├── color_mode.rs    # No changes
│   └── ...
├── storage/
│   └── ...              # No changes (no migration)
├── cli/
│   └── ...              # No changes (uses updated parser)
├── errors/
│   └── thought_error.rs # POSSIBLE: Add new error variants
└── ...

tests/
├── unit/                # New tests for aliased pattern matching
├── integration/         # New tests for end-to-end aliased entity flow
└── contract/            # CLI contract tests for new syntax
```

**Structure Decision**: Single project structure (Option 1) maintained. Changes are localized to two service files (entity_parser.rs, entity_styler.rs) with no architectural changes. This follows the existing pattern where entity handling is a pure service layer concern with no persistence changes needed.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations - all constitution principles met. The solution extends existing regex-based parsing without adding new layers, abstractions, or dependencies.

## Phase 0: Research & Investigation

**Research Areas:**

1. **Regex Pattern Design for Aliased Entities**
   - Objective: Design regex pattern that matches both `[entity]` and `[alias](entity)` without breaking existing functionality
   - Questions:
     - How to handle nested or escaped brackets in markdown-like syntax?
     - Should pattern matching be greedy or non-greedy?
     - How to extract both alias and reference from single capture?
   - Output: Recommended regex pattern with test cases

2. **Whitespace and Validation Best Practices**
   - Objective: Determine robust validation and normalization for alias/reference pairs
   - Questions:
     - How do other markdown parsers handle whitespace in link syntax?
     - What's the standard approach for rejecting invalid characters in structured text?
     - Should validation happen at parse time or thought creation time?
   - Output: Validation strategy and error messaging approach

3. **Backward Compatibility Testing Strategy**
   - Objective: Ensure existing thoughts with `[entity]` syntax continue to work
   - Questions:
     - How to structure tests to verify both old and new syntax?
     - What edge cases exist at the boundary (e.g., `[entity](` without closing)?
     - How to prevent regression in entity color consistency?
   - Output: Test suite structure and test data patterns

4. **Rust Regex Performance for Large Thoughts**
   - Objective: Verify regex approach scales to 10,000 character thoughts
   - Questions:
     - What's the performance characteristic of rust regex library for this pattern?
     - Is lazy_static sufficient or should we use once_cell?
     - Any security concerns with user-controlled regex input?
   - Output: Performance benchmarks and optimization recommendations

**Deliverable**: `research.md` with findings and decisions for each area

## Phase 1: Data Model & Contracts

### Data Model Changes

**Modified Structures:**

1. **EntityReference** (conceptual - not a stored struct currently, represented in parsing output)
   - Display text (alias)
   - Target entity name (reference)
   - Validation rules:
     - Both alias and reference must be non-empty after trimming
     - Alias must not contain `[`, `]`, `(`, `)` characters
     - Reference must match existing entity in database
     - Whitespace trimmed from both parts

**No Database Schema Changes:**
- Thoughts table unchanged (content stored as raw text with markup)
- Entity extraction remains runtime operation
- No migration required

### API Contracts

**Internal API Changes (Service Layer):**

1. **entity_parser::extract_entities(text: &str) -> Vec<String>**
   - Current: Returns entity names from `[entity]` syntax
   - New: Returns entity names from both `[entity]` and `[alias](entity)` syntax
   - Change: For aliased syntax, returns the reference (target entity), not the alias
   - Backward compatible: Existing `[entity]` continues to work

2. **entity_styler::render_content(content: &str) -> String**
   - Current: Replaces `[entity]` with styled entity name
   - New: Replaces `[alias](entity)` with styled alias text
   - Change: Pattern matching updated to recognize and extract alias portion
   - Backward compatible: Existing `[entity]` rendering unchanged

**New Internal Structures (if needed):**

```rust
// Potential struct to represent parsed entity reference
struct EntityMatch {
    display_text: String,  // What user sees (entity name or alias)
    target_entity: String, // What entity this links to
    span: (usize, usize),  // Position in original text
}
```

**CLI Contract (No Changes):**
- `wet add <thought>` - accepts both `[entity]` and `[alias](entity)` in thought text
- `wet thoughts` - displays thoughts with styled entities (now includes aliases)
- `wet entities` - no changes (lists entities by their names, not aliases)

### Design Artifacts

**Phase 1 Outputs:**

1. **data-model.md**:
   - EntityMatch structure definition
   - Validation rules for alias and reference
   - State diagram: unparsed text → matched pattern → validated reference → styled display

2. **contracts/service-api.md**:
   - Updated function signatures
   - Input/output examples for both syntaxes
   - Error conditions and handling

3. **quickstart.md**:
   - Example: "How to use entity aliases in thoughts"
   - Code snippets showing both traditional and aliased syntax
   - Testing examples for new functionality

**Deliverable**: data-model.md, contracts/, quickstart.md

## Phase 2: Agent Context Update

After Phase 1 design is complete, update the agent context file to include new technologies/approaches if any were introduced:

```bash
.specify/scripts/bash/update-agent-context.sh claude
```

**Expected Updates**: None - no new dependencies added, only extension of existing regex patterns

**Re-evaluate Constitution Check**: Verify that the design maintains all constitutional principles, especially:
- Layer separation (no new layer dependencies)
- Simplicity (no new abstractions beyond what's necessary)
- Test coverage (comprehensive tests for new patterns)

## Implementation Phases

**Not included in /speckit.plan output - this is created by /speckit.tasks:**

Phase 2 will generate tasks.md with:
- Task breakdown for regex pattern implementation
- Task breakdown for entity styler updates
- Test creation tasks (TDD approach)
- Integration testing tasks
- Documentation update tasks
- Backward compatibility verification tasks

## Notes

**Key Decisions:**

1. **No Database Migration**: Thoughts are stored with raw content (including markup). Entity extraction happens at runtime via regex parsing. This means:
   - No migration needed for existing data
   - Alias syntax works immediately for new and old thoughts
   - Entity-based queries still work (parser extracts target entity, not alias)

2. **Regex Extension vs. New Parser**: Chose to extend existing regex pattern rather than introduce parser combinator library because:
   - Existing approach already handles 100% of current use cases
   - Pattern is simple enough for regex (no complex nesting)
   - No performance concerns for 10k character limit
   - Maintains consistency with existing codebase

3. **Alias Display Without Storage**: Alias is displayed but not stored separately. This means:
   - Cannot query "show all thoughts where alias was 'robot'"
   - Can query "show all thoughts referencing entity 'robotics'"
   - Alias is purely presentational (improves readability, not searchability)
   - Acceptable trade-off per spec requirements (no alias-based querying requested)

4. **Validation at Parse Time**: Invalid syntax (empty alias, special characters) is detected when extracting entities, not when creating thought. This means:
   - Thought creation doesn't fail for syntax errors
   - Errors reported when displaying/querying thoughts
   - Consistent with current approach (malformed `[entity]` doesn't block thought creation)

**Risks:**

1. **Regex Complexity**: Pattern must handle both syntaxes without ambiguity
   - Mitigation: Comprehensive test suite with edge cases
   - Pattern ordering matters: try aliased pattern first, fall back to traditional

2. **Breaking Backward Compatibility**: Change to regex could affect existing entity extraction
   - Mitigation: Extensive regression tests with existing thought patterns
   - Integration tests with real thought data

3. **User Confusion**: Users might not understand when to use `[entity]` vs `[alias](entity)`
   - Mitigation: Clear documentation and examples in quickstart.md
   - Both syntaxes work equally well (no "wrong" choice)
