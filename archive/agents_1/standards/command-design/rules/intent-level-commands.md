# Intent-Level Commands

Use this rule when proposing, reviewing, or documenting user-facing commands.

## Prefer User Intent

Good active command names should describe what the user wants:

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
hindi viewer
hindi export --source "Complete Hindi" --topic "Chapter 02"
```

Avoid exposing implementation steps as the primary interface:

```bash
hindi hash-inputs
hindi split-lines
hindi validate-json-internal
hindi update-manifest
```

Internal steps can exist, but they should not be the default workflow the user
has to remember.

## Rules

- Every command that writes learner data or generated artifacts needs a preview
  or dry-run path.
- Write-capable commands should clearly name what they will write and what they
  will not touch.
- Local-model commands should report requested model, loaded model, timing, and
  validation result.
- Public commands should use user-facing workflow names, not internal model role
  names such as `sentence_generation`.
- The first Rust implementation should check model readiness and print the
  needed `ollama run ...` command when the wrong model is loaded. Do not add
  CLI-managed model switching until the workflow actually needs it.
- Commands with `--json` must suppress interactive prompts and return a stable
  machine-readable success/error envelope.
- Document exit codes for write-capable or automation-facing commands.
- Errors should give the next concrete recovery command when possible.
- Rust command design should preserve the safety behavior documented in
  `docs/DESIGN.md`.
