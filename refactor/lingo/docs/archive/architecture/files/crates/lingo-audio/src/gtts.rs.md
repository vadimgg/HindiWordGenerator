# `crates/lingo-audio/src/gtts.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Implements gTTS through an explicit subprocess invocation and captures stdout/stderr without shell interpolation.

## Scope: this file owns

- argument construction
- temporary output handling
- exit-status classification

## Out of scope: this file must not own

- shell scripts
- workspace destination paths
- fallback policy

## Allowed dependencies

- process runner seam
- audio backend contract

## Forbidden dependencies and shortcuts

- `sh -c` with learner text
- assuming `uv` exists without doctor check

## Key implementation shape

```rust
let status = Command::new(&self.uv)
    .args(["run", "--with", "gtts", "gtts-cli", request.text.as_str(), "--lang"])
    .arg(request.language.as_str())
    .args(["--output"])
    .arg(&temp_file)
    .status()?;
```

## Required tests / evidence

- target text is passed as an argument, never shell-concatenated
- non-zero exit captures safe diagnostics
- empty output rejected

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
