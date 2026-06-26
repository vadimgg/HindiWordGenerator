# `Cargo.toml`

> **Target kind:** Cargo manifest  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../ARCHITECTURE.md)

## Responsibility

Defines the Cargo workspace, shared dependency versions, and workspace-wide lint policy. It is the only authoritative list of Rust workspace members.

## Scope: this file owns

- workspace membership
- shared package metadata
- shared dependency versions
- workspace lint defaults

## Out of scope: this file must not own

- crate-specific features
- application behavior
- provider registration
- build scripts that hide code generation

## Allowed dependencies

- Cargo itself

## Forbidden dependencies and shortcuts

- path dependencies outside this repository
- wildcard workspace members
- hidden plugin discovery

## Key implementation shape

```toml
[workspace]
resolver = "2"
members = [
  "crates/lingo-domain",
  "crates/lingo-application",
  "crates/lingo-workspace-fs",
  "crates/lingo-prompt",
  "crates/lingo-audio",
  "crates/lingo-artifacts",
  "crates/lingo-cli",
]

[workspace.package]
edition = "2024"
rust-version = "1.85"

[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[workspace.lints.clippy]
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"
```

## Required tests / evidence

- `cargo metadata --no-deps` lists exactly the seven intended members
- all crates opt into workspace lints
- dependency audit has no forbidden crate edges

## Design notes

- Pin exact compatible dependency versions during repository bootstrap; do not copy a stale version table from this document.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
