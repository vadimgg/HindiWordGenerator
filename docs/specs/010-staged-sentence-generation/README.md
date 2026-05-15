# Staged Sentence Generation

## What We're Doing

Change `hindi sentences generate` from one large enrichment prompt into a
staged sentence pipeline. Rust will still own trusted source fields, lineage,
validation, accepted writes, and run reports; the model will provide smaller
enrichment pieces: register, literal translation, and word breakdown.

The eval runner showed that the focused prompts are stronger and easier to
debug than the full-enrichment prompt. Full enrichment is useful as a stress
test, but it is slower and less predictable. Generation should use the focused
path by default.

## Why Now

This is the next direct step toward safe real card generation. We already have
planning, validation, atomic writes, Ollama calls, and prompt evaluation. The
remaining risk is model output quality. Staging makes that risk smaller:
individual prompt failures are easier to identify, retry, and improve.

## What Changes

**Before:** `hindi sentences generate` asks one prompt for literal, register,
tokens, and words together.

**After:** `hindi sentences generate` asks focused prompts for register,
literal, and word breakdown, then Rust merges the pieces into the existing
validated candidate schema.

## What Stays The Same

- The command remains `hindi sentences generate --max-batches <n>`.
- Accepted output still writes only under `output/sentences/`.
- Rust still copies Hindi, romanisation, English, tags, title/subtitle, and
  `source_ref` from YAML/planner data.
- Existing validation and atomic writer rules stay authoritative.
- No CLI-managed Ollama model switching is added.
- Full-enrichment eval prompts remain available for comparison; they do not
  become the default generation path.

## What To Watch During Review

- Does generation avoid trusting the model for source fields and lineage?
- Does each stage record enough metadata to debug prompt/model behavior?
- Does validation still happen before any accepted output write?
- Does the implementation avoid hidden fallback to the old full-enrichment
  prompt?
- Does failed generation leave pending work pending and point to the run report?

## Workflow Example

```bash
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
```

## Where To Read More

| If you want to understand... | Read... |
|---|---|
| Exact scope and what is intentionally excluded | [spec.md](spec.md) |
| Module ownership, command flow, and data drift risks | [architecture.md](architecture.md) |
| How this change will be proven safe | [testing.md](testing.md) |
| CLI messages and run output | [cli.md](cli.md) |
| Research findings and code audit notes | [research.md](research.md) |
| Implementation order | [plan.md](plan.md) |
| Work packages | [tasks.md](tasks.md) |
