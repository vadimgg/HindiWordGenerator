# Ollama Generation Acceptance

## Scope

Make `hindi sentences generate` usable for the first real local-model test run.
This spec fixes two issues found while testing: Ollama output can include
punctuation in `tokens[]`, causing validation to reject the whole batch, and the
CLI gives no progress signal while a local model call is running.

## Problem

The generation path now reaches Ollama and safely refuses invalid output, but a
validating refusal is still a workflow blocker if the model predictably returns
punctuation tokens. The user also sees a quiet terminal for minutes while
Ollama works, which makes it hard to tell whether generation is alive or stuck.

## Goals

- Keep `tokens[]` word-only before validation and accepted writes.
- Preserve all-or-nothing batch safety: invalid output still writes no accepted
  cards.
- Print clear progress/timing lines for plan, model readiness, model request,
  validation, writes, and run report.
- Rerun one real `translategemma:12b` batch and inspect whether validation can
  pass or whether remaining issues are prompt/model quality issues.

## Non-Goals

- Do not change source lineage policy for Python-era output.
- Do not introduce model switching or model lifecycle management.
- Do not loosen the validator to allow punctuation tokens.
- Do not solve structured validation error reports; that remains BL004.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | Model output normalization removes punctuation/space tokens from `tokens[]` without weakening the validator. |
| AC02 | Unit tests cover punctuation-token cleanup and ensure real word tokens remain linked to `words[]`. |
| AC03 | `hindi sentences generate --max-batches 1` prints progress/timing lines during long Ollama work. |
| AC04 | Failed validation still writes no accepted `output/sentences/*batch_*.json` file. |
| AC05 | One live Ollama run is attempted and the outcome is recorded in the final handoff. |
| AC06 | BL001 and BL002 are marked done or explicitly left open with the reason. |

## Architecture Notes

### Files And Folders Changed

- `src/sentence_enrichment.rs`
- `src/sentence_generate.rs`
- `docs/backlog/backlog.jsonl`

### Workflow State Touched

- Diagnostic `runs/sentences/*.json` may be created during smoke testing.
- Accepted `output/sentences/` must remain unchanged unless validation passes.

### External Effects And Reuse

- Uses local Ollama at `127.0.0.1:11434`.
- Reuses the existing enrichment merge path and sentence validator.
- Uses `brief task done` and backlog commands to record completion.

## Testing Plan

### Unit Tests

- Add focused tests in the enrichment/normalization path for punctuation tokens.
- Keep existing validator tests unchanged so punctuation remains invalid if it
  reaches validation.

### Integration Tests

- Run `make check`.
- Run `cargo run -- sentences generate --max-batches 1` against the current
  Ollama model.

### Smoke Tests

- `cargo run -- doctor`
- `cargo run -- source ids check`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- sentences generate --max-batches 1`

### Drift / Consistency Checks

- Confirm no accepted output file is written on failed validation.
- Confirm `runs/` remains ignored.

### Not Covered In This Spec

- Source QA, legacy-output lineage migration, and structured validation error
  reporting stay in backlog.

## Open Questions

None. The first implementation can use deterministic cleanup before validation
and keep stricter prompt tuning as a follow-up if needed.
