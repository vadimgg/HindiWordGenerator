# `crates/lingo-application/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Declares use-case modules and exposes only requests, reports, errors, and port contracts needed by composition roots.

## Scope: this file owns

- module map
- intentional public API

## Out of scope: this file must not own

- global service locator
- concrete adapter construction
- presentation helpers

## Allowed dependencies

- application modules
- lingo-domain

## Forbidden dependencies and shortcuts

- adapter crates and CLI

## Key implementation shape

```rust
pub mod ports;

mod audio;
mod build;
mod check;
mod doctor;
mod export;
mod import;
mod init;
mod lang;
mod package;
mod report;
mod status;
mod viewer;

pub use import::{ApplyImport, ImportReport, PrepareImport};
pub use report::{CommandHint, NextAction};
```

## Required tests / evidence

- public API allowlist
- no concrete adapter type leaks through signatures

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
