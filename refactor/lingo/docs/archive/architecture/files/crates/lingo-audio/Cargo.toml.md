# `crates/lingo-audio/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares concrete audio provider dependencies. This adapter implements the application audio port and contains the only HTTP/process audio code.

## Scope: this file owns

- audio provider dependencies

## Out of scope: this file must not own

- workspace storage
- config file loading
- CLI presentation

## Allowed dependencies

- lingo-domain
- lingo-application
- blocking HTTP client
- process support
- secrecy

## Forbidden dependencies and shortcuts

- lingo-workspace-fs
- lingo-cli
- lingo-artifacts

## Key implementation shape

```toml
[package]
name = "lingo-audio"
edition.workspace = true

[dependencies]
lingo-application = { path = "../lingo-application" }
lingo-domain = { path = "../lingo-domain" }
reqwest = { workspace = true, features = ["blocking", "json"] }
secrecy.workspace = true
thiserror.workspace = true
```

## Required tests / evidence

- dependency audit keeps HTTP/process code isolated
- provider tests use local fake server/process runner seams

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
