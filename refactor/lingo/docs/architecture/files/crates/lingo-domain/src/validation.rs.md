# `crates/lingo-domain/src/validation.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns deterministic source/card validation, lineage checks, and romanisation policy checks as typed diagnostics.

## Scope: this file owns

- source validation
- card validation
- lineage coverage
- profile-aware romanisation checks

## Out of scope: this file must not own

- file loading
- automatic repair
- terminal rendering
- LLM-based grading

## Allowed dependencies

- source/card aggregates
- language policy
- diagnostics

## Forbidden dependencies and shortcuts

- workspace paths
- profile TOML structures
- prompt templates

## Key implementation shape

```rust
pub fn check_card_batch(
    cards: &CardBatch,
    source: &SourceBatch,
    profile: &LanguageProfile,
) -> ValidationReport {
    let mut report = ValidationReport::new();
    check_batch_identity(cards, source, &mut report);
    check_source_coverage(cards, source, &mut report);
    check_card_content(cards, profile, &mut report);
    report
}
```

## Required tests / evidence

- missing/duplicate lineage
- fingerprint drift
- romanisation required/forbidden cases
- token-word consistency
- register vocabulary

## Design notes

- Do not make validation return strings and later reverse-parse them into structured issues.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
