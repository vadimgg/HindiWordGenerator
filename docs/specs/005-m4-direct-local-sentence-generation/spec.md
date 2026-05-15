# M4 Direct Local Sentence Generation

## Scope

Add the first real Rust sentence-generation command:

```bash
hindi sentences generate --max-batches 1
```

The command should plan pending sentence work, call one configured local Ollama
model for enrichment only, merge trusted YAML source fields with model
enrichment, validate through the M3 validator, write accepted output through the
M3 writer, and record a run report. M4 uses one model role:
`sentence_generation`.

## Problem

The Rust CLI can now inspect pending work and validate candidate sentence JSON,
but it cannot generate accepted cards. The next useful slice is one small local
model batch that proves the full trusted-source -> Ollama enrichment ->
validator -> accepted-output path without adding source QA, model switching, or
review/accept ceremony.

## Goals

- Add `hindi sentences generate --max-batches <n>`.
- Read `[models].sentence_generation` from `hindi.toml`, defaulting to
  `ollama:translategemma:12b` when config is absent.
- Check Ollama readiness before model calls.
- If Ollama or the configured model is not ready, stop before generation and
  print the exact `ollama run ...` command.
- Call Ollama through its local HTTP API; do not spawn/stop Ollama and do not
  switch models.
- Build prompt input from planned YAML source rows.
- Send the enrichment prompt from `generation_prompt_sentences_enrichment.txt`.
- Extract JSON from raw model responses, tolerating markdown fences and
  leading/trailing prose.
- Merge model enrichment with trusted source fields owned by Rust.
- Validate merged batches using M3 validation.
- Write accepted output using M3 atomic writer.
- Write a minimal run report under `runs/sentences/` for both accepted and
  failed validation runs.
- Keep `hindi sentences plan` read-only.

## Non-Goals

- No source QA model.
- No CLI-managed Ollama lifecycle, unloading, RAM management, or model switching.
- No multi-model orchestration.
- No `--review`, `review`, or `accept` run-folder workflow.
- No word-card generation.
- No audio generation/backfill.
- No Anki export.
- No source repair.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi sentences generate --max-batches <n>` is exposed in CLI parsing and help. |
| AC02 | `--max-batches` remains a positive integer and means total output files, not source rows. |
| AC03 | Generation reuses the planner’s pending source selection and target filename logic. |
| AC04 | Generation fails before model calls if the planner reports source/output errors. |
| AC05 | Config reader supports `[models].sentence_generation`; absent config defaults to `ollama:translategemma:12b`. |
| AC06 | Ollama provider prefix `ollama:<model>` is parsed; unsupported providers fail clearly. |
| AC07 | Ollama readiness checks local API reachability and model availability before generation. |
| AC08 | If model/API is not ready, output prints `ollama run <model>` and writes no accepted output. |
| AC09 | The CLI never spawns `ollama`, stops models, or switches models automatically. |
| AC10 | Prompt payload sends source row `id`, `hindi`, `romanisation`, `english`, and optional `tags`; title/subtitle/source_ref are not trusted to the model. |
| AC11 | Model response extraction accepts raw JSON and fenced JSON. |
| AC12 | Model enrichment must be keyed by source row ID. |
| AC13 | Rust copies trusted `title`, `subtitle`, `hindi`, `romanisation`, `english`, source tags, `source_ref`, fingerprint, and target filename from source/planner data. |
| AC14 | Rust merges only `literal`, `register`, `tokens`, `words`, and optional `anki_tags` from the model. |
| AC15 | Merged output is validated with the M3 validator before any accepted write. |
| AC16 | Accepted output is written through the M3 writer and refuses collisions. |
| AC17 | If validation fails, no accepted output file is written. |
| AC18 | Run reports are written under `runs/sentences/` for accepted and failed attempts. |
| AC19 | Run reports include command, status, source files, targets, prompt path/fingerprint, model string, best available Ollama metadata, timings, validation result, and accepted/skipped writes. |
| AC20 | User-facing output summarizes planned batches, model used, target files, accepted/skipped writes, and next step. |
| AC21 | `hindi sentences plan --max-batches 1` output stays compatible and read-only. |
| AC22 | Protected paths are respected during failure tests: no writes under real `input/`, `output/`, or `audio/`; `runs/` is only written by generation attempts. |

## Architecture Notes

[architecture.md](architecture.md) owns module boundaries and write ordering.
The important decision is that M4 is model-aware, not model-managing. The user
starts Ollama separately. The CLI talks to the local Ollama API and gives clear
recovery instructions when the configured model is not ready.

### Files And Folders Changed

- `Cargo.toml`
- `Cargo.lock`
- `src/cli.rs`
- `src/main.rs`
- `src/sentence_plan.rs`
- `src/sentence_generate.rs` or equivalent orchestration module
- `src/ollama.rs` or equivalent provider module
- `src/config.rs` or equivalent config reader
- `src/run_report.rs` or equivalent report writer
- `docs/ROADMAP.md`
- `docs/specs/005-m4-direct-local-sentence-generation/**`

### Workflow State Touched

- Brief spec/task files under this spec.
- `runs/sentences/` is created/written by generation attempts.
- `output/sentences/` is written only after successful validation.

### External Effects And Reuse

- HTTP calls to local Ollama API at `http://localhost:11434`.
- No network calls beyond localhost.
- No shelling out to `ollama run`, `ollama ps`, `ollama stop`, or model
  switching commands.
- Reuse M2 planner and M3 validator/writer.

## Testing Plan

### Unit Tests

- CLI parses `sentences generate --max-batches <n>`.
- Config reader default and explicit model cases.
- Model string parsing accepts `ollama:name` and rejects unsupported providers.
- Ollama response extraction handles raw JSON, fenced JSON, and prose-wrapped
  JSON.
- Prompt payload excludes trusted fields that Rust owns.
- Merge copies trusted source fields and only accepts enrichment fields from the
  model.
- Validation failure prevents accepted writes.
- Run report serialization includes required fields.

### Integration Tests

- Fake Ollama client happy path generates one temp output batch.
- Fake Ollama not-ready path prints recovery and writes no accepted output.
- Fake Ollama invalid JSON path writes failed run report and no accepted output.
- Collision path refuses accepted write.

### Smoke Tests

- `cargo fmt`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run -- sentences plan --max-batches 1`
- `cargo run -- sentences generate --max-batches 1` with Ollama running
  `translategemma:12b` when available.

### Drift / Consistency Checks

- `git diff --name-only -- input audio` prints nothing.
- `output/sentences/` changes only after an intentional successful generation
  smoke test.
- `runs/sentences/` contains run reports for generation attempts.

### Not Covered In This Spec

- Translation quality evaluation beyond whether the batch validates. Human/model
  quality review remains manual for this first slice.
- Multi-model source QA is later.
- Audio and export are later.

## Open Questions

None for the spec. The implementation may tune exact prompt wording while
keeping the prompt contract unchanged.
