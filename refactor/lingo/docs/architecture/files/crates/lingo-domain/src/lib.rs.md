# `crates/lingo-domain/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Defines the public domain surface and documents the invariant that every exported aggregate is valid by construction.

## Scope: this file owns

- module declarations
- small intentional re-exports
- crate-level architecture documentation

## Out of scope: this file must not own

- business behavior itself
- glob re-exports
- adapter convenience shims

## Allowed dependencies

- sibling domain modules

## Forbidden dependencies and shortcuts

- any other workspace crate

## Key implementation shape

```rust
//! Language-neutral learning-card domain.
//!
//! Canonical aggregates are valid by construction. Boundary crates parse
//! untrusted YAML/JSON into DTOs and then call these constructors.

mod audio;
mod card;
mod diagnostic;
mod fingerprint;
mod ids;
mod language;
mod pipeline;
mod source;
mod validation;

pub use card::{Card, CardBatch, CardToken, Word};
pub use ids::{BatchId, CardId, SourceItemId};
pub use validation::{check_card_batch, ValidationReport};
```

## Required tests / evidence

- public API allowlist test catches accidental exports
- no private DTO type is re-exported

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
