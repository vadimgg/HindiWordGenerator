# Plan

## Design

Implement the validator as a reusable Rust boundary that future generation must
pass through. The code should parse candidate sentence batches into typed
structures, validate them against expected planner/source rows, and expose an
atomic writer that writes only already-validated batches. Viewer compatibility
for `word_id` should land in the same spec so the first Rust-generated card can
render before M4 writes real output.

## Modules

| Module | Responsibility |
|---|---|
| `src/sentence_schema.rs` | Typed accepted sentence-batch structures and JSON parse/serialize helpers. |
| `src/sentence_validate.rs` | Candidate validation rules: required fields, register enum, token/word alignment, source lineage, romanisation reconstruction. |
| `src/accepted_writer.rs` | Collision refusal and temp-file-then-rename writes for accepted output. |
| `src/sentence_plan.rs` | Source-row and planned-target data to share with validator/generation, possibly by extracting common source identity helpers. |
| `viewer/**` | Render `token.word_id` links while keeping legacy `word_index` support. |
| `docs/ROADMAP.md` | Mark M3 done and remove pending viewer-compatibility wording after implementation. |

## Operation Order

1. Add JSON/Unicode dependencies if needed.
2. Define sentence batch structs matching `docs/DESIGN.md`.
3. Extract or share source identity/fingerprint data needed by both planner and
   validator.
4. Implement validation in memory:
   - parse candidate JSON;
   - check required fields;
   - check register labels;
   - check word-only tokens and words;
   - check `word_id` links and unused words;
   - check exact source coverage and fingerprint matches;
   - check romanisation reconstruction.
5. Implement accepted writer:
   - validate target path parent exists;
   - fail if target exists;
   - serialize into memory;
   - write temp file in the target directory;
   - rename temp file to target;
   - clean temp file on failure when possible.
6. Update viewer token lookup to use `word_id` first and fallback to
   `word_index`.
7. Update docs/status and run validation.

Point of no return: the writer's rename from temp path to accepted target. All
validation, collision checks, and serialization must finish before that rename.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Review the M3 contract, current schema assumptions, and viewer lookup path before source edits. |
| WP02 | Implement typed sentence schema and validator with focused Rust tests. |
| WP03 | Implement accepted-output writer with collision and no-partial-write tests. |
| WP04 | Add viewer `word_id` compatibility and keep legacy `word_index` fallback. |
| WP05 | Review validation/writer safety, protected-path behavior, docs, and brief closeout notes. |

## Risks

| Risk | Mitigation |
|---|---|
| Validator accidentally accepts legacy `word_index` for new Rust output. | Keep legacy tolerance only in viewer tests; Rust validator requires `word_id`. |
| Writer creates partial accepted files. | Serialize before writing, write temp in target dir, rename only after success, test failure/collision paths. |
| Source fingerprint code drifts from planner. | Extract shared helper or keep one tested implementation used by planner and validator. |
| Viewer support ships after Rust output. | Make viewer compatibility WP04 and a blocking M3 acceptance item before M4. |
| Romanisation validation becomes a full linguistic engine. | Limit M3 to reconstruction against the romanisation string and structural token checks. |

## Validation

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `git diff --name-only -- input output audio runs`
- `git diff --check`
- `npm --prefix viewer run build` if dependencies are present.
