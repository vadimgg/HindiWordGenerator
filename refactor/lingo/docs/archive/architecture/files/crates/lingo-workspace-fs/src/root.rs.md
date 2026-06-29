# `crates/lingo-workspace-fs/src/root.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns workspace-root discovery and validation using `config.toml` as the root marker.

## Scope: this file owns

- absolute canonical root path
- discover/open/new-target distinction

## Out of scope: this file must not own

- directory layout paths
- config parsing
- current-directory global access

## Allowed dependencies

- std::fs/path

## Forbidden dependencies and shortcuts

- domain validation policy

## Key implementation shape

```rust
pub struct WorkspaceRoot(PathBuf);

impl WorkspaceRoot {
    pub fn discover(from: &Path) -> Result<Self, RootError> {
        for candidate in from.ancestors() {
            if candidate.join("config.toml").is_file() {
                return Ok(Self(candidate.to_path_buf()));
            }
        }
        Err(RootError::NotFound(from.to_path_buf()))
    }
}
```

## Required tests / evidence

- discovery stops at nearest marker
- fresh target path is not confused with open workspace
- symlink/canonicalization behavior is explicit

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
