# `crates/lingo-application/src/viewer.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns preparation of a read-only viewer session model; the CLI edge owns the HTTP server, browser launch, and frontend assets.

## Scope: this file owns

- viewer card projection facts
- display-policy resolution
- batch filtering

## Out of scope: this file must not own

- HTTP routing
- browser process
- Astro asset loading

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn prepare_viewer(deps: &ViewerDeps<'_>, request: ViewerRequest) -> Result<ViewerPlan, ViewerError> {
    let cards = deps.workspace.load_selected_cards(&request.selection)?;
    Ok(ViewerPlan::new(cards, deps.context.display()?))
}
```

## Required tests / evidence

- viewer plan contains no mutable workspace handle
- session lead override changes projection only
- canonical cards are never rewritten

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
