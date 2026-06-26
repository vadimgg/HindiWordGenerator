# `crates/lingo-domain/src/language.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns language facts and Rust-supported semantic policies used by validation and presentation.

## Scope: this file owns

- language/profile identity values
- script direction
- romanisation requirement and convention
- display lead

## Out of scope: this file must not own

- profile file resolution
- prompt text
- audio provider execution
- arbitrary validation expressions from TOML

## Allowed dependencies

- `ids.rs` value objects

## Forbidden dependencies and shortcuts

- filesystem and template engines

## Key implementation shape

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RomanisationConvention {
    None,
    IastTilde,
    Hepburn,
}

impl RomanisationConvention {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::IastTilde => "iast-tilde",
            Self::Hepburn => "hepburn",
        }
    }
}

pub struct LanguageProfile {
    id: ProfileId,
    language: LanguageName,
    code: LanguageCode,
    script: ScriptName,
    direction: TextDirection,
    romanisation: RomanisationConvention,
}
```

## Required tests / evidence

- all closed variants have stable metadata
- unknown profile convention fails during config parsing
- Latin-script profile may choose `None`

## Design notes

- A new profile is data-only only when it composes existing Rust-owned conventions. New semantic validation requires code and tests.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
