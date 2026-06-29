# `crates/lingo-artifacts/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Exposes concrete package and Anki publishers implementing application artifact ports.

## Scope: this file owns

- module map
- publisher constructors

## Out of scope: this file must not own

- format internals as public API
- workspace selection

## Allowed dependencies

- application artifact ports
- domain models

## Forbidden dependencies and shortcuts

- CLI/workspace

## Key implementation shape

```rust
mod anki;
mod checksum;
mod error;
mod manifest;
mod model;
mod package;
mod staging;

pub use anki::ApkgExporter;
pub use package::PortablePackagePublisher;
```

## Required tests / evidence

- public API allowlist

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
