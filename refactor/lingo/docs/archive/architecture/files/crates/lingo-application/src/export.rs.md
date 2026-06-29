# `crates/lingo-application/src/export.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns Anki export selection, deck-name resolution, and typed export reporting.

## Scope: this file owns

- batch selection policy
- config/override deck name
- export material assembly

## Out of scope: this file must not own

- interactive picker UI
- Anki note/model encoding
- zip writing

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- AnkiExporter

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn export(deps: &ExportDeps<'_>, request: ExportRequest) -> Result<ExportReport, ExportError> {
    let batches = deps.workspace.load_selected_cards(&request.selection)?;
    let deck = request.deck.or_else(|| deps.context.export_deck()).ok_or(ExportError::MissingDeck)?;
    let material = deps.workspace.load_export_material(batches)?;
    Ok(ExportReport::from(deps.exporter.export(AnkiExport::new(deck, material))?))
}
```

## Required tests / evidence

- explicit batch, all, and picker-derived selections produce same material
- missing deck name gives recovery hint
- no terminal interaction in service

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
