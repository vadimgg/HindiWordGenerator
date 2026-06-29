# `crates/lingo-artifacts/Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../ARCHITECTURE.md)

## Responsibility

Declares generated publication artifact dependencies: JSON/JSONL, checksums, zip/APKG support, and atomic directory staging.

## Scope: this file owns

- artifact adapter dependencies

## Out of scope: this file must not own

- workspace canonical storage
- prompt/audio providers
- CLI selection

## Allowed dependencies

- lingo-domain
- lingo-application
- serde_json
- sha2
- zip

## Forbidden dependencies and shortcuts

- lingo-workspace-fs
- lingo-cli
- reqwest

## Key implementation shape

```toml
[package]
name = "lingo-artifacts"
edition.workspace = true

[dependencies]
lingo-application = { path = "../lingo-application" }
lingo-domain = { path = "../lingo-domain" }
serde_json.workspace = true
sha2.workspace = true
thiserror.workspace = true
zip.workspace = true
```

## Required tests / evidence

- architecture test rejects canonical workspace dependency
- published artifacts are read back in tests

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
