# Hindi Generator Standard

Standards for projects that turn curated Hindi learning material into
learner-facing flashcard data, audio, viewer previews, or exports.

## Core Principles

- Treat generated JSON as learner-facing data, not disposable build output.
- Keep human-curated `input/` separate from accepted `output/`.
- Preserve append-only generation unless the task explicitly asks for repair,
  migration, replacement test data, or cleanup.
- Keep model/run metadata outside accepted cards unless the schema is
  deliberately expanded.
- Prefer small bounded local-model runs before full generation.
- Whenever Hindi is shown to the learner or reviewer, show romanisation directly
  underneath or beside it.

## Ownership Boundaries

- Source files own Hindi, romanisation, English, title/subtitle, and tags.
- Generation models should enrich trusted source rows, not rewrite source
  fields or lineage.
- Validators own structural acceptance before data reaches `output/`.
- Writers own temp-file writes, collision refusal, and append-only safety.
- Audio tooling may backfill missing media and relative `audio` fields only.
- Viewers and exports read accepted output; they do not become card authority.
- Archived runtimes are reference material unless the task explicitly scopes
  them in.

## Prompt And Schema Drift

- Prompt output, validator schema, viewer types, and export fields must agree.
- Sentence output should keep word breakdown entries word-only; never model
  spaces or punctuation as token entries.
- Optional fields should be omitted when empty, not written as `null`, empty
  strings, or filler arrays.
- Romanisation policy belongs in the project docs. Prompt examples and output
  validation should follow the same policy.

## Output Safety

- Never write a partially valid accepted batch.
- Batch numbering must remain deterministic for each source stem.
- New generation should continue from existing accepted output, not from a
  manifest that can drift.
- Audio paths must be project-relative and stable.
- Repair or migration must name exactly which surfaces it may rewrite.

## Fix Routing

- Repeated content-quality issue: review or update the relevant prompt, then
  test on a small slice.
- One-off generated-card mistake: edit accepted output only with explicit user
  approval.
- Missing structural guarantee: strengthen validation before generating more.
- Confusing or risky operator flow: route to command/design review before
  implementation.
- Delhi naturalness, learner usefulness, or romanisation judgment: use the
  `hindi-language-teacher-reviewer` pack before broad prompt changes.

## Review Questions

- Does this preserve the source -> generated -> accepted-output boundary?
- Would the viewer/export app still understand the output?
- Are Hindi, romanisation, English, tokens, words, audio paths, and tags still
  aligned?
- Can the user recover if a model returns bad JSON or weak language content?
- Is any generated or cached surface being treated as stronger than accepted
  output?
