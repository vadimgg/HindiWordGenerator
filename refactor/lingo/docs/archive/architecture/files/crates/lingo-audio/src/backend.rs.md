# `crates/lingo-audio/src/backend.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Defines the internal backend contract justified by two real production implementations and meaningful fakes.

## Scope: this file owns

- backend identity
- synthesis operation
- typed backend error classification

## Out of scope: this file must not own

- catalog selection
- fallback policy
- workspace writes

## Allowed dependencies

- adapter model values

## Forbidden dependencies and shortcuts

- public plugin API
- self-registration

## Key implementation shape

```rust
pub(crate) trait AudioBackend: Send + Sync {
    fn id(&self) -> AudioBackendId;
    fn synthesize(&self, request: &BackendRequest<'_>) -> Result<EncodedAudio, BackendError>;
}
```

## Required tests / evidence

- conformance suite runs against gTTS, ElevenLabs fake, and in-memory fake
- backend ID stable

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
