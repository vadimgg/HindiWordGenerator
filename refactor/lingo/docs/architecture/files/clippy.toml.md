# `clippy.toml`

> **Target kind:** Lint configuration  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../ARCHITECTURE.md)

## Responsibility

Holds project-level Clippy thresholds that cannot be expressed cleanly in the workspace manifest.

## Scope: this file owns

- size thresholds and project-wide Clippy tuning

## Out of scope: this file must not own

- allowing architectural violations
- suppressing warnings per crate without explanation

## Allowed dependencies

- Clippy

## Forbidden dependencies and shortcuts

- blanket allowlists
- thresholds used as substitutes for responsibility boundaries

## Key implementation shape

```text
too-many-arguments-threshold = 6
type-complexity-threshold = 250
```

## Required tests / evidence

- Clippy configuration is exercised by the normal workspace lint command

## Design notes

- A threshold is a warning signal, not permission to create a god service or god type.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
