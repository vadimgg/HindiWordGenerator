# Review

## Summary

M1.5 added a Rust `source ids` maintenance surface and migrated active source
YAML to stable file-scoped item IDs. The migration added IDs to 710 items across
13 active YAML files.

## Validation

- `cargo fmt`
- `cargo test`
- `cargo run -- source ids check`
- `cargo run -- source ids migrate --check`
- `git diff --check`
- `git diff --name-only -- output audio runs archive/python/legacy-input`

## Changed Files

- `src/cli.rs`
- `src/main.rs`
- `src/source_ids.rs`
- `input/sentences/*.yaml`
- `input/words/*.yaml`
- `docs/ROADMAP.md`
- `docs/specs/002-m1-5-yaml-id-migration/**`

## Follow-Ups

- M2 should consume these IDs in `hindi sentences plan`.
- Old output remains lineage-less by design; M2 should report it as
  `missing lineage`.
