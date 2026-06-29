# `rust-toolchain.toml`

> **Target kind:** Toolchain configuration  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../ARCHITECTURE.md)

## Responsibility

Pins the Rust toolchain and required components so local development and CI compile the same language edition and lint set.

## Scope: this file owns

- toolchain channel
- rustfmt component
- clippy component

## Out of scope: this file must not own

- crate dependencies
- CI workflow policy
- runtime configuration

## Allowed dependencies

- rustup

## Forbidden dependencies and shortcuts

- nightly-only features unless separately approved

## Key implementation shape

```toml
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
profile = "minimal"
```

## Required tests / evidence

- `cargo fmt --check` works on a clean checkout
- `cargo clippy --workspace --all-targets` uses the pinned toolchain

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
