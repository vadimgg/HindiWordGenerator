# `crates/lingo-application/src/audio.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns missing/replace audio planning, provider invocation order, durable-write sequencing, and card-reference updates.

## Scope: this file owns

- audio selection mode
- fallback-eligible outcome handling
- write audio before card reference
- per-card typed result

## Out of scope: this file must not own

- provider HTTP/process mechanics
- MP3 path joining
- terminal progress rendering

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- AudioSynthesizer
- Clock if run timestamps are reported

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn synthesize_audio(
    deps: &AudioDeps<'_>,
    request: AudioCommand,
) -> Result<AudioReport, AudioError> {
    let mut cards = deps.workspace.load_cards(&request.batch)?;
    for card in cards.cards_mut_for_audio(request.mode) {
        let bytes = deps.audio.synthesize(AudioRequest::from_card(card, deps.context.audio()?))?;
        let audio_ref = deps.workspace.write_audio(card.id(), &bytes)?;
        card.attach_audio(audio_ref)?;
    }
    deps.workspace.replace_cards(&cards)?;
    Ok(AudioReport::from(cards))
}
```

## Required tests / evidence

- existing audio skipped in missing-only mode
- replace mode is explicit enum, not boolean
- card never references unwritten audio
- non-retryable provider failure does not fall back

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
