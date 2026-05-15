# M2 Sentence Planner

## What We're Doing

Add the read-only planner command that explains what sentence generation would
do next. The command reads active sentence YAML and accepted sentence output,
then reports source validity, existing work, pending work, deferred work, and
the batch filenames Rust would use later.

This is not generation. No model is called, no output batch is written, and no
audio is touched. The planner is the preview and audit step that makes later
generation predictable.

## Why Now

M1.5 gave every source sentence a stable `id`. The next safe step is using
those IDs to derive pending work before M3/M4 add validators and model-backed
generation.

## What Changes

**Before:** `hindi sentences plan` is mentioned in help but intentionally not
available.

**After:** `hindi sentences plan --max-batches 1` prints a read-only plan for
pending sentence batches.

## What Stays The Same

- `output/`, `audio/`, and `runs/` are not written.
- Old output without `source_ref` remains lineage-less.
- The planner does not backfill, repair, generate, or delete data.
- `--max-batches` counts output files across the whole invocation, not source
  items.

## What To Watch During Review

- The planner must be read-only.
- Python-era output without `source_ref` must be reported as `missing lineage`,
  not counted as clean.
- Planned filenames must use the next unused zero-padded batch number for each
  source stem.
- Hindi examples in output must show romanisation directly under Hindi.

## Workflow Example

```bash
cargo run -- sentences plan --max-batches 1
```

## Where To Read More

| If you want to understand... | Read... |
|---|---|
| Exact scope and exclusions | [spec.md](spec.md) |
| Module ownership and drift risks | [architecture.md](architecture.md) |
| Validation strategy | [testing.md](testing.md) |
| CLI output shape | [cli.md](cli.md) |
| Research findings | [research.md](research.md) |
| Implementation sequence | [plan.md](plan.md) |
| Work packages | [tasks.md](tasks.md) |
