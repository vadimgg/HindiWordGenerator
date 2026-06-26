# `crates/lingo-application/src/build.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the two-phase source-to-card enrichment workflow and the acceptance boundary for canonical card output.

## Scope: this file owns

- pending-batch selection
- prompt context from source/profile
- card draft acceptance
- lineage validation before write
- typed build reports

## Out of scope: this file must not own

- editor/clipboard
- JSON fence stripping
- card file encoding
- quality rendering

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- PromptEngine
- RunJournal
- domain validation

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn apply_build(
    deps: &BuildDeps<'_>,
    request: ApplyBuild,
) -> Result<BuildReport, BuildError> {
    let source = deps.workspace.load_source(&request.batch)?;
    let draft = deps.prompts.parse_build_reply(&request.reply)?;
    let cards = draft.accept(&source, &deps.context.profile()?)?;
    let report = check_card_batch(&cards, &source, &deps.context.profile()?);
    if !report.is_clean() { return Err(BuildError::Rejected(report)); }
    let stored = deps.workspace.create_cards(&cards)?;
    Ok(BuildReport::accepted(&cards, stored))
}
```

## Required tests / evidence

- missing or extra source lineage rejected
- invalid card output never reaches storage
- print-only mode writes no canonical cards

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
