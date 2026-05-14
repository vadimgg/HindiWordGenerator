# Architecture

## Summary

M1 creates a small Rust CLI foundation without introducing generation behavior.
The architecture risk is letting the first command become a grab bag of path
rules, service checks, and output formatting. Keep the command layer thin and
put project discovery and doctor checks behind small named modules.

## Module Ownership

| Module | Owns | Must Never |
|---|---|---|
| `src/main.rs` | Process entrypoint, top-level error handling. | Own project checks or formatting details. |
| `src/cli.rs` | Argument parsing and command enum. | Inspect the filesystem or call Ollama. |
| `src/project.rs` | Project-root discovery and project-relative path helpers. | Print user-facing output. |
| `src/doctor.rs` | Doctor checks, report data, and report rendering. | Write learner data or parse clap structs directly. |

This module split is a starting point. Keep it internal to one binary crate.
Do not create workspace crates in M1.

## Command Flow

### `hindi doctor`

User-facing development command:

```bash
cargo run -- doctor
```

Internal sequence:

```text
main.rs
  parse CLI
  dispatch doctor

cli.rs
  map args to Command::Doctor

project.rs
  find project root by walking upward
  require docs/DESIGN.md, docs/ROADMAP.md, input/, output/

doctor.rs
  check required folders
  check prompt files
  check optional hindi.toml
  check Ollama service reachability
  build DoctorReport
  render DoctorReport

main.rs
  exit 0 when required checks pass
  exit 1 when required checks fail
```

## Project Root Discovery

Start from the current directory and walk upward. Accept the first directory
that contains:

- `docs/DESIGN.md`
- `docs/ROADMAP.md`
- `input/`
- `output/`

If no root is found, return a typed error that renders as:

```text
Project not found

Run this command from HindiWordGenerator or one of its subdirectories.
```

## Data And Drift Risks

| Surface | Written By M1 | Read By M1 | Rule |
|---|---:|---|---|
| `input/` | no | doctor | Human-curated source. Report only. |
| `output/` | no | doctor | Accepted learner data. Report only. |
| `audio/` | no | doctor | Generated media. Report only. |
| `generation_prompt_sentences_enrichment.txt` | no | doctor | Required Rust prompt presence check. |
| `generation_prompt_sentences.txt` | no | doctor | Archived Python prompt presence check. |
| `hindi.toml` | no | doctor | Optional config presence check. |
| `target/` | cargo | cargo | Build artifacts only. |

### Drift Scenario: Doctor Creates Missing Folders

How it happens: implementation uses `create_dir_all` to make the report turn
green.

What breaks: a diagnostic command mutates project data surfaces and hides setup
problems.

Detection: tests for missing required paths assert the command reports failure
without creating the path.

Resolution: doctor is read-only; later setup/repair commands must be explicit.

### Drift Scenario: Model Check Loads A Model

How it happens: doctor calls `/api/generate` or shells out to `ollama run`.

What breaks: M1 spends model time, changes Ollama runtime state, and violates
the "model-aware, not lifecycle manager" policy.

Detection: Ollama check is isolated behind a function seam that tests can fake;
review rejects generation endpoints.

Resolution: use only `GET /api/version` in M1.

## Review Checklist

| Area | Reject | Accept |
|---|---|---|
| CLI layer | Filesystem checks or HTTP calls inside argument parsing. | Parse args and dispatch typed commands. |
| Doctor | Writes to project data folders. | Read-only checks and report rendering. |
| Root discovery | Hard-coded absolute repo path. | Upward search from current directory. |
| Ollama | Model load/generate call. | Cheap service reachability check. |
| Errors | Vague "failed" messages. | Message names the failed check and recovery. |
| Scope | `sentences plan` scaffold exposed as a runnable command. | Only `doctor` exposed in M1. |

## Files Removed Or Moved

None.

## Out-Of-Scope Residue

- YAML item IDs are handled by M1.5.
- Sentence planning is handled by M2.
- Config parsing beyond presence of `hindi.toml` is deferred.
