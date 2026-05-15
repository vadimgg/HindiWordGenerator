# Review

## Summary

Implemented M3 as reusable safety infrastructure for future sentence
generation:

- typed sentence batch schema and JSON parse/serialize helpers;
- source fingerprint helper shared by planner and validator;
- pure validator for required fields, register labels, token/word alignment,
  source lineage, exact source coverage, and romanisation reconstruction;
- accepted-output writer that refuses collisions and writes via temp file then
  rename;
- viewer compatibility for new Rust `word_id` tokens with legacy `word_index`
  fallback.

No normal CLI command writes accepted sentence output in this spec.

## Validation

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `npm --prefix viewer run check:quality`
- `npm --prefix viewer run build`
- `git diff --name-only -- input output audio runs`
- `git diff --check`

Rust tests: 40 passed.

Viewer build passed.

Protected-path diff printed nothing.

## Changed Files

- `Cargo.toml`
- `Cargo.lock`
- `src/main.rs`
- `src/source_identity.rs`
- `src/sentence_plan.rs`
- `src/sentence_schema.rs`
- `src/sentence_validate.rs`
- `src/accepted_writer.rs`
- `viewer/src/utils/types.ts`
- `viewer/src/components/cards/sections/SentenceTokensSection.astro`
- `viewer/src/scripts/quality/sentenceTokens.js`
- `viewer/scripts/check-sentence-quality.js`
- `docs/ROADMAP.md`

## Follow-Ups

- M4 should call the validator and writer rather than reimplementing schema or
  write safety in the generation command.
- The module-level dead-code allowances on M3 infrastructure should be removed
  when M4 wires these internals into production generation.
