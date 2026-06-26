# `crates/lingo-audio/src/fallback.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns when and how a primary audio failure may fall back to a configured secondary backend.

## Scope: this file owns

- retryable classification check
- single fallback attempt
- attempt report

## Out of scope: this file must not own

- provider mechanics
- unbounded retries
- fallback on invalid input

## Allowed dependencies

- catalog lookup
- backend errors

## Forbidden dependencies and shortcuts

- sleep/backoff loops without a separate spec

## Key implementation shape

```rust
match primary.synthesize(request) {
    Ok(audio) => Ok(SynthesisOutcome::primary(primary.id(), audio)),
    Err(error) if error.class() == AudioFailureClass::Retryable => {
        let fallback = require_distinct_fallback(catalog, plan)?;
        fallback.synthesize(request).map(|audio| SynthesisOutcome::fallback(primary.id(), fallback.id(), audio))
    }
    Err(error) => Err(error.into()),
}
```

## Required tests / evidence

- no fallback for configuration/invalid request
- primary and fallback cannot be same ID
- only one fallback attempt

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
