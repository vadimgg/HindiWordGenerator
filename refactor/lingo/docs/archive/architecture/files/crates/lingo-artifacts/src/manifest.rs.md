# `crates/lingo-artifacts/src/manifest.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns the versioned `lingo.package/v1` manifest vocabulary and maps publication facts into deterministic JSON.

## Scope: this file owns

- format tag
- language/display/count/group/file/integrity fields
- stable wire names

## Out of scope: this file must not own

- workspace authority
- package selection policy
- checksumming mechanics

## Allowed dependencies

- domain values
- checksum results

## Forbidden dependencies and shortcuts

- ad hoc JSON maps

## Key implementation shape

```rust
#[derive(Serialize)]
pub(crate) struct PackageManifest {
    format: &'static str,
    language: ManifestLanguage,
    display: ManifestDisplay,
    counts: ManifestCounts,
    groups: Vec<ManifestGroup>,
    files: ManifestFiles,
    integrity: ManifestIntegrity,
}
```

## Required tests / evidence

- golden JSON structure
- format tag exactly `lingo.package/v1`
- all listed files exist and hash correctly

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
