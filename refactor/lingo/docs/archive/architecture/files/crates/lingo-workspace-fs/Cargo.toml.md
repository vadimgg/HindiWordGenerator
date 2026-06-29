# `crates/lingo-workspace-fs/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares the filesystem adapter and its parsing/config dependencies. It implements application ports but must not depend on other adapters or the CLI.

## Scope: this file owns

- filesystem adapter dependencies
- embedded profile assets

## Out of scope: this file must not own

- audio, prompt rendering, artifact writing, presentation

## Allowed dependencies

- lingo-domain
- lingo-application
- serde_yaml
- serde_json
- toml
- directories

## Forbidden dependencies and shortcuts

- lingo-prompt
- lingo-audio
- lingo-artifacts
- lingo-cli

## Key implementation shape

```toml
[package]
name = "lingo-workspace-fs"
edition.workspace = true

[dependencies]
lingo-application = { path = "../lingo-application" }
lingo-domain = { path = "../lingo-domain" }
serde_json.workspace = true
serde_yaml.workspace = true
toml.workspace = true
thiserror.workspace = true
```

## Required tests / evidence

- architecture test rejects adapter-to-adapter edges
- fixture round-trips exercise actual codecs

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
