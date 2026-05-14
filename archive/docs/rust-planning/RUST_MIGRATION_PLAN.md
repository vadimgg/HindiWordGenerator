# Rust Migration Plan

The Rust rewrite starts with sentence generation. Python has already moved to
`archive/python/` and remains the behavior reference until Rust owns the normal
workflow.

## Current Status

| Area | Status | Notes |
|---|---|---|
| Python archived | Done | Runtime, scripts, tests, experiments, and manifest live under `archive/python/`. |
| YAML source input | Done | Active `input/words/` and `input/sentences/` files are YAML-only. |
| Archived Python YAML support | Done | Archived Python reads project-level YAML input and writes project-level output/audio. |
| Rust CLI | Pending | No Rust command surface exists yet. |
| Sentence planner | Pending | First Rust feature after the CLI skeleton. |
| Local generation | Pending | Planned around Ollama, starting with `translategemma:12b` for sentence generation. Optional source QA uses `gemma4:latest`. |

## Current Reference Commands

Use these commands to compare Rust behavior while building:

```bash
uv run archive/python/runtime/main.py check --type sentences --max-batches 1
uv run archive/python/runtime/main.py run --type sentences --max-batches 1
uv run archive/python/runtime/main.py audio --type sentences
python3 archive/python/scripts/check-python-contracts.py
```

## Migration Rules

- Preserve pending/skipped decisions, output filenames, schema validation, and
  append-only write safety.
- Use YAML as the only active source input format.
- Keep `output/` as the completed-card authority.
- Keep run metadata outside accepted card JSON.
- Add durable source lineage to accepted sentence cards when Rust generation
  starts: `source_ref.file`, `source_ref.item_id`, and
  `source_ref.fingerprint`.
- Keep viewer-compatible JSON at every accepted generation milestone.

## Milestone Checklist

### M0: Python Archive And YAML Migration

Status: Done

Acceptance:

- Python runtime is under `archive/python/runtime/`.
- Legacy input files are under `archive/python/legacy-input/`.
- Active input files are `.yaml`.
- Archived Python check commands read YAML successfully.

### M1: Rust CLI Skeleton

Status: Pending

Workspace crates:

```text
crates/hindi-cli
crates/hindi-config
crates/hindi-core
crates/hindi-models
```

Commands:

```bash
hindi doctor
hindi config show
hindi models status
hindi models prepare <workflow>
hindi sentences check
hindi runs list
```

Goal: a polished command surface with no generation.

Acceptance:

- Rust workspace exists with separate CLI, config, core, and model-provider
  crates.
- `hindi-cli` owns argument parsing and terminal output.
- `hindi-models` owns Ollama discovery/check/switch behavior.
- `doctor` reports project root, required folders, prompt availability, and
  Ollama reachability.
- `config show` reports effective model/runtime config without exposing secrets.
- `models status` reports model-runtime readiness in user-facing workflow terms.
- `models prepare <workflow>` reports the configured runtime for a workflow and
  refuses to switch without confirmation unless `--allow-model-switch` is set.
- `sentences check` may print a clearly labelled M1 stub that says planner
  support arrives in M2.
- `runs list` may print a clearly labelled M1 stub that says run management
  arrives with generation/review milestones.
- No command writes learner-facing data.

### M2: Sentence Planner

Status: Pending

Add or complete:

```text
crates/hindi-source
crates/hindi-planner
```

Read `input/sentences/*.yaml`, parse title/subtitle/items, detect completed
cards from `output/sentences/`, and print planned work.

Acceptance:

- Planner output matches the archived Python check command for current YAML
  inputs.
- `--max-batches` is total across the command invocation, not per source file.
- `sentences check` writes nothing; generation re-derives pending work from
  current source/output state.
- Test fixtures include at least one YAML sentence source with existing output
  and one pending item.
- Fixture names should be concrete, for example
  `tests/fixtures/sentences/one_existing_one_pending.yaml` and
  `tests/fixtures/output/sentences/one_existing_one_pending_batch_01.json`.
- Planned output filenames match the archived Python batch naming.
- Existing output JSON remains the completed-card authority.
- Before Rust writes new accepted output, the source lineage contract is fixed:
  source items have stable `item_id` values and generated cards carry
  `source_ref`.
- `sentences audit` may start with output/audio checks. Full stale-source
  detection lands after `source_ref` exists in accepted cards.

### M3: Sentence Validator

Status: Pending

Add or complete:

```text
crates/hindi-schema
crates/hindi-writer
```

Validate candidate sentence batches before any accepted write.

Acceptance:

- Register labels are canonicalized before new Rust generation writes output.
  The planned enum is `informal | standard | formal`.
- Existing current sentence batches are either migrated to that enum or the
  validator has a clearly labelled one-time compatibility mode for the
  migration fixture.
- Missing required fields fail clearly.
- `tokens` with spaces or punctuation fail clearly.
- `tokens[].word_index`, `tokens[].hindi`, and `tokens[].roman` must match the
  corresponding `words[]` entry.
- Empty optional fields are rejected or omitted according to the current schema.

### M4: Local Generation Spike

Status: Pending

Add or complete:

```text
crates/hindi-sentences
```

Generate one small batch with `ollama:translategemma:12b`, validate it, and
write only if clean.

Acceptance:

- Generation refuses loaded-model mismatch by default.
- Generation refuses output-file collisions.
- Output is written only after validation succeeds.
- Batch acceptance is all-or-nothing: a partially valid batch stays in the run
  folder and does not write accepted output.
- A durable `runs/.../report.json` records requested model, loaded Ollama model
  or model ID, prompt identity, timing, target output path, and validation
  result.
- Accepted card JSON does not carry run metadata unless the schema is
  deliberately expanded.
- Accepted card JSON does carry `source_ref` lineage so future audit can detect
  source drift after run cleanup.

### M5: Review And Repair Loop

Status: Pending

Add run reports, rejected-output capture, and optional evaluator prompts so bad
local model output does not silently become learner data.

Acceptance:

- Malformed JSON and validation failures are retained under `runs/`.
- `accept` fails on output collisions.
- Viewer-compatible accepted JSON remains under `output/sentences/`.

### M6a: Audio Parity

Status: Pending

Add:

```text
crates/hindi-audio
```

Backfill sentence audio without regenerating sentence cards.

Acceptance:

- Sentence audio can be backfilled without regenerating sentence cards.
- Existing MP3s and existing JSON `audio` fields are skipped by default.
- Audio writes use a temp path before accepted JSON is updated.

### M6b: Viewer Parity

Status: Pending

Make Rust-owned sentence output visible in the viewer.

Acceptance:

- Viewer reads generated output and audio paths.
- Refreshing the viewer after generation/audio backfill shows the new batch.
- Viewer export controls can use the same export contract as CLI export.

### M6c: Quick Export Parity

Status: Pending

Add:

```text
crates/hindi-export
```

Create a source/topic Anki export from accepted output.

Acceptance:

- Quick Anki export works for a source/topic.
- Export artifacts are written under `exports/anki/`.
- CLI export and viewer export do not duplicate formatting logic.
- Python remains available for comparison or emergency fallback.
