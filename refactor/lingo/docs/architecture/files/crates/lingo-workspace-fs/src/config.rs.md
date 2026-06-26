# `crates/lingo-workspace-fs/src/config.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Loads and merges built-in defaults, global config, profile facts, deck config, deck profile override, and per-run overrides into one typed resolved context with provenance.

## Scope: this file owns

- layer order
- TOML DTOs
- typed merge rules
- origin/provenance for every effective field
- environment-variable names, not secret values

## Out of scope: this file must not own

- prompt rendering
- domain validation expressions from config
- reading secret contents

## Allowed dependencies

- domain profile/config values
- application context port

## Forbidden dependencies and shortcuts

- CLI flags directly
- provider clients

## Key implementation shape

```rust
pub fn resolve_context(sources: ConfigSources) -> Result<ResolvedDeckContext, ConfigError> {
    let mut merged = ResolvedBuilder::from_builtin_defaults();
    merged.apply_optional(load_global(&sources.global)? , ConfigOrigin::Global)?;
    merged.apply_profile(load_profile(&sources.profile)?, ConfigOrigin::Profile)?;
    merged.apply_optional(load_deck(&sources.deck)?, ConfigOrigin::Deck)?;
    merged.apply_optional(load_deck_profile(&sources.deck_profile)?, ConfigOrigin::DeckProfile)?;
    merged.finish()
}
```

## Required tests / evidence

- precedence matrix
- unknown enum values fail loudly
- missing optional layers are okay
- API keys are never deserialized as values

## Design notes

- Config composes Rust-owned choices; it is not a scripting language.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
