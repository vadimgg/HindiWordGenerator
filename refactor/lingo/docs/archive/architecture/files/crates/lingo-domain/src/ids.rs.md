# `crates/lingo-domain/src/ids.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns validated identifiers and their stable textual vocabulary: profile, batch, source item, card, word, run, and artifact identifiers.

## Scope: this file owns

- parsing and validation of IDs
- stable `Display`/wire form
- deterministic ID composition

## Out of scope: this file must not own

- slugging user-facing titles
- filesystem paths
- random ID generation
- CLI parsing

## Allowed dependencies

- standard library
- thiserror

## Forbidden dependencies and shortcuts

- workspace layout strings
- provider names not represented by typed IDs

## Key implementation shape

```rust
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct BatchId(String);

impl BatchId {
    pub fn parse(raw: impl Into<String>) -> Result<Self, IdError> {
        let raw = raw.into();
        let valid = !raw.is_empty()
            && raw.len() <= 80
            && raw.bytes().all(|b| b.is_ascii_lowercase()
                || b.is_ascii_digit() || matches!(b, b'-' | b'_'));
        valid.then_some(Self(raw)).ok_or(IdError::InvalidBatch)
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

pub struct CardId {
    batch: BatchId,
    source_item: SourceItemId,
}
```

## Required tests / evidence

- valid/invalid parsing for every ID
- stable display round-trip
- card IDs cannot be constructed from unrelated raw strings

## Design notes

- Do not expose tuple fields publicly. Do not use type aliases such as `type BatchId = String`.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
