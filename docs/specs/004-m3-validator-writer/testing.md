# Testing

## Drift This Must Prevent

- New Rust candidate output accidentally uses or accepts legacy `word_index`.
- Viewer only supports `word_index` and cannot render `word_id`.
- Planner and validator compute different source fingerprints.
- Writer overwrites or partially creates an accepted output file.
- Validation failures write anything under real `output/`.
- Active docs say M3 viewer compatibility is still pending after it lands.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Typed sentence schema | Invalid JSON or missing fields are accepted. | Rust unit tests around parse/required fields. | Proves candidate data is structured before validation. |
| Register enum | Model output invents labels. | Unit tests for accepted/rejected register values. | Keeps learner-facing register values stable. |
| Token/word alignment | Tokens point nowhere or words are unused. | Unit tests for missing, unknown, duplicate, and unused IDs. | Prevents broken word-by-word display. |
| Source lineage | Candidate writes stale or extra source rows. | Unit tests for exact source coverage and fingerprint mismatch. | Keeps output tied to current YAML. |
| Romanisation reconstruction | Token roman forms drift from sentence romanisation. | Unit tests with punctuation and mismatch cases. | Keeps highlighting/alignment deterministic. |
| Accepted writer | Partial or overwritten output. | Tempdir writer tests for success, collision, and failure. | Protects accepted learner data. |
| Viewer compatibility | Rust output does not render. | Viewer build/fixture test or focused component utility test. | Keeps preview/export path compatible. |

## Unit Tests

- Valid one-sentence batch passes.
- Missing `literal`, blank `romanisation`, and missing `source_ref` fail.
- Invalid `register` fails.
- `kind: "space"` and `kind: "punct"` fail for tokens and words.
- Missing `word_id`, unknown `word_id`, duplicate `words[].id`, and unused
  words fail.
- Duplicate candidate `source_ref.file + item_id` fails.
- Candidate missing a planned source row fails.
- Candidate with an extra source row fails.
- Candidate with stale `source_ref.fingerprint` fails.
- Reconstruction passes with romanisation punctuation and fails on token roman
  mismatch.
- Writer refuses existing target and leaves the original file unchanged.
- Writer success writes valid JSON through the accepted writer API.

## Integration Tests

- Validator + writer happy path using a temp target path.
- Validator failure path proves no target is written.
- Viewer compatibility test covers:
  - Rust token: `{ "word_id": "w1" }`
  - Legacy token: `{ "word_index": 0 }`

## Drift Checks

- `git diff --name-only -- input output audio runs`
- `rg -n "word_index" src` should only show tests or explicit legacy comments
  if any; production validator should not accept it for candidates.
- `rg -n "word_id support.*pending|viewer compatibility.*pending" docs README.md`
  should not find active-doc claims after M3.

## Manual Review Checks

- Confirm no normal CLI command writes accepted output in M3.
- Confirm writer tests write only under temp directories.
- Confirm viewer fallback keeps existing Python output rendering.
- Confirm docs still point to M4 for real generation.

## Not Covered

- No Ollama/model smoke test; M4 owns model calls.
- No run-report validation; M4 owns generation run reports.
- No Anki export test; M6 owns export parity.
