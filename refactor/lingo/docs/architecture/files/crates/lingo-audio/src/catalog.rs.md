# `crates/lingo-audio/src/catalog.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the explicit duplicate-checked catalog of configured audio backends and implements the application audio port.

## Scope: this file owns

- visible backend list
- ID lookup
- primary/fallback selection entry point

## Out of scope: this file must not own

- hidden registration
- provider construction from config files
- retry loops

## Allowed dependencies

- AudioBackend trait
- fallback policy

## Forbidden dependencies and shortcuts

- runtime scanning
- inventory/linkme

## Key implementation shape

```rust
let audio = AudioCatalogBuilder::new()
    .add(Box::new(GttsBackend::new(process_runner)))?
    .add(Box::new(ElevenLabsBackend::new(client, api_key)))?
    .build()?;
```

## Required tests / evidence

- duplicate IDs rejected
- unknown configured backend rejected
- catalog ordering deterministic

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
