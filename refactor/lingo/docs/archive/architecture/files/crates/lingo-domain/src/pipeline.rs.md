# `crates/lingo-domain/src/pipeline.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns pipeline-stage vocabulary and derived batch progress facts without pretending the pipeline is one mutable state machine.

## Scope: this file owns

- ordered stage metadata
- audio coverage values
- check status values
- batch progress facts

## Out of scope: this file must not own

- directory scans
- next-command selection
- terminal colors

## Allowed dependencies

- batch IDs and diagnostic summary values

## Forbidden dependencies and shortcuts

- workspace and CLI crates

## Key implementation shape

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipelineStage { Raw, Source, Cards, Check, Audio, Package, Export }

pub struct BatchProgress {
    pub batch: BatchId,
    pub raw_present: bool,
    pub source_present: bool,
    pub cards_present: bool,
    pub check: CheckState,
    pub audio: AudioCoverage,
}
```

## Required tests / evidence

- stage order and labels are centralized
- progress can represent missing cards and orphan audio without impossible enum hacks

## Design notes

- `NextAction` belongs to the application layer because it is workflow policy, not a stored domain fact.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
