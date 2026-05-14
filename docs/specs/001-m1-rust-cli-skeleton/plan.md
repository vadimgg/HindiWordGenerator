# Plan

## Design

Build the smallest Rust command that proves the CLI shell and project
inspection path. The command parses `doctor`, discovers the project root,
runs read-only checks, renders a report, and exits with a simple status code.

## Modules

| Module | Responsibility |
|---|---|
| `src/main.rs` | Entrypoint, dispatch, top-level exit behavior. |
| `src/cli.rs` | `clap` parser and command enum. |
| `src/project.rs` | Project-root discovery and relative path helpers. |
| `src/doctor.rs` | Doctor checks, Ollama reachability seam, report rendering. |

## Operation Order

1. Parse CLI args.
2. Discover project root.
3. Build the list of required and optional checks.
4. Check paths and prompts.
5. Check Ollama service reachability with `/api/version`.
6. Render the full report.
7. Exit `0` if all required checks pass; otherwise exit `1`.

The point of no return is only process exit. `hindi doctor` has no write phase.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Create the Rust CLI skeleton and `doctor` command. |
| WP02 | Add focused tests and validation coverage. |
| WP03 | Review docs/spec alignment and prepare M1 for implementation closeout. |

## Risks

| Risk | Mitigation |
|---|---|
| Doctor mutates project data. | Keep checks read-only and test missing-path behavior. |
| Command layer grows business logic. | Keep root discovery in `project.rs` and checks in `doctor.rs`. |
| Ollama check loads a model. | Use `/api/version` only and isolate the checker. |
| M2 command leaks into M1. | Add CLI/help assertion that `sentences plan` is unavailable. |

## Validation

- `cargo fmt`
- `cargo test`
- `cargo run -- doctor`
- `python3 archive/python/scripts/check-agent-workflows.py`
- `uv run python archive/python/scripts/check-python-contracts.py`
- `git diff --check`
