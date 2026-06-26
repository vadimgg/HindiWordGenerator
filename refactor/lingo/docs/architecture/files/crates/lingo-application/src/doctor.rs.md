# `crates/lingo-application/src/doctor.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns environment-check policy and recovery facts while probes own operating-system mechanics.

## Scope: this file owns

- which capabilities are required by resolved config
- check aggregation
- typed recovery suggestions

## Out of scope: this file must not own

- running shell commands directly
- printing secrets
- terminal colors

## Allowed dependencies

- DeckContextProvider
- EnvironmentProbe
- WorkspaceStore health facts

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn doctor(deps: &DoctorDeps<'_>) -> Result<DoctorReport, DoctorError> {
    let context = deps.context.resolve()?;
    let required = RequiredCapabilities::for_context(&context);
    Ok(DoctorReport::from(deps.environment.probe(&required)?))
}
```

## Required tests / evidence

- ElevenLabs check reports presence only, never key value
- gTTS requirement depends on selected/fallback backends
- read-only guarantee

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
