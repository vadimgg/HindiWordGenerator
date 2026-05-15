# Testing

## Drift This Must Prevent

- Generation uses a full-enrichment prompt while docs/eval say staged prompts
  are the default.
- Eval register/literal/word prompts improve but generation keeps stale copies.
- A stage omits an item and merge silently fills an empty/default field.
- Model output supplies trusted source fields or lineage.
- A validation failure still writes an accepted output file.
- Run reports omit which stage failed.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Stage prompt registry | Prompt IDs/version drift | Unit test | Confirms required generation stages exist with fingerprints. |
| Stage parsers | Bad model output accepted | Unit tests | Confirms YAML/JSON/fenced parsing and parse failures. |
| Staged merge | Wrong enrichment attached to source row | Unit tests | Confirms exact source ID coverage and Rust-owned source fields. |
| Generate orchestration | Partial output written | Integration tests with fake model | Confirms no accepted writes on stage/merge/validation failure. |
| Run report stages | Debugging data missing | Unit/integration test | Confirms per-stage metadata appears in success and failure reports. |
| Active docs | Old mental model survives | Drift greps | Confirms default generation is documented as staged. |

## Unit Tests

- Prompt registry includes `sentence/register`, `sentence/literal`, and
  `sentence/word-breakdown-from-translation`.
- Prompt fingerprints change when prompt text changes.
- Register parser accepts valid keyed output and rejects duplicate IDs.
- Literal parser accepts valid keyed output and rejects missing literals.
- Word-breakdown parser accepts valid words and rejects duplicate IDs.
- Staged merger:
  - succeeds for complete matching stage records
  - fails on missing stage item
  - fails on duplicate stage item
  - fails on extra stage item
  - copies source fields/source_ref from planner rows
  - never uses model-provided source fields
- Run report serialization includes `stages[]`.

## Integration Tests

- Fake model returns three successful stage responses; command writes one
  accepted batch and one accepted run report.
- Fake model fails register stage; command writes no accepted output and writes
  failed run report.
- Fake model returns invalid word breakdown; validation fails and command writes
  no accepted output.
- Planner error path still calls no model methods.
- Output collision still fails before accepted write.

## Drift Checks

- `rg "full-enrichment|generation_prompt_sentences_enrichment.txt|single-prompt" docs src`
- `rg "hindi sentences generate --max-batches" docs README.md src`
- `rg "source_ref|fingerprint" src/sentence_*`

## Manual Review Checks

- Verify command output names accepted output and run report paths.
- Verify run report identifies the slowest/failing stage.
- Verify Hindi display rule in any docs/CLI examples that show Hindi.
- Verify no generated eval artifacts are committed.

## Not Covered

- Large-scale model quality benchmarking is not covered here; use `hindi eval`
  for prompt/model comparison.
- Audio and viewer/export behavior are covered by later specs.
