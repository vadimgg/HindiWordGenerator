# Lingo Rust refactor direction

This workspace is a pre-refactor Rust baseline. The files under `docs/` describe
the intended sentence-centric SQLite refactor direction; the current code has not
fully implemented that direction yet.

The target design makes `library.db` the canonical runtime state. JSON packages,
study packages, and Anki packages are derived publishers, not source-of-truth
files.

The CLI is the first product surface for the refactor. The viewer is deferred
until it can call the same application use cases as the CLI; do not treat the
current prototype viewer as part of the Phase 1 contract.

Documentation:

- `docs/CLI.md` — CLI reference hub (per-command pages under `docs/cli/`)
- `docs/workflows.md` — how the tool is used end to end
- `docs/package-and-agents.md` — on-disk layout, exports, and coding-agent flow
- `docs/schema.md` — prose schema overview; exact SQL lives in `docs/arch/schema_v02.sql`
- `docs/pending-decisions.md` — reconciliation notes + queued implementation checks

Run:

```bash
cargo test --workspace --all-targets
cargo run -p lingo-cli --bin lingo -- --help
```
