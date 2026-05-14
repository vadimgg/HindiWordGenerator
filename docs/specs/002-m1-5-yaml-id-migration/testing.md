# Testing

## Drift This Must Prevent

- Source IDs are regenerated after being committed.
- Duplicate IDs pass validation and later confuse planner lineage.
- Migration writes accepted output while adding source IDs.
- Check mode mutates source YAML.
- Existing Hindi, romanisation, English, or tags change during ID insertion.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| ID allocation | Existing IDs get rewritten. | Unit test with existing IDs and gaps. | Proves stability. |
| Source validation | Duplicate/malformed IDs pass. | Unit tests for duplicate and malformed IDs. | Blocks bad source authority. |
| Dry run | Preview changes files. | Integration test compares fixture before/after. | Proves no-write mode. |
| Migration writes | Writes outside active YAML. | Smoke `git diff --name-only -- output audio runs archive/python/legacy-input`. | Protects accepted data. |
| YAML preservation | Existing fields change. | Fixture comparison for item order and fields. | Prevents accidental data edits. |

## Unit Tests

- `allocates_ids_for_missing_items_in_order`
- `preserves_existing_ids`
- `fills_missing_ids_without_reusing_existing_ids`
- `rejects_duplicate_ids_within_one_file`
- `rejects_malformed_ids`
- `migration_is_idempotent`

## Integration Tests

- Fixture project with one sentence YAML and one word YAML.
- `source ids check` reports missing IDs before migration.
- `source ids migrate --check` reports planned writes and changes no files.
- `source ids migrate` adds IDs.
- `source ids check` passes after migration.

## Drift Checks

```bash
cargo fmt
cargo test
cargo run -- source ids check
cargo run -- source ids migrate --check
cargo run -- source ids migrate
cargo run -- source ids check
git diff --check
git diff --name-only -- output audio runs archive/python/legacy-input
```

The final `git diff --name-only -- output audio runs archive/python/legacy-input`
must print nothing.

## Manual Review Checks

- Inspect at least one sentence YAML and one word YAML to confirm `id` appears
  before `hindi` for each item.
- Confirm IDs are quoted strings, not YAML numbers.
- Confirm active docs do not say M2 planner is available yet.
- Confirm the migration command output gives a clear next command.

## Not Covered

- Planner consumption of IDs is not tested here; M2 owns it.
- Accepted output lineage is not tested here; old output remains missing
  lineage by design.
