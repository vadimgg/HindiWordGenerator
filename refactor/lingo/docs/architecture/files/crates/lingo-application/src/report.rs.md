# `crates/lingo-application/src/report.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns shared typed workflow report concepts and next-action policy vocabulary. CLI and JSON edges map these facts into presentation.

## Scope: this file owns

- `CommandHint`
- `NextAction`
- created/updated/skipped counts
- path facts returned by ports

## Out of scope: this file must not own

- ANSI colors
- terminal tables
- serialized CLI DTO keys

## Allowed dependencies

- lingo-domain IDs and stage values

## Forbidden dependencies and shortcuts

- clap
- console styling libraries

## Key implementation shape

```rust
pub enum NextAction {
    Import { raw: Option<RawDocumentId> },
    Build { batch: BatchId },
    Check { batch: Option<BatchId> },
    Audio { batch: Option<BatchId> },
    Package,
    Export,
    None,
}

impl NextAction {
    pub fn command_hint(&self) -> Option<CommandHint> { /* one mapping */ }
}
```

## Required tests / evidence

- all variants map to one stable command hint
- no CLI renderer reimplements the mapping

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
