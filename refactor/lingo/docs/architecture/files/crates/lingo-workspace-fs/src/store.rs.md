# `crates/lingo-workspace-fs/src/store.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Implements canonical raw/source/card/audio persistence ports using typed layout and codecs.

## Scope: this file owns

- read/list/create/replace operations
- collision semantics
- audio byte persistence
- port implementation

## Out of scope: this file must not own

- business validation
- prompt parsing
- package publication
- terminal messages

## Allowed dependencies

- layout
- codecs
- atomic file writer
- application ports

## Forbidden dependencies and shortcuts

- direct path strings in use cases
- silent overwrite

## Key implementation shape

```rust
impl WorkspaceStore for FsWorkspace {
    fn create_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.source_file(source.batch_id());
        let bytes = encode_source(source).map_err(map_codec)?;
        create_atomic(&path, &bytes).map_err(map_io)?;
        Ok(self.stored_file(path))
    }

    fn replace_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure> {
        let path = self.layout.card_file(cards.batch_id());
        replace_atomic(&path, &encode_cards(cards)?).map_err(map_io)?;
        Ok(self.stored_file(path))
    }
}
```

## Required tests / evidence

- create refuses collision
- replace preserves prior file on failure
- round-trip returns equal domain value
- audio path remains safe

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
