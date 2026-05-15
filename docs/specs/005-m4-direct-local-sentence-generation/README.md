# 005 - M4 Direct Local Sentence Generation

## Goal

Add the first Rust command that generates accepted sentence cards with a local
Ollama model:

```bash
hindi sentences generate --max-batches 1
```

## User-Visible Result

The user starts Ollama separately, then runs generation. The CLI checks
readiness, calls one configured model for sentence enrichment, validates the
merged result, writes accepted output, and records a run report.

## Key Boundaries

- One required model role: `sentence_generation`.
- No automatic Ollama spawning, stopping, unloading, or model switching.
- Rust owns trusted source fields and lineage.
- Model owns enrichment only.
- Accepted output is written only after M3 validation passes.
- Failed attempts may write run reports, but not accepted output.

## Documents

- [spec.md](spec.md)
- [plan.md](plan.md)
- [architecture.md](architecture.md)
- [testing.md](testing.md)
- [cli.md](cli.md)
- [research.md](research.md)
- [tasks.md](tasks.md)
- [review.md](review.md)
