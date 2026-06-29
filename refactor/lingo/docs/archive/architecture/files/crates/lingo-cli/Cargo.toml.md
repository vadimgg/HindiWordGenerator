# `crates/lingo-cli/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the binary/composition edge and may depend on all inward crates plus terminal, HTTP-server, and browser/process helpers.

## Scope: this file owns

- binary metadata
- edge-only dependencies

## Out of scope: this file must not own

- domain rules
- provider implementations outside their adapter crates

## Allowed dependencies

- all six library crates
- clap
- console/terminal helpers
- local HTTP server

## Forbidden dependencies and shortcuts

- downstream crates depending back on lingo-cli

## Key implementation shape

```toml
[package]
name = "lingo-cli"
edition.workspace = true

[[bin]]
name = "lingo"
path = "src/main.rs"

[dependencies]
lingo-application = { path = "../lingo-application" }
lingo-workspace-fs = { path = "../lingo-workspace-fs" }
lingo-prompt = { path = "../lingo-prompt" }
lingo-audio = { path = "../lingo-audio" }
lingo-artifacts = { path = "../lingo-artifacts" }
clap = { workspace = true, features = ["derive"] }
```

## Required tests / evidence

- binary name is `lingo`
- architecture test makes this the only composition root

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
