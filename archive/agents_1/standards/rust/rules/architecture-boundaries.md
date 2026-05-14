# Rust Architecture Boundaries

Use this rule when adding Rust modules, command handlers, or migration adapters.
Start in one binary crate; extract workspace crates only after a real boundary
appears in the code.

## Required Shape

Prefer these owners:

- `cli`: argument parsing and command routing
- `doctor`: dependency and project health checks
- `models` or `ollama`: local model discovery and smoke tests
- `planner`: input parsing, dedupe, pending batch planning
- `schema`: output contract validation
- `writer`: append-only output writes and run reports
- `generator`: staged local-model orchestration
- `audio`: audio backfill command boundary
- `report`: user-facing summaries and tables

## Rules

- A command handler may orchestrate, but should not own parsing, validation,
  model HTTP calls, and writes all at once.
- Provider-specific code must stay behind a provider boundary.
- Generated learner data must only be written through the writer/validator path.
- Keep Python compatibility adapters visibly temporary and remove them once Rust
  owns the path.
