# `crates/lingo-prompt/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the prompt adapter. It implements the application prompt port using Handlebars and strict YAML/JSON boundary DTOs.

## Scope: this file owns

- prompt adapter dependencies

## Out of scope: this file must not own

- editor/clipboard processes
- workspace filesystem
- audio/providers

## Allowed dependencies

- lingo-domain
- lingo-application
- handlebars
- serde_yaml
- serde_json

## Forbidden dependencies and shortcuts

- lingo-workspace-fs
- lingo-cli
- reqwest

## Key implementation shape

```toml
[package]
name = "lingo-prompt"
edition.workspace = true

[dependencies]
lingo-application = { path = "../lingo-application" }
lingo-domain = { path = "../lingo-domain" }
handlebars.workspace = true
serde_json.workspace = true
serde_yaml.workspace = true
```

## Required tests / evidence

- architecture test rejects filesystem/CLI dependencies
- prompt snapshots are deterministic

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
