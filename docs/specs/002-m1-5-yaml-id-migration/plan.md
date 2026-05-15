# Plan

## Design

Add a small Rust source-ID domain that can inspect and migrate active YAML
source files. The CLI stays thin: it exposes `source ids check` and `source ids
migrate`, while the domain owns YAML parsing, ID validation, allocation, and
write planning.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `source ids check`, `source ids migrate`, and `--check`. |
| `src/main.rs` | Dispatch commands, print reports, and return exit codes. |
| `src/project.rs` | Reuse project root discovery. |
| `src/source_ids.rs` or `src/source.rs` | Source YAML discovery, validation, ID allocation, rendering, and writes. |

## Operation Order

1. Discover the project root.
2. Find active YAML files under `input/sentences/` and `input/words/`.
3. Parse all source files and collect items.
4. Validate existing IDs for shape and per-file uniqueness.
5. If validation fails, print blocking errors and write nothing.
6. Allocate missing IDs in memory, preserving existing IDs and item order.
7. In `--check` mode, print planned changes and write nothing.
8. In migration mode, write only changed active YAML files.
9. Print changed files and the next check command.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Finalize source ID rules, CLI shape, and YAML fixture expectations. |
| WP02 | Implement source ID validation/allocation and tests. |
| WP03 | Wire CLI commands, run migration on active YAML, and update roadmap status. |
| WP04 | Review, validate, and capture follow-ups. |

## Risks

| Risk | Mitigation |
|---|---|
| Existing IDs are rewritten. | Treat existing IDs as authority and add idempotency tests. |
| Migration writes outside source YAML. | Centralize write targets and verify protected paths stay unchanged. |
| YAML formatting churn hides data changes. | Keep rendering stable and manually inspect representative source files. |
| The command becomes normal workflow surface too early. | Document it as a one-off source maintenance helper. |

## Validation

- `cargo fmt`
- `cargo test`
- `cargo run -- source ids check`
- `cargo run -- source ids migrate --check`
- `cargo run -- source ids migrate`
- `cargo run -- source ids check`
- `python3 archive/python/scripts/check-agent-workflows.py`
- `uv run python archive/python/scripts/check-python-contracts.py`
- `git diff --check`
