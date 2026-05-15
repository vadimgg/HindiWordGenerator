# M1.5 YAML ID Migration

## What We're Doing

Add stable `id` fields to every active source item under `input/sentences/` and
`input/words/`. These IDs are file-scoped handles such as `"0001"`; they are
not globally unique by themselves.

This is the bridge between the YAML source migration and the Rust planner. Once
source rows have stable IDs, later Rust commands can identify pending, done, and
changed items by lineage instead of by list position.

## Why Now

M2 depends on source IDs. Without this migration, `hindi sentences plan` would
have to infer identity from item position or content, and both are brittle: item
position changes when we insert a row, and content changes when we fix source
text.

## What Changes

**Before:** Source YAML items contain `hindi`, `romanisation`, and `english`,
but no stable item ID.

**After:** Every active source YAML item has an `id` field that is unique within
its file and stable once committed.

## What Stays The Same

- `output/` remains untouched; old generated cards do not get `source_ref`
  backfilled.
- `audio/` remains untouched.
- Existing source item order is preserved.
- CSV files remain archived history only.
- The Rust CLI still exposes only M1 behavior; no planner command ships in this
  spec.

## What To Watch During Review

- IDs are short, quoted strings like `"0001"`, not bare numbers.
- Existing IDs are preserved on rerun.
- Duplicate IDs within one file are reported as errors.
- The migration never changes `output/`, `audio/`, or archived CSV files.
- The migration does not embed chapter names, source stems, or filenames inside
  IDs.

## Workflow Example

```bash
cargo run -- source ids check
cargo run -- source ids migrate --check
cargo run -- source ids migrate
cargo run -- source ids check
```

## Where To Read More

| If you want to understand... | Read... |
|---|---|
| Exact scope and what is intentionally excluded | [spec.md](spec.md) |
| Ownership, write ordering, and drift risks | [architecture.md](architecture.md) |
| How this change will be proven safe | [testing.md](testing.md) |
| CLI commands and output | [cli.md](cli.md) |
| Source audit notes | [research.md](research.md) |
| Implementation order | [plan.md](plan.md) |
| Work packages | [tasks.md](tasks.md) |
