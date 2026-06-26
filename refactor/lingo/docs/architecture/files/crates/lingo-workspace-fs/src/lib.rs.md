# `crates/lingo-workspace-fs/src/lib.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Exposes the concrete `FsWorkspace` adapter and its explicit constructors while keeping codecs and path mechanics private.

## Scope: this file owns

- module map
- `FsWorkspace` public type
- explicit built-in profile source constructor

## Out of scope: this file must not own

- re-exporting internal DTOs
- global singleton workspace

## Allowed dependencies

- application port traits
- domain models

## Forbidden dependencies and shortcuts

- CLI and other adapters

## Key implementation shape

```rust
mod atomic_file;
mod codecs;
mod config;
mod error;
mod layout;
mod profiles;
mod root;
mod runs;
mod scan;
mod store;

pub use error::FsWorkspaceError;
pub use root::WorkspaceRoot;
pub use store::FsWorkspace;
```

## Required tests / evidence

- public API allowlist
- constructing adapter does not perform hidden writes

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
