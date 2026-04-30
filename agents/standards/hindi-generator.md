# Hindi Generator Standard

Project-specific standards for agents working in this repository.

## Core Principles

- Treat generated JSON as learner-facing data, not disposable build output.
- Run `uv run main.py check` before token-spending generation work.
- Keep normal processing append-only. Do not delete, renumber, or rewrite old
  batches unless the task explicitly asks for migration or repair.
- Existing output JSON is the source of truth for completed cards.
- `manifest.json` is metadata; it must not be the only basis for deciding what
  has already been generated.
- Prefer small bounded runs with `--max-items` or `--max-batches` before a full
  pipeline run.

## Ownership Boundaries

- `main.py` owns CLI orchestration, check rendering, and operator ergonomics.
- `generate.py` owns model selection, prompt loading, parallel generation,
  retries, token reporting, write delegation, and fail-fast orchestration.
- `process.py` owns CSV parsing, batch planning, dedupe, schema validation,
  output paths, output writes, and manifest updates.
- `audio_generator.py` owns MP3 generation, deterministic audio filenames, and
  relative `audio` paths.
- `generation_prompt_words.txt` and `generation_prompt_sentences.txt` own model
  behavior and card quality guidance.
- `review_prompt_words.txt` and `review_prompt_sentences.txt` own QA reviewer
  behavior.
- `output/` and `audio/` are generated learner data; edit them directly only for
  one-off corrections, schema migrations, or requested backfills.

## Fix Routing

- Repeated content-quality issue: update the relevant generation prompt, then
  test on a small slice.
- One-off generated-card mistake: edit the output JSON directly.
- Missing or unsafe structural guarantee: update `process.py` validation.
- Confusing or risky operator flow: update `main.py`.
- Model, retry, concurrency, token, or fail-fast issue: update `generate.py`.
- Audio path, naming, or synthesis issue: update `audio_generator.py`.
- Delhi naturalness concern: use `language-teacher-reviewer.md` before changing
  prompts broadly.
- Downstream UI compatibility concern: prefer stricter validation over relying
  on prompt wording alone.

## Prompt And Schema Drift

- Prompt schema and `process.py` validation must agree.
- README and AGENTS docs must mention new required fields or workflow changes.
- If existing output predates the current schema, `check` should report the gap
  instead of silently treating it as fine.
- Do not re-run generation just to fix one bad card unless the user asks.

## Output Safety

- Never write partial batches after validation failure.
- Batch numbers must stay contiguous per stem.
- New runs must continue from the highest existing batch number.
- Audio paths must be relative and stable.
- Optional fields should be omitted when empty, not written as `null`, empty
  strings, or empty arrays.

## Review Questions

- Is this a code issue, prompt issue, data issue, or review issue?
- Does the change preserve append-only behavior?
- Does `check` make the next run understandable before tokens are spent?
- Would the downstream study/export app still understand the output?
- Are Delhi-naturalness and learner usefulness preserved?

