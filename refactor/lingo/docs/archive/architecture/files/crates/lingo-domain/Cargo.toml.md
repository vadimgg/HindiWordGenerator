# `crates/lingo-domain/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the pure domain crate. Its dependency list is intentionally small and must not acquire filesystem, process, HTTP, CLI, or template-engine crates.

## Scope: this file owns

- domain crate package metadata
- domain-only dependencies

## Out of scope: this file must not own

- adapter dependencies
- feature flags that expose infrastructure

## Allowed dependencies

- serde for canonical serialization
- sha2
- unicode-normalization
- thiserror

## Forbidden dependencies and shortcuts

- clap
- reqwest
- handlebars
- toml
- directories
- std::process wrappers

## Key implementation shape

```toml
[package]
name = "lingo-domain"
edition.workspace = true

[dependencies]
serde = { workspace = true, features = ["derive"] }
sha2.workspace = true
thiserror.workspace = true
unicode-normalization.workspace = true

[lints]
workspace = true
```

## Required tests / evidence

- architecture test rejects forbidden dependencies
- `cargo tree -p lingo-domain` remains inward-only

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
