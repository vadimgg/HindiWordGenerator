# `crates/lingo-workspace-fs/src/error.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns adapter-level errors with path and source context and maps them into application port failures without presentation prose.

## Scope: this file owns

- filesystem/config/codec error variants
- source chaining
- port failure mapping

## Out of scope: this file must not own

- terminal recovery paragraphs
- domain diagnostics
- string parsing to classify errors

## Allowed dependencies

- thiserror
- application failure enums

## Forbidden dependencies and shortcuts

- clap and console output

## Key implementation shape

```rust
#[derive(Debug, thiserror::Error)]
pub enum FsWorkspaceError {
    #[error("workspace file already exists: {path}")]
    Collision { path: PathBuf },
    #[error("could not read {path}")]
    Read { path: PathBuf, #[source] source: io::Error },
    #[error("invalid canonical document {path}")]
    Decode { path: PathBuf, #[source] source: CodecError },
}
```

## Required tests / evidence

- source error is preserved
- secret values never enter debug/display output

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
