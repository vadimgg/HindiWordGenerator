# M1.5 YAML ID Migration

## Scope

Create the migration and validation path that adds stable `id` fields to active
source YAML files in `input/sentences/` and `input/words/`. This spec may add
small Rust source-loading helpers and a temporary Rust CLI surface for source ID
checks/migration, but it must not implement the M2 planner or modify accepted
output.

## Problem

The active YAML source files currently identify items only by their position and
content. The Rust planner needs a stable handle that survives source text fixes
and row insertion, so it can later build `source_ref.file + source_ref.item_id`
without guessing.

## Goals

- Add stable, file-scoped IDs to every active YAML source item.
- Provide a repeatable migration command that preserves existing IDs and source
  order.
- Provide a validation command/check mode that reports missing IDs, duplicate
  IDs, and malformed IDs.
- Keep all writes limited to `input/sentences/*.yaml` and `input/words/*.yaml`.
- Update active docs and spec notes to show M1.5 as complete once implemented.

## Non-Goals

- Do not implement `hindi sentences plan`.
- Do not write or modify `output/`, `audio/`, or `runs/`.
- Do not backfill `source_ref` into Python-era generated cards.
- Do not migrate archived CSV files again.
- Do not assign globally unique IDs or encode filenames, chapters, or titles in
  the ID.
- Do not build a general YAML editing framework beyond this migration.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | Every item in `input/sentences/*.yaml` has a quoted string `id` field. |
| AC02 | Every item in `input/words/*.yaml` has a quoted string `id` field. |
| AC03 | IDs are unique within each YAML file and use short zero-padded numeric strings such as `"0001"`. |
| AC04 | Existing IDs are preserved when migration is rerun. |
| AC05 | Items without IDs receive the next available zero-padded ID in file order. |
| AC06 | Existing item order and existing `hindi`, `romanisation`, `english`, and `tags` values are preserved. |
| AC07 | Duplicate IDs within one file block the migration and print the affected file and ID. |
| AC08 | Malformed IDs block validation and print a recovery hint. |
| AC09 | Check mode reports whether migration is needed without changing files. |
| AC10 | Normal migration writes only active YAML files under `input/sentences/` and `input/words/`. |
| AC11 | No files under `output/`, `audio/`, `archive/python/legacy-input/`, or `runs/` change. |
| AC12 | `cargo test` covers ID allocation, idempotency, duplicate detection, and malformed ID detection. |
| AC13 | Active roadmap/status docs show M1.5 complete after the migration lands. |

## Architecture Notes

[architecture.md](architecture.md) owns module boundaries, command flow, and
write-order rules. The core rule is simple: parse all relevant YAML, validate
the current IDs, prepare edits in memory, then write YAML files only after all
blocking validation passes.

### Files And Folders Changed

- `src/main.rs`
- `src/cli.rs`
- New Rust modules for source YAML loading and ID migration, likely
  `src/source.rs` or `src/source_ids.rs`.
- `input/sentences/*.yaml`
- `input/words/*.yaml`
- `docs/ROADMAP.md`
- `docs/specs/002-m1-5-yaml-id-migration/**`

### Workflow State Touched

- Brief spec/task files for this spec.
- Roadmap status row for `YAML item IDs migrated`.

### External Effects And Reuse

- Filesystem reads from active YAML source files.
- Filesystem writes only to active YAML source files when migration is not in
  check mode.
- No network calls.
- No Ollama calls.
- No GitHub calls.

## Testing Plan

### Unit Tests

- ID allocation on files with no IDs.
- Preservation of existing IDs.
- Filling gaps without changing existing IDs.
- Duplicate ID detection.
- Malformed ID detection.
- Idempotency: migrated YAML parses to the same IDs on rerun.

### Integration Tests

- Fixture directory with sentence and word YAML files.
- Check mode reports pending migration and writes nothing.
- Migration mode writes IDs only to source YAML.
- Validation mode passes after migration.

### Smoke Tests

```bash
cargo fmt
cargo test
cargo run -- source ids check
cargo run -- source ids migrate --check
cargo run -- source ids migrate
cargo run -- source ids check
git diff --check
```

### Drift / Consistency Checks

- `git diff --name-only` should show active YAML and Rust/docs/spec files only.
- `git diff --name-only -- output audio runs archive/python/legacy-input`
  should be empty.
- `rg -n "id:" input/sentences input/words` should show IDs in all active
  source files after migration.

### Not Covered In This Spec

- Planner behavior that consumes the IDs; M2 owns that.
- `source_ref` generation; M4 owns accepted output generation.

## Open Questions

- None. The active design already chooses file-scoped opaque IDs and explicitly
  says old generated output is not backfilled.
