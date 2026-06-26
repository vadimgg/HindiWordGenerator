# `crates/lingo-application/src/init.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the policy for creating a new workspace safely and idempotently around a selected language profile.

## Scope: this file owns

- first-init requirements
- create-missing-only behavior
- typed init report and next action

## Out of scope: this file must not own

- directory creation mechanics
- TOML serialization details
- prompt editing

## Allowed dependencies

- WorkspaceBootstrap port
- ProfileCatalog port
- Clock only if state timestamps are added

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn init(
    workspace: &dyn WorkspaceBootstrap,
    profiles: &dyn ProfileCatalog,
    request: InitRequest,
) -> Result<InitReport, InitError> {
    let profile = profiles.require(&request.profile)?;
    let changes = workspace.create_missing(&request.target, &profile)?;
    Ok(InitReport { profile: profile.summary(), changes, next: NextAction::Import { raw: None } })
}
```

## Required tests / evidence

- fresh workspace requires a profile
- re-run preserves existing config and data
- unknown profile is actionable

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
