# `crates/lingo-workspace-fs/src/runs.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns advisory run-journal files for prepared prompt packets, pasted replies, and diagnostics. These files support reproducibility but are never canonical learning data.

## Scope: this file owns

- run directory naming
- prompt/reply/diagnostic records
- prepared/applied status facts

## Out of scope: this file must not own

- deciding card validity
- using run files as source of truth
- editor operations

## Allowed dependencies

- layout
- atomic writer
- clock/run IDs from application request

## Forbidden dependencies and shortcuts

- canonical source/card mutation through run files

## Key implementation shape

```rust
pub fn record_prepared(&self, run: &PreparedRun) -> Result<RunRecord, RunStoreError> {
    let dir = self.layout.run_dir(run.stage(), run.id());
    create_dir_all(&dir)?;
    create_atomic(&dir.join("prompt.md"), run.packet().as_bytes())?;
    create_atomic(&dir.join("meta.json"), &encode_meta(run)?)?;
    Ok(RunRecord::prepared(run.id().clone(), dir))
}
```

## Required tests / evidence

- run files may be deleted without losing canonical data
- applied record references canonical output
- collisions do not merge two runs

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
