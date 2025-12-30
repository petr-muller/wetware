<!--
Sync Impact Report:
- Version change: [INITIAL] → 1.0.0
- Initial ratification of Wetware Constitution
- Principles established:
  1. Test-First Development (90%+ Coverage)
  2. Layer Separation
  3. Strong Typing & Error Handling
  4. Observability & Documentation
  5. Simplicity & YAGNI
- Templates status:
  ✅ .specify/templates/plan-template.md - UPDATED with constitution checklist
  ✅ .specify/templates/spec-template.md - reviewed, aligns with constitution (no changes needed)
  ✅ .specify/templates/tasks-template.md - UPDATED:
     - Tests now REQUIRED (was optional)
     - Added 90%+ coverage verification tasks
     - Changed file extensions .py → .rs (Rust)
     - Added Result/Option type validation tasks
     - Added logging tasks per Observability principle
  ⚠ No command files exist yet in .specify/templates/commands/
  ⚠ No README.md exists - should be created referencing these principles
- Follow-up TODOs: None
-->

# Wetware Constitution

## Core Principles

### I. Test-First Development (90%+ Coverage)

**Requirements:**
- Target 90%+ test coverage across all modules
- Write tests before implementation when adding new features (TDD recommended)
- Tests MUST fail before implementation begins
- Red-Green-Refactor cycle for new functionality

**Testing Hierarchy:**
- Unit tests: Individual function/struct behavior
- Integration tests: Module interaction and persistence layer
- Contract tests: CLI interface contracts

**Rationale:** High test coverage ensures reliability and confidence in refactoring.
The codebase maintains quality through comprehensive automated testing.

### II. Layer Separation

**Architecture Layers:**
- **CLI Layer**: Command-line interface using clap, handles user interaction
- **Domain Model**: Core business logic (Thought, Entity types), framework-agnostic
- **Persistence Layer**: SQLite storage, isolated behind clear interfaces
- **Input Handling**: User input parsing and validation

**Rules:**
- Layers MUST have clear boundaries with minimal coupling
- Domain model MUST NOT depend on CLI or persistence implementation details
- Persistence layer MUST be swappable (interface-based design)
- CLI MUST delegate business logic to domain layer

**Rationale:** Layer separation enables independent testing, maintenance, and future
technology changes without cascading rewrites.

### III. Strong Typing & Error Handling

**Requirements:**
- Use Rust's type system to prevent invalid states at compile time
- Prefer `Result<T, E>` and `Option<T>` over panics or exceptions
- Public APIs MUST return meaningful error types
- Use domain-specific error types, not generic strings

**Error Handling:**
- Errors MUST be propagated with context (`?` operator, `context()`, `with_context()`)
- User-facing errors MUST be clear and actionable
- Internal errors MUST preserve debugging information

**Rationale:** Strong typing catches bugs at compile time. Explicit error handling
makes failure cases visible and forces developers to handle them appropriately.

### IV. Observability & Documentation

**Documentation Requirements:**
- Public APIs MUST have rustdoc comments
- Complex algorithms MUST have implementation comments
- Modules MUST have purpose documentation
- Architecture decisions MUST be documented (ADRs or inline rationale)

**Observability:**
- Log significant operations (persistence, CLI commands)
- Include structured context in logs
- Errors MUST be logged with full context before propagation

**Rationale:** Good documentation reduces onboarding time and maintenance costs.
Logging enables debugging production issues and understanding system behavior.

### V. Simplicity & YAGNI

**Principles:**
- Start simple, add complexity only when needed
- Avoid premature abstraction or optimization
- Keep functions small and focused (< 50 lines preferred)
- Prefer clear code over clever code
- No unused code or dead features

**Code Review Questions:**
- Is this the simplest solution that works?
- Are we solving a current problem or a hypothetical future one?
- Can this be broken into smaller, simpler pieces?

**Rationale:** Simple code is easier to understand, test, and maintain. YAGNI
prevents over-engineering and keeps the codebase focused on actual requirements.

## Development Workflow

**Conventional Commits:**
- Use conventional commit format: `type(scope): description`
- Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`, `style`
- Scope: module or component affected (e.g., `cli`, `storage`, `model`)

**Code Quality Gates:**
- All tests MUST pass (`cargo nextest run`)
- No clippy warnings (`cargo clippy`)
- Code MUST be formatted (`cargo fmt`)
- Test coverage MUST meet 90% threshold

**Pull Request Requirements:**
- Description of changes and rationale
- Tests included for new functionality
- Documentation updated if public API changed
- Constitution compliance verified

## Technology Standards

**Rust Edition:** 2024

**Core Dependencies:**
- CLI: `clap` (argument parsing)
- Persistence: `rusqlite` (SQLite integration)
- Testing: `nextest` (test runner)

**Development Tools:**
- Linting: `cargo clippy`
- Formatting: `cargo fmt`
- Build: `cargo build`

## Governance

**Constitution Authority:**
- This constitution supersedes undocumented practices
- All PRs MUST verify compliance with core principles
- Reviewers MUST challenge violations or request justification

**Amendment Procedure:**
- Amendments require documented rationale
- Version increments follow semantic versioning:
  - MAJOR: Backward-incompatible principle removals/redefinitions
  - MINOR: New principles or materially expanded guidance
  - PATCH: Clarifications, wording fixes, non-semantic refinements
- Amendments MUST update dependent templates and documentation

**Complexity Justification:**
- Violations of Simplicity & YAGNI MUST be justified in code/PR
- Justifications MUST reference specific current needs, not hypothetical futures
- Unjustified complexity MUST be refactored

**Compliance Review:**
- Constitution compliance is checked during code review
- Persistent violations trigger architecture review
- Use this document as runtime development guidance

**Version**: 1.0.0 | **Ratified**: 2023-08-23 | **Last Amended**: 2025-12-30
