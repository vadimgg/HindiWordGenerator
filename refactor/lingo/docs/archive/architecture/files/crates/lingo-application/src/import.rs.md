# `crates/lingo-application/src/import.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the two-phase raw-text-to-source workflow: prepare a prompt packet, then accept a pasted reply into canonical source data.

## Scope: this file owns

- raw selection policy
- prompt context construction
- reply acceptance sequence
- stable item-ID assignment
- typed import reports

## Out of scope: this file must not own

- clipboard/editor operations
- Handlebars syntax
- YAML parsing mechanics
- atomic file mechanics

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- PromptEngine
- RunJournal
- domain source constructors

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn apply_import(
    deps: &ImportDeps<'_>,
    request: ApplyImport,
) -> Result<ImportReport, ImportError> {
    let prepared = deps.runs.require_prepared(&request.run_id)?;
    let draft = deps.prompts.parse_import_reply(&request.reply)?;
    let source = draft.accept(prepared.batch_id(), prepared.profile())?;
    let stored = deps.workspace.create_source(&source)?;
    deps.runs.record_applied(&request.run_id, &request.reply, &stored)?;
    Ok(ImportReport::accepted(&source, stored))
}
```

## Required tests / evidence

- print-only preparation writes no canonical source
- malformed reply never mutates input
- duplicate source text gets deterministic distinct IDs
- existing canonical source collision is explicit

## Design notes

- No retry loop parses and repairs silently. Diagnostics go back to the user; the user edits and reapplies.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
