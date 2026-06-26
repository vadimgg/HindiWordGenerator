# `crates/lingo-artifacts/src/package.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Builds and publishes a self-contained portable package from fully selected card/audio material.

## Scope: this file owns

- card JSON/JSONL projection
- audio path rewrite
- README text
- manifest assembly
- publication order

## Out of scope: this file must not own

- workspace scans
- missing-audio policy
- terminal output

## Allowed dependencies

- manifest
- checksum
- staging
- application package port

## Forbidden dependencies and shortcuts

- references outside package root

## Key implementation shape

```rust
impl PackagePublisher for PortablePackagePublisher {
    fn publish(&self, request: PublishPackage) -> Result<PublishedPackage, ArtifactFailure> {
        let plan = build_package_plan(request)?;
        self.staging.publish_directory(&plan.destination, |stage| {
            write_payload_files(stage, &plan)?;
            let manifest = build_and_verify_manifest(stage, &plan)?;
            write_manifest_last(stage, &manifest)
        })?;
        Ok(plan.published())
    }
}
```

## Required tests / evidence

- all audio references rewritten inside root
- manifest written last
- read-back verification before swap
- deterministic JSONL order

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
