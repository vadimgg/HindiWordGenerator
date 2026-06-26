# `crates/lingo-workspace-fs/src/scan.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Derives a workspace snapshot by reading canonical files and checking audio presence; it reports facts only.

## Scope: this file owns

- sorted discovery
- snapshot facts
- orphan and missing-file observations

## Out of scope: this file must not own

- next-action policy
- terminal labels
- mutating or repairing files

## Allowed dependencies

- layout
- codecs
- domain progress values

## Forbidden dependencies and shortcuts

- writing indexes as authority

## Key implementation shape

```rust
pub fn scan_workspace(workspace: &FsWorkspace) -> Result<WorkspaceSnapshot, ScanError> {
    let raw = scan_raw(workspace.layout())?;
    let sources = scan_sources(workspace.layout())?;
    let cards = scan_cards(workspace.layout())?;
    Ok(WorkspaceSnapshot::new(raw, sources, cards))
}
```

## Required tests / evidence

- deterministic ordering
- malformed canonical file reported as a problem, not skipped
- snapshot can be rebuilt from files

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
