# `crates/lingo-audio/src/error.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns provider and catalog errors and maps them to application audio failures with an explicit failure class.

## Scope: this file owns

- catalog errors
- process/HTTP provider errors
- failure classification

## Out of scope: this file must not own

- terminal recovery prose
- secret values

## Allowed dependencies

- thiserror
- domain failure class

## Forbidden dependencies and shortcuts

- CLI

## Key implementation shape

```rust
pub struct BackendError {
    backend: AudioBackendId,
    class: AudioFailureClass,
    source: BackendErrorSource,
}
```

## Required tests / evidence

- classification tests for every provider status family
- Debug output redacts secrets

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
