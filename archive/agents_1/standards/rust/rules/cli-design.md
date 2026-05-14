# Rust CLI Design

Use this rule when designing or reviewing Rust commands.

## Rules

- Commands should name user intent, not implementation detail.
- Prefer the explicit first workflow: `hindi sentences plan`,
  `hindi sentences generate`, `hindi sentences audio`, `hindi viewer`.
- Every write-capable command needs a preview or dry-run path.
- Output should clearly name:
  - what was read
  - what will be written
  - what was skipped
  - what failed
  - what to run next
- Keep provider names/model role names in config, status details, and run
  reports.
- Generation commands should check model readiness and fail with the exact
  `ollama run ...` command when the wrong model is loaded.
- `--json` output should be non-interactive, stable, and suitable for agents or
  scripts.
- Every command family should have documented exit-code behavior before it is
  treated as implementation-ready.
- Slow generation commands should print per-batch progress and timing.
- Errors should be short, specific, and actionable.

## Preferred Command Surface

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
hindi viewer
hindi export --source "Complete Hindi" --topic "Chapter 02"
```
