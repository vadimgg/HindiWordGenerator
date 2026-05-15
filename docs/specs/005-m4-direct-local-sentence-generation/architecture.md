# Architecture

## Part 1 - What Changed And Why

M4 turns the sentence pipeline from read-only planning into one accepted local
generation path. The main risk is letting the generation command become a pile
of unrelated responsibilities. The design keeps model IO, prompt building,
source-owned data, validation, accepted writes, and run reports separated.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/cli.rs` | Command parsing and help text. | Call Ollama, parse prompt JSON, or write output. |
| `src/config.rs` | `hindi.toml` parsing and model default resolution. | Decide generation behavior or start processes. |
| `src/ollama.rs` | Local Ollama HTTP API boundary and model readiness. | Shell out to `ollama run`, stop models, or choose model roles. |
| `src/sentence_plan.rs` | Pending source rows and target filenames. | Call models or write accepted output. |
| `src/sentence_generate.rs` | M4 orchestration and typed generation result. | Own low-level HTTP, schema validation rules, or temp-file mechanics. |
| `src/sentence_schema.rs` | Accepted sentence batch structs. | Trust model-owned source fields. |
| `src/sentence_validate.rs` | Candidate validation. | Read files, call models, or print output. |
| `src/accepted_writer.rs` | Atomic accepted-output writes. | Overwrite collisions or call validation itself. |
| `src/run_report.rs` | Diagnostic report serialization. | Become accepted-output authority. |

## Part 3 - Command Internals

### `hindi sentences generate --max-batches 1`

What the user sees:

```text
Generate Sentences

Model
  configured        ollama:translategemma:12b
  provider          ollama
  ready             yes

Plan
  planned batches   1
  planned items     5
  target            output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Generation
  prompt            generation_prompt_sentences_enrichment.txt
  validation        ok
  accepted          1
  skipped           0
  run report        runs/sentences/20260515T093000Z_ollama_translategemma_12b.json

Next
  hindi sentences audio
```

Model-not-ready output:

```text
Model not ready

Needed   translategemma:12b

The configured sentence model is not installed or not reachable. Run:
  ollama run translategemma:12b
```

Internal sequence:

```text
cli
  parse max-batches
  call sentence_generate

sentence_generate
  discover root
  load config/model
  plan pending work
  check Ollama
  build prompt
  call model
  extract enrichment JSON
  merge trusted source + enrichment
  validate
  write accepted output if valid
  write run report
  return typed result

main
  print typed result
```

## Part 4 - Shared Abstractions

| Abstraction | Used By | Contract | Must Not |
|---|---|---|---|
| `ModelSpec` | Config, Ollama client, generation output | Parses `ollama:<model>` and exposes provider/model. | Accept unknown providers silently. |
| Planner generation view | Plan command, generate command | Provides planned source rows and target paths. | Mutate output or include done rows. |
| Prompt payload | Generation | Contains only source row IDs and trusted source text. | Include `source_ref`, target filenames, title/subtitle as model-owned fields. |
| Enrichment extractor | Generation | Returns parsed enrichment keyed by source row ID from raw model text. | Treat prose as valid without JSON extraction. |
| Run report | Generation | Records diagnostics for accepted/failed attempts. | Become input to future planning. |

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `output/sentences/*.json` | M4 generation via M3 writer | Planner, viewer, export | Accepted learner authority. Write only after validation and refuse collisions. |
| `runs/sentences/*.json` | M4 generation | Humans/agents for diagnostics | Safe to delete intentionally; never source of truth. |
| `hindi.toml` | Human | Config reader | Optional. Missing file falls back to default sentence model. |
| `generation_prompt_sentences_enrichment.txt` | Human | Generation | Prompt fingerprint goes into run report. |

### Drift Scenario A - Model Output Becomes Trusted Source

**How it happens.** Merge code copies model-returned Hindi, English,
romanisation, title, source_ref, or filename.

**What breaks.** Curated YAML stops being the source of truth.

**Detection.** Merge tests include malicious extra trusted fields and prove they
are ignored.

**Resolution.** Rust copies trusted fields from planner/source data only.

### Drift Scenario B - Failed Validation Writes Accepted Output

**How it happens.** Writer is called before validation result is checked.

**What breaks.** Invalid learner data lands under `output/`.

**Detection.** Fake model invalid response tests check no output file exists.

**Resolution.** Validate before writer call; failed run report only.

### Drift Scenario C - CLI Manages Ollama

**How it happens.** Implementation shells out to `ollama run`, `ollama stop`, or
tries to unload models.

**What breaks.** M4 grows model lifecycle ceremony and RAM behavior becomes
implicit.

**Detection.** Review grep for process spawning and shell commands around
Ollama.

**Resolution.** Use local HTTP only and print recovery commands.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| Model lifecycle | Spawning/stopping/switching Ollama. | Local HTTP calls plus clear recovery text. |
| Source trust | Model can overwrite source fields. | Rust owns source/title/source_ref/target. |
| Validation | Accepted writer called before validation. | Validator must pass before accepted write. |
| Planner reuse | Generator builds independent target filenames. | Generator uses planner data. |
| Reports | Run report drives future pending state. | Run report is diagnostics only. |
| Protected paths | Failure tests mutate real `input/`, `output/`, or `audio/`. | Tests use temp dirs/fake clients. |

## Appendix - Files Removed Or Moved

None planned.

## Appendix - Out-Of-Scope Residue

- Model quality evaluation remains manual after the first successful M4 smoke
  test.
