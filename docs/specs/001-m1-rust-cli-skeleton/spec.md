# M1 Rust CLI Skeleton

## Scope

Build the first Rust binary crate and implement one read-only command:
`hindi doctor`. The command discovers the project root, reports required
folders and prompt files, reports optional config status, checks Ollama service
reachability, and exits with a clear status.

## Problem

The project has moved its active plan to a Rust-first local-model workflow, but
there is not yet a Rust CLI. We need a tiny, safe first slice that proves the
binary shape and operator output without touching learner data.

## Goals

- Create one Rust binary crate for the future `hindi` CLI.
- Implement `hindi doctor`.
- Discover the project root from the current directory or a child directory.
- Report `input/`, `input/sentences/`, `input/words/`, `output/`, and `audio/`.
- Report `generation_prompt_sentences_enrichment.txt` and
  `generation_prompt_sentences.txt`.
- Report `hindi.toml` as optional.
- Check local Ollama service reachability without loading a model.
- Keep output calm, scannable, and actionable.
- Write no learner data.

## Non-Goals

- No `hindi sentences plan`.
- No sentence generation.
- No schema validation.
- No audio generation.
- No viewer/export command.
- No YAML ID migration.
- No source QA.
- No model switching.
- No Ollama model calls or model loading.
- No writes to `input/`, `output/`, `audio/`, `runs/`, or `exports/`.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | A Rust binary crate exists and builds. |
| AC02 | `cargo run -- doctor` prints a doctor report. |
| AC03 | Doctor discovers the project root from the root or a child directory. |
| AC04 | Doctor reports required data folders and prompt files as `ok` or `missing`. |
| AC05 | Missing `hindi.toml` is reported but does not fail the command. |
| AC06 | Missing required paths or prompts fail with exit code `1` after the full report prints. |
| AC07 | Ollama service reachability is checked without calling or loading a model. |
| AC08 | Unreachable Ollama fails with exit code `1` and prints a recovery hint. |
| AC09 | CLI usage errors are handled by the argument parser with exit code `2`. |
| AC10 | `hindi sentences plan` is not exposed in M1. |
| AC11 | `hindi doctor` writes no learner data. |

## Architecture Notes

See [architecture.md](architecture.md). The important boundary is simple:
commands parse and print; doctor/project modules own checks and root discovery.

### Files And Folders Changed

- `Cargo.toml`
- `Cargo.lock`
- `src/main.rs`
- `src/cli.rs`
- `src/doctor.rs`
- `src/project.rs`
- focused Rust tests, either inline or under `tests/`

### Workflow State Touched

None. M1 creates code only. It does not create, update, or derive learner-data
state.

### External Effects And Reuse

- Filesystem reads under the project root.
- One HTTP request to `http://localhost:11434/api/version`.
- Build/test artifacts under `target/`.
- No accepted-output writes.

## Testing Plan

See [testing.md](testing.md).

## Open Questions

- Development uses `cargo run -- doctor`; installed usage will eventually be
  `hindi doctor`.
- `hindi.toml` is the assumed config name, but M1 does not require it to exist.
