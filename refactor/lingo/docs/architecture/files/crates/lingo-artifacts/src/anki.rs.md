# `crates/lingo-artifacts/src/anki.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the Anki note/model mapping, stable field order, deterministic IDs, media names, and APKG writing.

## Scope: this file owns

- Anki model fields/templates
- note GUID derivation from `CardId`
- deck ID derivation
- media map

## Out of scope: this file must not own

- interactive selection
- card validation
- workspace layout

## Allowed dependencies

- application Anki port
- domain cards/audio
- zip/APKG support

## Forbidden dependencies and shortcuts

- random IDs that churn exports
- HTML escaping spread across callers

## Key implementation shape

```rust
fn note_guid(card: &Card) -> AnkiGuid {
    AnkiGuid::from_stable_hash(card.id().as_str().as_bytes())
}

const FIELDS: &[&str] = &["Lead", "Secondary", "English", "Literal", "Register", "Audio"];
```

## Required tests / evidence

- same card gets same GUID across exports
- display lead controls front field only
- media references resolve
- HTML is escaped once

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
