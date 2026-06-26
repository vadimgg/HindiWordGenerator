# `crates/lingo-domain/src/card.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the canonical language-neutral card model and the relationship between cards, tokens, words, source lineage, tags, and audio references.

## Scope: this file owns

- card batch aggregate
- card/word/token invariants
- register and grammar vocabulary
- source lineage reference

## Out of scope: this file must not own

- JSON parsing
- Anki mapping
- terminal formatting
- audio synthesis

## Allowed dependencies

- IDs, text, language values, audio references

## Forbidden dependencies and shortcuts

- workspace paths
- provider DTOs
- CLI DTOs

## Key implementation shape

```rust
pub struct Card {
    id: CardId,
    target: TargetText,
    romanisation: Option<Romanisation>,
    english: Gloss,
    literal: Gloss,
    register: Register,
    tokens: Vec<CardToken>,
    words: Vec<Word>,
    tags: CardTags,
    audio: Option<AudioRef>,
    source: SourceRef,
}

impl Card {
    pub fn attach_audio(&mut self, audio: AudioRef) -> Result<(), CardError> {
        if audio.card_id() != &self.id { return Err(CardError::WrongAudioOwner); }
        self.audio = Some(audio);
        Ok(())
    }
}
```

## Required tests / evidence

- token word references resolve
- word IDs are unique per card
- source reference belongs to the same batch
- audio cannot attach to another card
- optional romanisation follows profile validation rather than empty strings

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
