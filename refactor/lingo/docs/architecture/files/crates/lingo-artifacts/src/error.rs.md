# `crates/lingo-artifacts/src/error.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns artifact planning, encoding, integrity, and publication errors with safe path context.

## Scope: this file owns

- typed error variants
- source chaining
- application port mapping

## Out of scope: this file must not own

- terminal prose
- canonical data repair

## Allowed dependencies

- thiserror

## Forbidden dependencies and shortcuts

- CLI types

## Key implementation shape

```rust
#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("artifact would contain unsafe path {path}")]
    UnsafePath { path: String },
    #[error("integrity verification failed for {path}")]
    Integrity { path: PathBuf },
}
```

## Required tests / evidence

- unsafe paths and checksum mismatches covered
- errors do not include card text unnecessarily

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
