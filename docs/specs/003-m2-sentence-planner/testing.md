# Testing

## Drift This Must Prevent

- Missing-lineage output is counted as done.
- Source fingerprint mismatches are ignored.
- `--max-batches` limits source items instead of output files.
- Planned target filenames collide with existing output files.
- Planner writes data while previewing.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Source fingerprinting | Source edits not detected. | Unit test for normalized fingerprint. | Proves stable versioning. |
| Accepted output classification | Old output counted as done. | Fixture without `source_ref`. | Protects lineage rules. |
| Batch filename selection | Collision with existing batch. | Fixture with batches 01-04. | Proves next unused naming. |
| Max batches | Wrong planning unit. | Fixture with multiple pending batches. | Proves file-count semantics. |
| Read-only command | Data mutation. | Protected path diff check. | Preserves authority surfaces. |

## Unit Tests

- `computes_source_fingerprint_with_normalized_whitespace`
- `classifies_missing_lineage_output`
- `classifies_done_when_source_ref_matches`
- `classifies_source_changed_when_fingerprint_differs`
- `plans_next_unused_batch_filename`
- `max_batches_limits_output_files`

## Integration Tests

- Fixture project with sentence YAML and no output: all source rows pending.
- Fixture project with Python-era output: `missing lineage` increases and done
  remains zero.
- Fixture project with matching Rust-style `source_ref`: source row is done.
- Fixture project with changed fingerprint: source row is reported changed.

## Drift Checks

```bash
cargo fmt
cargo test
cargo run -- sentences plan --max-batches 1
git diff --name-only -- input output audio runs
git diff --check
```

The protected path diff must print nothing.

## Manual Review Checks

- Planner output has clear sections: Sources, Accepted Output, Plan, Planned
  Files, Next.
- Existing `output/sentences/*batch_01..04.json` causes the first Chapter 02
  planned filename to be `batch_05`.
- `hindi doctor` no longer says the planner is a future command once M2 lands.

## Not Covered

- Full accepted-output schema validation belongs to M3.
- Generation and output writes belong to M4.
