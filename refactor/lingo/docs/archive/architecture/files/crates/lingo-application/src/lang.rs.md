# `crates/lingo-application/src/lang.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns profile discovery, inspection, prompt-origin reporting, and creation of explicit global/deck prompt overrides.

## Scope: this file owns

- list/show/which/edit decisions
- override target selection
- typed profile reports

## Out of scope: this file must not own

- editor process
- TOML/path mechanics
- hidden registration

## Allowed dependencies

- ProfileCatalog
- DeckContextProvider
- ProfileOverrideStore

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn create_prompt_override(
    profiles: &dyn ProfileOverrideStore,
    request: EditPromptRequest,
) -> Result<EditPromptReport, LangError> {
    let target = match request.scope {
        OverrideScope::Global => profiles.global_override_path(&request.profile, request.stage)?,
        OverrideScope::Deck => profiles.deck_override_path(request.stage)?,
    };
    let created = profiles.create_from_resolved_if_missing(&target, request.stage)?;
    Ok(EditPromptReport { target, created })
}
```

## Required tests / evidence

- built-in catalog duplicate IDs rejected
- unknown profile fails loudly
- edit never overwrites an existing override

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
