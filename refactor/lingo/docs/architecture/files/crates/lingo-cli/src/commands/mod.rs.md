# `crates/lingo-cli/src/commands/mod.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../../ARCHITECTURE.md)

## Responsibility

Lists command modules explicitly and provides the single dispatch function.

## Scope: this file owns

- visible command catalog
- dispatch

## Out of scope: this file must not own

- command behavior
- dynamic registration

## Allowed dependencies

- all command modules

## Forbidden dependencies and shortcuts

- inventory/linkme

## Key implementation shape

```rust
pub mod audio;
pub mod build;
pub mod check;
pub mod doctor;
pub mod export;
pub mod import;
pub mod init;
pub mod lang;
pub mod package;
pub mod status;
pub mod viewer;
```

## Required tests / evidence

- every Clap variant has exactly one dispatch arm
- no hidden commands

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
