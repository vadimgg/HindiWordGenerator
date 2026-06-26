# `crates/lingo-domain/src/fingerprint.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns canonical text normalization and SHA-256 fingerprint construction used for lineage and stale-output detection.

## Scope: this file owns

- Unicode NFC normalization
- whitespace normalization
- fingerprint prefixes and composition order

## Out of scope: this file must not own

- hand-written cryptography
- filesystem hashing policy
- package manifest traversal

## Allowed dependencies

- sha2
- unicode-normalization
- text value objects

## Forbidden dependencies and shortcuts

- custom SHA-256 implementation
- platform-dependent normalization

## Key implementation shape

```rust
pub fn source_fingerprint(item: &SourceTextParts) -> SourceFingerprint {
    let canonical = [
        normalize(&item.target),
        item.romanisation.as_deref().map(normalize).unwrap_or_default(),
        normalize(&item.english),
    ].join("\n");

    SourceFingerprint::from_sha256(Sha256::digest(canonical.as_bytes()))
}
```

## Required tests / evidence

- equivalent Unicode forms fingerprint identically
- whitespace-only differences do not churn IDs
- meaningful text changes change fingerprints

## Design notes

- Reuse the attached code’s normalization idea, but use the maintained `sha2` crate rather than a local SHA implementation.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
