# `crates/lingo-artifacts/src/checksum.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns package checksum calculation and canonical `sha256:<hex>` formatting.

## Scope: this file owns

- stream/file hashing
- stable digest formatting

## Out of scope: this file must not own

- domain source fingerprints
- filesystem traversal policy

## Allowed dependencies

- sha2

## Forbidden dependencies and shortcuts

- hand-written SHA implementation

## Key implementation shape

```rust
pub fn sha256(bytes: &[u8]) -> Checksum {
    Checksum::from_digest(Sha256::digest(bytes))
}
```

## Required tests / evidence

- known-vector test
- manifest checksum verification

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
