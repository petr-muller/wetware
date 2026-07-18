# AGENTS.md

wetware is a Rust CLI (`wet`) for tracking dated "thoughts" linked to named entities, backed by SQLite.
This file is an index and operating guide for AI agents — it tells you where to look and how to work.
Detailed system behavior lives in [`docs/`](docs/), not here. Build, lint, test, and coding-convention
instructions live in [`CLAUDE.md`](CLAUDE.md).

## Docs map

| Need to know... | Look in |
|---|---|
| Where do I start? | [`docs/README.md`](docs/README.md) |
| How does a specific module work? | [`docs/systems/`](docs/systems/) |
| How does an important cross-system workflow behave? | [`docs/flows/`](docs/flows/) |
| Why is the application shaped this way? | [`docs/architecture/README.md`](docs/architecture/README.md) |
| Why was a specific technical decision made? | [`docs/architecture/decisions/`](docs/architecture/decisions/) |
| What does a domain term mean? | [`docs/glossary.md`](docs/glossary.md) |
| What structure should a new doc follow? | [`docs/templates/`](docs/templates/) |

## Operating rules

- Read the relevant `docs/systems/` and `docs/flows/` docs before changing a system's behavior.
- When a change affects responsibilities, runtime behavior, interfaces, data, security, or invariants,
  update the affected docs in the same change — don't leave it for later.
- Docs and code must agree. If you find them disagreeing, fix one to match the other, or flag the
  mismatch explicitly rather than guessing which is correct.
- Don't dump detailed system behavior into this file — it belongs in `docs/systems/` or `docs/flows/`.
  This file stays an index.

## Agent workflow

1. Read this file.
2. Use [`docs/README.md`](docs/README.md) to find documentation relevant to the area you're changing.
3. Read the relevant system, flow, architecture, and glossary docs.
4. Inspect the actual source files for implementation detail.
5. Make the change.
6. Update any docs whose behavior, responsibilities, flows, invariants, assumptions, or interfaces
   changed.
7. Verify docs and code agree before finishing.
