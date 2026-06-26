# `crates/lingo-application/src/package.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns selection and preconditions for publishing a portable package; the artifact adapter owns format and staging mechanics.

## Scope: this file owns

- batch selection
- missing-audio gate
- package request assembly
- typed package report

## Out of scope: this file must not own

- manifest JSON shape
- checksum implementation
- directory swaps

## Allowed dependencies

- WorkspaceStore
- DeckContextProvider
- PackagePublisher

## Forbidden dependencies and shortcuts

- clap arguments
- ANSI output
- concrete adapter types
- direct filesystem/process/HTTP calls

## Key implementation shape

```rust
pub fn package(deps: &PackageDeps<'_>, request: PackageRequest) -> Result<PackageReport, PackageError> {
    let material = deps.workspace.load_publishable_material(request.batch.as_ref())?;
    if material.has_missing_audio() { return Err(PackageError::MissingAudio(material.missing_audio())); }
    let published = deps.publisher.publish(PublishPackage::new(material, deps.context.display()?))?;
    Ok(PackageReport::published(published))
}
```

## Required tests / evidence

- missing audio blocks publication
- batch filter is respected
- artifact writer receives fully typed material

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
