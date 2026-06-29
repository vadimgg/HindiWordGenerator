# Lingo Rust refactor

This workspace implements the sentence-centric SQLite library described in the refactor docs. The Astro viewer/UI has intentionally been left out of this phase; the CLI and Rust crates are the product surface.

The canonical runtime state is `library.db`. JSON packages and Anki packages are derived publishers, not source-of-truth files.

Documentation:

- `docs/CLI.md` — CLI reference hub (per-command pages under `docs/cli/`)
- `docs/workflows.md` — how the tool is used end to end
- `docs/package-and-agents.md` — on-disk layout, exports, and coding-agent flow
- `docs/schema.md` — the `library.db` schema
- `docs/pending-decisions.md` — open schema/architecture decisions + queued doc work

Run:

```bash
cargo test --workspace --all-targets
cargo run -p lingo-cli --bin lingo -- --help
```
