# M1 Rust CLI Skeleton

## What We're Doing

Create the first Rust CLI surface for Hindi Word Generator. M1 ships only
`hindi doctor`, a read-only command that checks whether the project is
recognizable and ready for later sentence-generation work.

This spec intentionally keeps the first slice small. It creates the Rust binary
shape, project-root discovery, a calm doctor report, and an Ollama service
reachability check. It does not plan batches, migrate YAML IDs, call a model,
or write learner data.

## Why Now

The active docs now describe a Rust-first local-model workflow, but the Rust
CLI does not exist yet. A read-only doctor command gives us a safe first
implementation step and a concrete foundation for later `hindi sentences plan`
and `hindi sentences generate`.

## What Changes

**Before:** The project has archived Python commands and Rust docs, but no Rust
binary.

**After:** The project has a Rust binary crate with `hindi doctor`, usable in
development as `cargo run -- doctor`.

## What Stays The Same

- YAML source remains under `input/`.
- Accepted learner data remains under `output/`.
- Audio remains under `audio/`.
- Archived Python remains reference material.
- No normal command writes accepted output in M1.
- Ollama model loading and model calls are deferred.

## What To Watch During Review

- `hindi doctor` must not create or repair missing data folders.
- Missing `hindi.toml` is reported but not fatal.
- Ollama reachability must not load a model.
- `hindi sentences plan` must not be exposed yet.
- CLI output should be specific enough that the user knows what failed and what
  to do next.

## Workflow Example

```bash
cargo run -- doctor
```

Expected sections:

```text
Hindi Word Generator

Project
Data
Prompts
Ollama
Next
```

## Where To Read More

| If you want to understand... | Read... |
|---|---|
| Exact scope and acceptance criteria | [spec.md](spec.md) |
| Module ownership and data-safety boundaries | [architecture.md](architecture.md) |
| Validation strategy | [testing.md](testing.md) |
| Command output and errors | [cli.md](cli.md) |
| Research notes | [research.md](research.md) |
| Implementation order | [plan.md](plan.md) |
| Work packages | [tasks.md](tasks.md) |
