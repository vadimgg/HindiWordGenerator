# `crates/lingo-workspace-fs/src/atomic_file.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns durable create-only and replace-existing file mechanics: sibling temp file, flush, sync, and rename.

## Scope: this file owns

- atomic create/replace primitives
- temp cleanup
- parent existence checks
- collision behavior

## Out of scope: this file must not own

- serialization
- domain naming
- directory publication

## Allowed dependencies

- std::fs/io

## Forbidden dependencies and shortcuts

- silent overwrite in create mode
- timestamp-only temp uniqueness without create_new

## Key implementation shape

```rust
pub fn create_atomic(target: &Path, bytes: &[u8]) -> Result<(), AtomicFileError> {
    require_existing_parent(target)?;
    if target.exists() { return Err(AtomicFileError::Collision(target.into())); }
    let temp = unique_sibling(target);
    write_new_and_sync(&temp, bytes)?;
    fs::rename(&temp, target).map_err(|e| cleanup_and_wrap(temp, target, e))
}
```

## Required tests / evidence

- atomic create
- collision preserves original
- failed rename cleans temp
- missing parent is explicit

## Design notes

- This deliberately carries forward the strongest mechanic from the attached writer code.

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
