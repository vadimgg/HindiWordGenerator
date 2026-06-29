# `crates/lingo-workspace-fs/src/layout.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns every stable workspace-relative directory and filename convention behind semantic methods.

## Scope: this file owns

- raw/input/output/audio/runs/packages/exports paths
- batch filename construction
- safe path joining

## Out of scope: this file must not own

- creating directories
- checking card validity
- package-internal paths

## Allowed dependencies

- WorkspaceRoot
- typed IDs

## Forbidden dependencies and shortcuts

- scattered string joins in callers

## Key implementation shape

```rust
pub struct WorkspaceLayout { root: WorkspaceRoot }

impl WorkspaceLayout {
    pub fn source_file(&self, batch: &BatchId) -> PathBuf {
        self.root.join("input/sentences").join(format!("{}.yaml", batch.as_str()))
    }

    pub fn card_file(&self, batch: &BatchId) -> PathBuf {
        self.root.join("output/sentences").join(format!("{}.json", batch.as_str()))
    }
}
```

## Required tests / evidence

- all paths remain inside root
- stable path vocabulary test
- batch ID cannot inject separators

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
