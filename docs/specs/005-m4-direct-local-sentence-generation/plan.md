# Plan

## Design

Add one generation orchestration path that reuses the planner and validator
instead of inventing a parallel flow. The command gathers pending source rows,
checks the configured Ollama model, sends an enrichment prompt, extracts
enrichment JSON, merges it with trusted source data, validates, writes accepted
JSON atomically, and records a run report.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `sentences generate --max-batches <n>` and help text. |
| `src/config.rs` | Read `hindi.toml` and resolve `[models].sentence_generation` with default. |
| `src/ollama.rs` | Local Ollama HTTP client, model readiness check, generate call, model metadata best-effort lookup. |
| `src/sentence_plan.rs` | Expose planned source rows and target paths for generation reuse. |
| `src/sentence_generate.rs` | Orchestrate plan -> prompt -> model -> merge -> validate -> write -> report. |
| `src/sentence_schema.rs` | Candidate accepted-batch structs from M3. |
| `src/sentence_validate.rs` | Validate merged candidate batch. |
| `src/accepted_writer.rs` | Write accepted output atomically. |
| `src/run_report.rs` | Serialize run reports under `runs/sentences/`. |

## Operation Order

1. Parse CLI args.
2. Discover project root.
3. Load config; resolve `sentence_generation` model, defaulting to
   `ollama:translategemma:12b`.
4. Build sentence plan from current YAML and existing output.
5. Stop if planner reports errors or no planned batches.
6. Check Ollama readiness:
   - local API reachable;
   - configured model available/responding;
   - unsupported provider rejected.
7. Read and fingerprint `generation_prompt_sentences_enrichment.txt`.
8. Build prompt payload from planned source rows only.
9. Call Ollama local HTTP API.
10. Extract enrichment JSON from model response.
11. Merge enrichment into trusted source-owned batch data.
12. Validate merged batch through M3 validator.
13. If validation fails, write failed run report and no accepted output.
14. If validation passes, write accepted output through M3 writer.
15. Write accepted run report.
16. Print summary and next step.

Point of no return: accepted-output writer rename. Everything before that can
fail with no accepted output mutation. Run report writes are diagnostic and may
exist for failed attempts.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP01 | Review the M4 generation contract, model policy, and current M2/M3 reusable APIs. |
| WP02 | Implement config/model parsing and Ollama client/readiness boundary. |
| WP03 | Implement prompt payload builder, response extraction, and enrichment merge. |
| WP04 | Wire `hindi sentences generate` through planner, Ollama, validator, and writer. |
| WP05 | Add run reports and user-facing generation output. |
| WP06 | Review generation safety, failure behavior, protected paths, docs, and closeout notes. |

## Risks

| Risk | Mitigation |
|---|---|
| CLI starts managing Ollama lifecycle too early. | M4 only prints recovery commands and uses local HTTP API. |
| Model rewrites trusted source fields. | Prompt excludes trusted fields; merge ignores them even if model returns them. |
| Invalid model response writes accepted output. | Extract -> merge -> validate all happen before writer call. |
| Planner and generator choose different targets. | Expose/reuse planner rows and target data for generation. |
| Run reports become source of truth. | Reports are diagnostic; output remains accepted-card authority. |

## Validation

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- sentences generate --max-batches 1` with Ollama running when
  available.
- `git diff --name-only -- input audio`
- `git diff --check`
