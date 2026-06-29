# `crates/lingo-audio/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Exposes the configured audio catalog as the application `AudioSynthesizer` implementation.

## Scope: this file owns

- module map
- `AudioCatalog` export
- provider configuration constructors

## Out of scope: this file must not own

- provider internals
- workspace writes
- secret lookup

## Allowed dependencies

- application audio port
- domain audio values

## Forbidden dependencies and shortcuts

- CLI and workspace

## Key implementation shape

```rust
mod backend;
mod catalog;
mod elevenlabs;
mod error;
mod fallback;
mod gtts;
mod model;

pub use catalog::{AudioCatalog, AudioCatalogBuilder};
pub use error::AudioAdapterError;
```

## Required tests / evidence

- public API exposes catalog, not concrete trait objects

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
