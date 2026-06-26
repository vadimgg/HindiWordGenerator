# `crates/lingo-audio/src/elevenlabs.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Implements ElevenLabs synthesis with typed request DTOs, secret-safe headers, response classification, and bounded body reads.

## Scope: this file owns

- HTTP endpoint construction
- request DTO
- status classification
- audio body extraction

## Out of scope: this file must not own

- environment lookup
- fallback policy
- printing API key

## Allowed dependencies

- blocking HTTP client
- secret string
- audio backend contract

## Forbidden dependencies and shortcuts

- workspace/config/CLI

## Key implementation shape

```rust
let response = self.client
    .post(self.endpoint.for_voice(&self.voice))
    .header("xi-api-key", self.api_key.expose_secret())
    .json(&ElevenLabsRequest::from(request))
    .send()?;

classify_status(response.status())?;
Ok(EncodedAudio::mp3(response.bytes()?.to_vec())?)
```

## Required tests / evidence

- 401/403 configuration, 429/5xx retryable, 4xx invalid request
- secret redacted in Debug and errors
- oversized response rejected

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
