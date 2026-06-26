# `crates/lingo-workspace-fs/src/codecs.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns canonical `lingo.source/v1` YAML and `lingo.cards/v1` JSON wire vocabulary and maps file DTOs to valid domain aggregates.

## Scope: this file owns

- persisted keys and format tags
- DTO-to-domain conversion
- pretty deterministic output
- newline termination

## Out of scope: this file must not own

- prompt reply leniency
- domain validation policy duplication
- terminal JSON

## Allowed dependencies

- serde YAML/JSON
- domain constructors

## Forbidden dependencies and shortcuts

- using domain aggregates with public mutable fields
- substring-based contract tests

## Key implementation shape

```rust
#[derive(Deserialize)]
struct SourceBatchFileDto {
    format: String,
    batch: String,
    title: String,
    subtitle: Option<String>,
    items: Vec<SourceItemFileDto>,
}

pub fn decode_source(bytes: &[u8]) -> Result<SourceBatch, CodecError> {
    let dto: SourceBatchFileDto = serde_yaml::from_slice(bytes)?;
    require_format(&dto.format, "lingo.source/v1")?;
    dto.try_into()
}
```

## Required tests / evidence

- golden structured round-trip for both formats
- unknown format version rejected
- invalid DTO cannot create invalid domain aggregate

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
