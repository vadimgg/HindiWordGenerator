# `crates/lingo-prompt/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Exposes the concrete `HandlebarsPromptEngine` and keeps reply DTOs private.

## Scope: this file owns

- module map
- prompt engine constructor

## Out of scope: this file must not own

- profile file loading
- interactive process behavior
- DTO re-exports

## Allowed dependencies

- application prompt port
- domain models

## Forbidden dependencies and shortcuts

- workspace and CLI

## Key implementation shape

```rust
mod build_reply;
mod error;
mod import_reply;
mod packet;
mod render;

pub use error::PromptAdapterError;
pub use render::HandlebarsPromptEngine;
```

## Required tests / evidence

- public API allowlist
- constructor enables Handlebars strict mode

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
