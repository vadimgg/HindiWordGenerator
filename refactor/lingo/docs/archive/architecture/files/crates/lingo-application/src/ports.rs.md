# `crates/lingo-application/src/ports.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Defines small capability contracts owned by the use cases: workspace persistence, resolved deck context, prompt rendering/parsing, audio synthesis, artifact publication, clock, and environment probes.

## Scope: this file owns

- port traits
- typed port requests/results
- failure categories that use cases can act on

## Out of scope: this file must not own

- provider-specific fields
- filesystem layout
- HTTP status handling
- terminal messages

## Allowed dependencies

- lingo-domain
- std::path only in explicit edge DTOs

## Forbidden dependencies and shortcuts

- concrete adapter constructors
- generic JSON payloads
- one-method traits with no meaningful fake

## Key implementation shape

```rust
pub trait WorkspaceStore {
    fn load_source(&self, batch: &BatchId) -> Result<SourceBatch, WorkspaceFailure>;
    fn create_source(&self, source: &SourceBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn load_cards(&self, batch: &BatchId) -> Result<CardBatch, WorkspaceFailure>;
    fn replace_cards(&self, cards: &CardBatch) -> Result<StoredFile, WorkspaceFailure>;
    fn scan(&self) -> Result<WorkspaceSnapshot, WorkspaceFailure>;
}

pub trait PromptEngine {
    fn render_import(&self, request: ImportPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_import_reply(&self, reply: &str) -> Result<SourceBatchDraft, PromptFailure>;
}

pub trait AudioSynthesizer {
    fn synthesize(&self, request: AudioRequest) -> Result<AudioBytes, AudioFailure>;
}
```

## Required tests / evidence

- fake workspace proves import/build/check orchestration
- fake prompt engine proves prepare/apply split
- fake audio backend covers fallback-class behavior at service boundary

## Design notes

- Ports exist because each boundary has a production adapter and a meaningful fake. Do not add a registry/event bus/plugin contract.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
