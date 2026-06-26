# `crates/lingo-artifacts/src/staging.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns atomic directory publication for packages and export folders.

## Scope: this file owns

- sibling temp directory
- complete-build callback
- destination replacement policy
- cleanup

## Out of scope: this file must not own

- artifact format
- workspace canonical writes

## Allowed dependencies

- std::fs

## Forbidden dependencies and shortcuts

- publishing partial destination
- deleting prior destination before replacement is ready

## Key implementation shape

```rust
pub fn publish_directory(
    destination: &Path,
    build: impl FnOnce(&Path) -> Result<(), ArtifactError>,
) -> Result<(), ArtifactError> {
    let stage = unique_sibling_dir(destination)?;
    build(&stage)?;
    verify_stage(&stage)?;
    swap_directory(&stage, destination)
}
```

## Required tests / evidence

- failed build leaves existing destination untouched
- temp directory cleaned
- replacement is explicit

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
