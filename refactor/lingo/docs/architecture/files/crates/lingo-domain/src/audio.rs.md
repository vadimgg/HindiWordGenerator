# `crates/lingo-domain/src/audio.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns audio-related domain values: backend identity, voice identity, audio format, relative artifact reference, and synthesis outcome facts.

## Scope: this file owns

- typed backend IDs
- typed audio references
- audio content hash
- retry classification vocabulary

## Out of scope: this file must not own

- subprocess invocation
- HTTP calls
- environment lookup
- file writes

## Allowed dependencies

- card IDs and fingerprints

## Forbidden dependencies and shortcuts

- reqwest
- std::process
- workspace paths

## Key implementation shape

```rust
pub struct AudioRef {
    card_id: CardId,
    relative_path: ArtifactPath,
    content_hash: ContentHash,
    backend: AudioBackendId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioFailureClass {
    Retryable,
    Configuration,
    InvalidRequest,
}
```

## Required tests / evidence

- unsafe paths are rejected
- audio ref round-trip preserves backend and hash
- failure-class wire names remain stable

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
