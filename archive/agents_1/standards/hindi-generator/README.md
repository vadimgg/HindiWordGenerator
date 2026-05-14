# Hindi Generator Standard

Project-specific standards for active HindiWordGenerator work.

## Core Principles

- Treat generated JSON as learner-facing data, not disposable build output.
- Build the Rust sentence workflow first.
- Keep normal processing append-only. Do not delete, renumber, or rewrite old
  batches unless the task explicitly asks for migration or repair.
- Existing output JSON is the source of truth for completed cards.
- Keep model/run metadata outside accepted card JSON.
- Keep source lineage in accepted sentence JSON once Rust generation starts.
- Prefer small bounded local-model runs before full generation.

## Active Ownership Boundaries

- Rust CLI owns the future operator workflow.
- Rust planner should own source parsing, dedupe, and pending/skipped work.
- Rust schema validation should own accepted output contracts.
- Rust writer should own append-only writes.
- Rust provider boundary should own Ollama/local-model calls.
- Viewer reads `output/` and `audio/`; it does not own generated-card truth.
- Archived Python under `archive/python/` is reference material for parity.

## Prompt And Schema Drift

- Prompt schema and validator schema must agree.
- Sentence output requires `tokens` and `words`.
- `tokens` and `words` contain word entries only, never spaces or punctuation.
- README, AGENTS, active docs, prompts, schema, viewer, and export behavior should
  describe the same output contract.

## Output Safety

- Never write partial batches after validation failure.
- Batch numbers must stay stable per stem.
- New runs must continue from the highest existing batch number.
- Audio paths must be relative and stable.
- Optional fields should be omitted when empty, not written as `null`, empty
  strings, or empty arrays.

## Fix Routing

- Repeated content-quality issue: update the relevant generation prompt, then
  test on a small slice.
- One-off generated-card mistake: edit the output JSON only with explicit user
  approval.
- Missing structural guarantee: strengthen schema validation.
- Confusing or risky operator flow: improve CLI design and docs.
- Delhi naturalness concern: use `agents/packs/language-teacher-reviewer/AGENT.md`
  before changing prompts broadly.

## Review Questions

- Does the change preserve append-only behavior?
- Would the viewer/export app still understand the output?
- Are title/subtitle, romanisation, tokens, words, audio paths, and tags still
  aligned?
- Is archived Python being used only as reference unless explicitly in scope?
