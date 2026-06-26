# `crates/lingo-application/src/status.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the read-only pipeline dashboard facts and the single best next action derived from a workspace snapshot.

## Scope: this file owns

- status filtering
- progress derivation
- problem classification
- next-action choice

## Out of scope: this file must not own

- filesystem scan mechanics
- ANSI table output
- persisting status

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider when validation needs profile

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn status(deps: &StatusDeps<'_>, request: StatusRequest) -> Result<StatusReport, StatusError> {
    let snapshot = deps.workspace.scan()?;
    let rows = derive_progress(snapshot, request.filter, deps.context.profile()?);
    let next = choose_next_action(&rows);
    Ok(StatusReport { rows, next })
}
```

## Required tests / evidence

- next action priority is deterministic
- `--problems` filtering does not change underlying counts
- status performs no writes

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
