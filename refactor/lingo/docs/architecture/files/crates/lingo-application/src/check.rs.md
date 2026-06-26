# `crates/lingo-application/src/check.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the deterministic read-only quality gate and aggregation of domain diagnostics across one or more card batches.

## Scope: this file owns

- batch selection
- validation invocation
- summary counts
- exit-relevant clean/problem result

## Out of scope: this file must not own

- repairing IDs
- rewriting cards
- terminal formatting
- model grading

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- domain validation

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn check(deps: &CheckDeps<'_>, request: CheckRequest) -> Result<CheckReport, CheckError> {
    let batches = deps.workspace.select_card_batches(request.batch.as_ref())?;
    let mut report = CheckReport::default();
    for cards in batches {
        let source = deps.workspace.load_source(cards.batch_id())?;
        report.add(cards.batch_id().clone(), check_card_batch(&cards, &source, &deps.context.profile()?));
    }
    Ok(report)
}
```

## Required tests / evidence

- command is provably read-only
- one bad batch does not hide diagnostics from other batches
- problem result maps to exit code 1 at CLI edge

## Design notes

- Clean-slate v1 does not include `--fix-ids`; mutation must be a separately named command if ever needed.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
