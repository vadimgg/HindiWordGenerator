# Plan

## Design

Build the planner as a read-only Rust domain that loads sentence sources,
indexes accepted output lineage, derives plan rows, and returns a typed report
for CLI rendering. Reuse or extract source YAML parsing from M1.5 rather than
duplicating source ID validation logic.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `sentences plan --max-batches <n>`. |
| `src/main.rs` | Dispatch and print planner reports. |
| `src/source_ids.rs` or extracted source module | Source YAML loading and source ID validation. |
| `src/sentence_plan.rs` | Accepted output reading, lineage classification, pending/deferred derivation, target filenames. |

## Operation Order

1. Parse `--max-batches` as a positive integer.
2. Discover project root.
3. Load and validate `input/sentences/*.yaml`.
4. Compute source fingerprints.
5. Load `output/sentences/*.json`.
6. Classify accepted output as done, missing lineage, source changed, or
   unrelated/stale.
7. Derive pending source rows.
8. Apply `--max-batches` by output-file count.
9. Choose next unused batch filenames.
10. Print a read-only report.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Review planner contract, output shape, and source/output edge cases. |
| WP02 | Implement planner domain and tests. |
| WP03 | Wire CLI/help/doctor text and run smoke checks. |
| WP04 | Review read-only safety, docs alignment, and protected path diff. |

## Risks

| Risk | Mitigation |
|---|---|
| Missing lineage is hidden. | Add explicit classification and fixture. |
| Planner writes files accidentally. | Keep domain read-only and run protected diff check. |
| Source parsing duplicates M1.5 rules. | Reuse/extract source ID parsing. |
| JSON parser becomes too ad hoc. | Keep extraction narrow but tested; full validation remains M3. |

## Validation

- `cargo fmt`
- `cargo test`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- source ids check`
- `python3 archive/python/scripts/check-agent-workflows.py`
- `uv run python archive/python/scripts/check-python-contracts.py`
- `git diff --name-only -- input output audio runs`
- `git diff --check`
