# Rust Standard

Rust standards for the planned HindiWordGenerator rewrite. Use this standard
for Rust CLI, local-model orchestration, parsing, validation, and migration
work.

Load this file first, then load focused rules when the task touches that area.

## Rules

| Rule | When to read |
|---|---|
| [Architecture Boundaries](rules/architecture-boundaries.md) | Adding modules, ownership boundaries, local model flow, or data surfaces. |
| [CLI Design](rules/cli-design.md) | Adding or changing commands, flags, output, errors, or help. |
| [Schema And Data Safety](rules/schema-and-data-safety.md) | Reading input, validating generated JSON, writing output, or touching manifests. |
| [Testing](rules/testing.md) | Changing Rust behavior or migration parity. |

## Architecture Boundaries

- Keep the Rust CLI user-facing and task-oriented: `doctor`,
  `sentences plan`, `sentences generate`, `sentences audio`, `viewer`,
  `export`, and future `deliver`.
- Separate parsing/planning, validation, model calls, persistence, and CLI
  rendering into distinct modules.
- Treat the current Python implementation as the reference until Rust reaches
  parity.
- Do not hide Python calls inside Rust as the final design. Temporary adapters
  are acceptable only when documented as migration scaffolding.
- Keep local-model calls behind a provider boundary so Ollama can be replaced or
  extended later.
- Keep viewer and export code consumers of `output/` and `audio/`; they must not
  become card authorities.

## File And Function Size

Size is a review trigger, not an automatic failure.

- Prefer modules under roughly 250 lines when practical.
- Prefer functions that fit on one comfortable screen.
- Split large command handlers into parse/plan/execute/render steps.
- Avoid broad `utils`, `helpers`, or `common` modules. Name modules by
  ownership: `planner`, `schema`, `ollama`, `writer`, `audio`, `report`.

## Types And Errors

- Model input rows, planned batches, generated payloads, validation errors, and
  write reports should have explicit structs.
- Use enums for pipeline type, card type, register labels, command modes, and
  provider selection.
- Do not pass untyped maps across module boundaries unless the boundary is raw
  JSON ingestion.
- Errors should include the path, stem, batch number, model, command, or field
  involved.
- User-facing errors should explain the next useful action.

## Data Safety

- Preserve append-only output behavior during normal generation.
- Refuse to overwrite existing output batch files unless the command name and
  flags make repair/regeneration explicit.
- Validate generated JSON before writing.
- Optional fields should be omitted when empty, not written as `null`, empty
  strings, or empty arrays.
- Sentence output must include `tokens` and `words`; tokens contain word entries
  only, not spaces or punctuation.
- Manifest metadata must not become the only authority for completed cards.
- Do not silently repair model output without reporting what changed.

## Testing And Validation

Behavior changes need focused validation.

- Parser and planner work should compare Rust output against known Python
  planner behavior.
- Schema validation should use small fixtures for valid and invalid word/sentence
  batches.
- CLI output changes should be checked with a command-level smoke test.
- Local model calls should have a tiny timeout-bounded smoke test and a dry-run
  mode that does not require Ollama.
- Before replacing a Python path, run the Rust command and the current Python
  command on the same small fixture and compare the important fields.

Good future validation commands:

```bash
cargo test
cargo run -- doctor
cargo run -- sentences plan --max-batches 1
cargo run -- models status
```
