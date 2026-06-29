# `crates/lingo-prompt/src/import_reply.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Parses an untrusted import YAML reply into a typed draft and reports structural issues without mutating canonical data.

## Scope: this file owns

- optional single code-fence removal
- reply DTO
- field-level parse diagnostics
- draft conversion

## Out of scope: this file must not own

- source ID policy
- canonical file writes
- silent repair of invalid values

## Allowed dependencies

- serde_yaml
- application draft types
- domain value constructors

## Forbidden dependencies and shortcuts

- accepting prose before/after the document
- multiple YAML documents

## Key implementation shape

```rust
pub(crate) fn parse_import_reply(raw: &str) -> Result<SourceBatchDraft, PromptAdapterError> {
    let body = strip_one_optional_fence(raw)?;
    let dto: ImportReplyDto = serde_yaml::from_str(body)?;
    dto.try_into()
}
```

## Required tests / evidence

- plain YAML and one fenced YAML block accepted
- surrounding prose rejected
- missing target/English produces structured path

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
