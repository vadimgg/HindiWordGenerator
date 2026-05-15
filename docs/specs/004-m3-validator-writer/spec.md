# M3 Validator And Writer

## Scope

Build the sentence validation and accepted-output write layer that M4 generation
will use before any local model output can become learner data. This spec adds
typed sentence-batch structures, deterministic validation rules, an atomic
writer for accepted sentence JSON, and viewer compatibility for new `word_id`
links while keeping production generation out of scope.

## Problem

M2 can plan the next sentence batch, but nothing in Rust can yet decide whether a
candidate batch is safe to accept. Without M3, M4 would have to mix model calls,
schema checks, source lineage validation, collision checks, and file writes in
one risky step. We need the gate first: validate everything in memory, refuse bad
or partial batches, then write accepted output only through a safe temp-file
rename.

## Goals

- Define typed Rust structs for the accepted sentence batch shape in
  `docs/DESIGN.md`.
- Validate required sentence fields before any write.
- Enforce register labels: `informal`, `standard`, `formal`.
- Enforce word-only `tokens[]` and `words[]`; no spaces or punctuation entries.
- Enforce `token.word_id -> words[].id` alignment and reject unused `words[]`
  entries.
- Enforce `source_ref.file + item_id + fingerprint` against the source rows
  selected by the planner.
- Enforce romanisation reconstruction from `tokens[].roman` against the sentence
  `romanisation`.
- Refuse accepted-output collisions.
- Write accepted JSON through a temp file followed by rename.
- Add viewer support for Rust `word_id` while preserving legacy Python
  `word_index` rendering.

## Non-Goals

- No Ollama/model calls.
- No `hindi sentences generate` command yet.
- No production write to `output/sentences/` from a user-facing command.
- No review/accept run workflow.
- No source QA or source repair.
- No word-card generation.
- No migration/backfill of `source_ref` into Python-era output.
- No full Hindi romanisation engine; validation should enforce the mechanical
  reconstruction contract and high-signal schema rules only.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | Sentence batch structs model `title`, `subtitle`, `sentences[]`, `tokens[]`, `words[]`, `source_ref`, and optional `anki_tags`. |
| AC02 | Candidate JSON can be parsed into typed Rust structs with helpful errors on invalid JSON. |
| AC03 | Required sentence fields reject missing or blank `hindi`, `romanisation`, `english`, `literal`, and `register`. |
| AC04 | Register validation accepts only `informal`, `standard`, and `formal`. |
| AC05 | `tokens[]` and `words[]` reject entries whose `kind` is not `word`; punctuation and spaces are invalid in new Rust output. |
| AC06 | Every token must contain `word_id`; `word_index` is rejected for new Rust candidates. |
| AC07 | Every token `word_id` must reference exactly one `words[].id`. |
| AC08 | Every `words[]` entry must be referenced by at least one token. |
| AC09 | Duplicate `words[].id` values are rejected. |
| AC10 | `source_ref.file + source_ref.item_id` must match an expected source row for the candidate batch. |
| AC11 | `source_ref.fingerprint` must match the current fingerprint for that source row. |
| AC12 | Candidate batches must cover exactly the planned source IDs for the batch: no missing source rows, extra source rows, or duplicate source rows. |
| AC13 | Romanisation reconstruction compares against the `romanisation` string after NFC normalization, using spacing and punctuation from the romanisation string. |
| AC14 | Validation returns all useful errors for a batch rather than stopping at the first sentence when practical. |
| AC15 | Accepted writer refuses to write if the target output file already exists. |
| AC16 | Accepted writer writes through a temp file in the target directory and renames only after serialization succeeds. |
| AC17 | Writer tests prove failed validation and collisions leave no accepted output file. |
| AC18 | `hindi sentences plan` remains read-only and unchanged except for shared code reuse if needed. |
| AC19 | Viewer renders new Rust output using `token.word_id` and falls back to legacy `word_index` for Python-era output. |
| AC20 | Active docs no longer imply `word_id` viewer compatibility is pending once this spec lands. |

## Architecture Notes

[architecture.md](architecture.md) owns module ownership and write ordering. The
important boundary: validation/writer code may be callable from tests and future
M4 generation, but no normal user-facing command should write accepted sentence
output in this spec.

### Files And Folders Changed

- `Cargo.toml`
- `Cargo.lock`
- `src/main.rs`
- `src/sentence_plan.rs`
- `src/sentence_schema.rs` or equivalent typed schema module
- `src/sentence_validate.rs` or equivalent validator module
- `src/accepted_writer.rs` or equivalent writer module
- `viewer/**` files needed for `word_id` compatibility
- `docs/ROADMAP.md`
- `docs/specs/004-m3-validator-writer/**`

### Workflow State Touched

- Brief spec/task files under `docs/specs/004-m3-validator-writer/**`.
- No source YAML, accepted output, audio, or run folders should be modified by
  implementation or validation.

### External Effects And Reuse

- May add Rust dependencies for JSON parsing/serialization and Unicode
  normalization.
- Reuse M2 planner concepts for expected source rows, target filenames, and
  source fingerprints.
- Tests may write to temp directories only.
- No network calls and no Ollama calls.

## Testing Plan

### Unit Tests

- Valid batch passes.
- Missing required fields fail.
- Invalid register fails.
- Space/punctuation token entries fail.
- Missing `word_id`, unknown `word_id`, duplicate word IDs, and unused words
  fail.
- Source-ref missing, unknown, duplicate, and stale fingerprint cases fail.
- Romanisation reconstruction accepts punctuation/spacing from `romanisation`
  and rejects mismatches.
- Writer refuses collisions and does not leave accepted files on failure.

### Integration Tests

- End-to-end validator + writer happy path into a temp `output/sentences`
  directory.
- Collision path proves existing accepted output is preserved.
- Viewer fixture or component test proves `word_id` output renders and
  `word_index` output still renders.

### Smoke Tests

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `npm run build` inside `viewer/` if viewer dependencies are available.

### Drift / Consistency Checks

- `git diff --name-only -- input output audio runs` prints nothing.
- Search active docs for stale claims that `word_id` viewer support is still
  pending.
- Search Rust candidate-schema code for accidental `word_index` acceptance in
  new output.

### Not Covered In This Spec

- Model response extraction and generation run reports are M4.
- Full romanisation linguistic correctness is deferred; this spec only enforces
  structural reconstruction and schema safety.

## Open Questions

None. M3 is infrastructure-only: typed validation, safe writes, and viewer
compatibility before real model generation.
