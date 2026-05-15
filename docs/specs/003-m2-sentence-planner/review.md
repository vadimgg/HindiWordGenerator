# Review

## Summary

Implemented M2 as a read-only sentence planner surfaced through:

```bash
cargo run -- sentences plan --max-batches 1
```

The planner reads `input/sentences/*.yaml` and `output/sentences/*.json`,
validates file-scoped source IDs, reports existing accepted cards without
`source_ref` as `missing lineage`, and plans the next unused batch filename
without writing learner data.

## Validation

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- source ids check`
- `git diff --name-only -- input output audio runs`
- `git diff --check`

Real project smoke output reported:

- 6 sentence source files
- 296 source items
- 296 valid IDs
- 4 existing output batch files
- 20 accepted cards
- 0 done cards
- 20 missing-lineage cards
- 1 planned batch
- 5 planned items
- 291 deferred items
- planned output file:
  `output/sentences/complete_hindi_chapter_02_sentences_batch_05.json`

The protected-path diff printed nothing.

## Changed Files

- `src/cli.rs`
- `src/main.rs`
- `src/doctor.rs`
- `src/sentence_plan.rs`
- `docs/ROADMAP.md`

## Follow-Ups

- M3 should replace the narrow planner-side JSON scanning with the real typed
  accepted-output reader/validator.
- M4 should reuse the planner target selection instead of re-deriving planned
  output filenames independently.
