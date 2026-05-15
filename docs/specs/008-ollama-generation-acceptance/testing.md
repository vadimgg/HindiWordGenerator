# Testing

## Drift This Must Prevent

- Prompt/model output containing punctuation tokens should not reach accepted
  validation as token entries.
- Validation must still reject punctuation tokens if normalization is bypassed.
- Long local model calls must not look like a frozen CLI.
- Failed validation must not write accepted sentence batches.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Token cleanup | Non-word tokens break validation | Unit test in `sentence_enrichment` | Proves punctuation entries are removed before validation. |
| Validator strictness | Cleanup weakens the contract | Existing validator tests | Proves punctuation tokens are still invalid candidate data. |
| Progress output | CLI appears stuck | Live smoke run | Confirms phase lines print before and after slow model work. |
| No partial writes | Failed batch pollutes output | Live smoke run + file check | Confirms accepted output is only written on valid batches. |

## Unit Tests

- Add `sentence_enrichment` coverage for punctuation token removal.
- Keep validator tests for spaces/punctuation tokens.

## Integration Tests

- `make check`

## Drift Checks

- `find output/sentences -maxdepth 1 -name '*batch_05.json' -print`
- `git status --short`

## Manual Review Checks

- Review generated run report for status, timings, validation result, and skipped
  target.
- Inspect live generation output for useful progress lines.

## Not Covered

- Quality grading of accepted translations is not covered here; this spec only
  makes the first local-model acceptance loop operational.
