# M2 Sentence Planner

## Scope

Implement `hindi sentences plan --max-batches <n>` as a read-only Rust command.
It reads active sentence YAML and accepted sentence output, validates source
IDs, derives which source items are pending, and prints the next planned batch
filenames. It does not generate, validate candidate model output, or write
accepted output.

## Problem

Before generation can be safe, the user needs to see exactly what the Rust
pipeline thinks is done, pending, deferred, stale, or missing lineage. Without a
planner, M4 generation would be the first command to discover source/output
drift, which is too late.

## Goals

- Expose `hindi sentences plan --max-batches 1`.
- Read all active sentence YAML files under `input/sentences/`.
- Validate source item IDs and source row shape.
- Read existing accepted sentence JSON files under `output/sentences/`.
- Report Python-era output without `source_ref` as `missing lineage`.
- Report source rows that are done, pending, deferred, and changed.
- Plan the next unused `*_batch_XX.json` filename for each planned batch.
- Keep the command read-only.

## Non-Goals

- Do not implement `hindi sentences generate`.
- Do not call Ollama or any model.
- Do not write `output/`, `audio/`, or `runs/`.
- Do not backfill `source_ref` into Python-era output.
- Do not implement M3 validator/writer behavior.
- Do not implement word planning.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `cargo run -- sentences plan --max-batches 1` is accepted by the CLI. |
| AC02 | The command reads `input/sentences/*.yaml` and validates every item has a stable quoted `id`. |
| AC03 | Duplicate or malformed source IDs block planning and print the affected file/ID. |
| AC04 | The command reads `output/sentences/*.json` without modifying it. |
| AC05 | Accepted output entries with `source_ref.file + source_ref.item_id + fingerprint` matching current source count as done. |
| AC06 | Accepted output entries without `source_ref` count as `missing lineage`, not done. |
| AC07 | Accepted output entries whose `file + item_id` exists but fingerprint differs count as `source changed`. |
| AC08 | Source rows without matching current accepted output count as pending. |
| AC09 | `--max-batches` limits planned output files across the invocation, not source items. |
| AC10 | Planned output filenames use the next unused zero-padded batch number for the source stem. |
| AC11 | Planned rows beyond the max batch limit count as deferred. |
| AC12 | Planner output includes clear totals and at least one source-level breakdown row. |
| AC13 | Planner prints a next command for generation, but generation remains unavailable until M4. |
| AC14 | The command writes nothing to `input/`, `output/`, `audio/`, or `runs/`. |
| AC15 | Unit tests cover source fingerprinting, done/pending derivation, missing lineage, source changed, and batch filename selection. |

## Architecture Notes

[architecture.md](architecture.md) owns the detailed module boundaries. The
important trust boundary is that source YAML is authority for source rows, while
accepted output is authority for completed cards. The planner derives state
from both and writes nothing.

### Files And Folders Changed

- `src/cli.rs`
- `src/main.rs`
- New planner/source-output modules as needed, likely `src/sentence_plan.rs`
- `docs/ROADMAP.md`
- `docs/specs/003-m2-sentence-planner/**`

### Workflow State Touched

- Brief spec/task files for this spec.
- Roadmap status row for `Sentence planner` once implemented.

### External Effects And Reuse

- Filesystem reads from `input/sentences/*.yaml`.
- Filesystem reads from `output/sentences/*.json`.
- No writes outside spec/task status updates and docs.
- No network calls.
- No Ollama calls.

## Testing Plan

### Unit Tests

- Source fingerprint normalization.
- Source ID validation reuse or integration.
- Accepted output parsing for cards with and without `source_ref`.
- Done/pending/source-changed derivation.
- Next batch filename selection with existing batch files.
- `--max-batches` limiting and deferred counts.

### Integration Tests

- Fixture project with sentence YAML and accepted output JSON.
- Planner reports pending rows when output is absent.
- Planner reports missing lineage for Python-era output.
- Planner reports done for matching `source_ref`.
- Planner writes no data files.

### Smoke Tests

```bash
cargo fmt
cargo test
cargo run -- sentences plan --max-batches 1
git diff --name-only -- input output audio runs
git diff --check
```

### Drift / Consistency Checks

- Active docs should no longer say `hindi sentences plan` is unavailable after
  implementation.
- `hindi doctor` next-step text should point to the real planner command.

### Not Covered In This Spec

- Model generation.
- Candidate batch validation.
- Accepted output writes.
- Word planning.

## Open Questions

- None. The roadmap already defines `--max-batches` as total output files across
  the invocation.
