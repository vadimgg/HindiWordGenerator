# `crates/lingo-prompt/src/build_reply.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Parses an untrusted build JSON reply into a card draft while preserving source IDs and rejecting unknown/extra source items.

## Scope: this file owns

- JSON reply DTO
- typed draft conversion
- source reference extraction

## Out of scope: this file must not own

- semantic card acceptance
- lineage validation against workspace
- automatic field invention

## Allowed dependencies

- serde_json
- application/domain draft values

## Forbidden dependencies and shortcuts

- generic JSON values in application service

## Key implementation shape

```rust
pub(crate) fn parse_build_reply(raw: &str) -> Result<CardBatchDraft, PromptAdapterError> {
    let body = strip_one_optional_fence(raw)?;
    let dto: BuildReplyDto = serde_json::from_str(body)?;
    CardBatchDraft::try_from(dto).map_err(PromptAdapterError::InvalidDraft)
}
```

## Required tests / evidence

- unknown fields policy is explicit
- missing source item ID rejected
- empty strings fail through value constructors

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
