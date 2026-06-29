# `crates/lingo-artifacts/src/model.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns artifact-internal plans and safe relative paths used by both portable packages and Anki exports.

## Scope: this file owns

- publication plan
- artifact file entry
- safe relative output path
- deterministic ordering

## Out of scope: this file must not own

- canonical card mutation
- workspace paths

## Allowed dependencies

- domain card/audio values

## Forbidden dependencies and shortcuts

- raw path strings accepted without validation

## Key implementation shape

```rust
pub(crate) struct ArtifactFile {
    pub path: ArtifactPath,
    pub bytes: Vec<u8>,
}

pub(crate) struct PublicationPlan {
    pub files: Vec<ArtifactFile>,
}
```

## Required tests / evidence

- duplicate output paths rejected
- paths cannot escape artifact root
- ordering deterministic

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
