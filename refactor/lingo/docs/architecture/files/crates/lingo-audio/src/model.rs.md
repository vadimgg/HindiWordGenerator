# `crates/lingo-audio/src/model.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns adapter-internal provider request/response values and converts them to application/domain audio types.

## Scope: this file owns

- provider-neutral synthesis input
- encoded audio bytes
- provider attempt facts

## Out of scope: this file must not own

- card mutation
- filesystem paths
- HTTP DTOs shared across providers

## Allowed dependencies

- application audio request
- domain IDs

## Forbidden dependencies and shortcuts

- serde_json::Value as provider-neutral state

## Key implementation shape

```rust
pub(crate) struct BackendRequest<'a> {
    pub card_id: &'a CardId,
    pub text: &'a TargetText,
    pub language: &'a LanguageCode,
    pub voice: Option<&'a VoiceId>,
}

pub(crate) struct EncodedAudio {
    pub bytes: Vec<u8>,
    pub mime: AudioMime,
}
```

## Required tests / evidence

- empty provider bytes rejected
- MIME/type classification preserved

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
