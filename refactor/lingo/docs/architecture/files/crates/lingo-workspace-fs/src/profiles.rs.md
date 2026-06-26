# `crates/lingo-workspace-fs/src/profiles.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the explicit built-in profile catalog and layered resolution of profile facts and prompt assets.

## Scope: this file owns

- visible catalog list
- duplicate-ID checks
- built-in/global/deck prompt precedence
- origin reporting

## Out of scope: this file must not own

- runtime directory discovery of executable plugins
- prompt rendering
- language validation implementation

## Allowed dependencies

- config/path mechanics
- domain profile values
- application profile ports

## Forbidden dependencies and shortcuts

- inventory/linkme/self-registration
- other adapter crates

## Key implementation shape

```rust
const BUILT_INS: &[BuiltInProfile] = &[
    BuiltInProfile::new(
        "hindi",
        include_str!("../assets/profiles/hindi/profile.toml"),
        include_str!("../assets/profiles/hindi/prompts/import.md.hbs"),
        include_str!("../assets/profiles/hindi/prompts/build.md.hbs"),
    ),
];

pub fn built_in_catalog() -> Result<ProfileCatalog, ProfileError> {
    ProfileCatalog::try_from(BUILT_INS)
}
```

## Required tests / evidence

- duplicate IDs rejected
- unknown IDs rejected
- each prompt reports effective origin
- deleting a catalog line visibly removes support

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
