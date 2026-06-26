# `crates/lingo-domain/src/source.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the canonical reviewed input model produced by `lingo import` and consumed by `lingo build`.

## Scope: this file owns

- source batch aggregate
- source item ordering
- title/subtitle values
- source fingerprints and item identity

## Out of scope: this file must not own

- YAML parsing
- raw-file discovery
- LLM reply repair
- file paths

## Allowed dependencies

- IDs, text values, fingerprints

## Forbidden dependencies and shortcuts

- serde YAML codecs
- workspace crate

## Key implementation shape

```rust
pub struct SourceBatch {
    format: SourceFormat,
    batch: BatchId,
    title: SourceTitle,
    subtitle: Option<SourceSubtitle>,
    items: Vec<SourceItem>,
}

impl SourceBatch {
    pub fn try_new(
        batch: BatchId,
        title: SourceTitle,
        subtitle: Option<SourceSubtitle>,
        items: Vec<SourceItem>,
    ) -> Result<Self, SourceError> {
        if items.is_empty() { return Err(SourceError::EmptyBatch); }
        reject_duplicate_item_ids(&items)?;
        Ok(Self { format: SourceFormat::V1, batch, title, subtitle, items })
    }

    pub fn items(&self) -> &[SourceItem] { &self.items }
}
```

## Required tests / evidence

- empty batches rejected
- duplicate source IDs rejected
- all text values are normalized through their value objects
- canonical serialization round-trips through workspace codec

## Design notes

- The aggregate exposes slices and getters, not mutable public fields. Changes happen through named methods or reconstruction.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
