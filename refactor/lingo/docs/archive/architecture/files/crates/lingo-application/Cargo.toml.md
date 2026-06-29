# `crates/lingo-application/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the use-case and port crate. It may depend only on the domain crate and small error/async-neutral utilities.

## Scope: this file owns

- application crate metadata
- inward dependency declaration

## Out of scope: this file must not own

- concrete adapters
- clap
- filesystem libraries
- HTTP/process/template dependencies

## Allowed dependencies

- lingo-domain
- thiserror

## Forbidden dependencies and shortcuts

- lingo-workspace-fs
- lingo-prompt
- lingo-audio
- lingo-artifacts
- lingo-cli

## Key implementation shape

```toml
[package]
name = "lingo-application"
edition.workspace = true

[dependencies]
lingo-domain = { path = "../lingo-domain" }
thiserror.workspace = true

[lints]
workspace = true
```

## Required tests / evidence

- architecture test proves no adapter dependency
- application unit tests use meaningful fakes for ports

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
