# Documentation

This directory is wetware's repo-local knowledge base for humans and AI agents. It exists because
reliable changes depend on finding the right context quickly — without it, contributors have to
reconstruct behavior, invariants, and rationale from source code and commit history every time.

Source code remains the implementation source of truth. These docs explain the system at a useful level
of abstraction: what each part does, how major workflows operate, what must remain true, and where the
key source files are.

## Map

| Directory | Answers |
|---|---|
| [`systems/`](systems/) | How does a given module work? What does it own? |
| [`flows/`](flows/) | How does an important cross-system workflow behave end to end? |
| [`architecture/`](architecture/) | How is the application shaped, and why? |
| [`architecture/decisions/`](architecture/decisions/) | Why was a specific technical decision made? |
| [`glossary.md`](glossary.md) | What does a domain-specific term mean? |
| [`templates/`](templates/) | What structure should a new system/flow/ADR doc follow? |

## How to use these docs

Start at the repo root [`AGENTS.md`](../AGENTS.md) for the full agent workflow. In short: read the
relevant system/flow docs before changing behavior, inspect the source for implementation detail, make
the change, then update whichever docs the change affects.

See [`STYLE.md`](STYLE.md) for how these docs are written and kept accurate.
