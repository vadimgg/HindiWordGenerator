# Architecture

## Part 1 - What Changed And Why

`hindi sentences generate` should become a staged pipeline. The command still
plans pending batches, calls the configured local model, validates candidates,
and writes accepted output. The internal model interaction changes from one
large "give me everything" prompt to focused stages for register, literal, and
word breakdown.

The main architectural risk is partial trust: each model stage returns only a
slice of enrichment, while Rust remains responsible for source fields, lineage,
merge semantics, validation, and accepted writes. Reviewers should reject any
implementation that lets a stage mutate source truth or bypass validation.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/cli.rs` | Existing `hindi sentences generate --max-batches <n>` parse/help. | Know about stages, prompt IDs, merge rules, or run report internals. |
| `src/main.rs` | Route parsed command to `sentence_generate`. | Own generation rules. |
| `src/sentence_generate.rs` | Generation orchestration: plan, model readiness, staged calls, validation, writes, user-facing outcome. | Parse model output directly in ad hoc ways or construct accepted cards without validation. |
| `src/sentence_enrichment.rs` or `src/sentence_stages.rs` | Stage prompt rendering, stage response parsing, merge-by-ID, candidate construction from trusted planner rows plus stage outputs. | Write files, call Ollama, or trust model-provided source fields. |
| `src/eval_prompts/` or `src/sentence_prompts/` | Prompt text for focused generation stages if prompts are compiled into the binary. | Depend on ignored `eval/` run artifacts. |
| `src/sentence_plan.rs` | Current YAML/output planning. | Know about model stages. |
| `src/sentence_validate.rs` | Candidate validation and token/word/register/source invariants. | Repair or silently normalize bad model output beyond existing validator rules. |
| `src/accepted_writer.rs` | Atomic accepted-output writes and collision refusal. | Accept partial batches or overwrite output. |
| `src/run_report.rs` | Persistent run report schema and write helper. | Omit stage-level prompt/model/timing data after this spec. |
| `src/ollama.rs` | Configured model readiness and local model generation. | Start/stop/switch Ollama models. |

## Part 3 - Command Internals

### `hindi sentences generate --max-batches <n>`

What the user runs:

```bash
hindi sentences generate --max-batches 1
```

Success output remains compact:

```text
Generate Sentences

  model             ollama:gemma4:latest
  planned batches   1
  accepted batches  1

Accepted Output
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Run Reports
  runs/sentences/1778850000000_ollama_gemma4_latest.json

Next
  hindi sentences audio
```

Internal sequence:

```text
src/main.rs
  call sentence_generate::generate_from_current_dir(max_batches)

src/sentence_generate.rs
  discover project root
  load config sentence_generation model
  derive plan from current YAML/output
  stop before model calls if planner reports errors or no pending work
  check configured model readiness once
  load/resolve staged prompt definitions
  for each planned batch:
    render register prompt from trusted planner rows
    call model
    parse register response keyed by source id
    render literal prompt from trusted planner rows
    call model
    parse literal response keyed by source id
    render word-breakdown prompt from trusted planner rows
    call model
    parse word-breakdown response keyed by source id
    merge all stages by source id
    build candidate sentence batch with Rust-owned source fields/source_ref
    validate candidate
    if validation passes:
      write accepted output atomically
      write accepted run report with stage metadata
    if any stage/merge/validation/write fails:
      write failed run report with stage metadata and errors
      return failure without writing accepted output for that batch
```

Point of no return: the accepted-output write. All model calls, parsing,
merging, and validation must finish before this point.

### Stage Order

The default stage order should be:

1. `sentence/register`
2. `sentence/literal`
3. `sentence/word-breakdown-from-translation`

Use the translation-guided word breakdown because source English is trusted
curated input and eval results were strong. `sentence/word-breakdown` remains
useful as an eval comparison prompt but is not the default generation stage.

### Stage Failure Behavior

Any stage failure fails the whole batch:

```text
Problem
  Stage sentence/word-breakdown-from-translation did not return item 0003.

Run
  Inspect the run report, fix prompt/model/source issues, then rerun
  `hindi sentences generate --max-batches 1`.
```

## Part 4 - Shared Abstractions

### Stage Prompt Registry

Used by:

- `sentence_generate`
- staged response parser/merger
- run report metadata
- tests

Contract:

- Input: stage ID such as `sentence/register`.
- Output: stage version, prompt text, prompt fingerprint, expected response
  parser.
- Error: unknown stage ID is a code/config bug and should fail tests.
- Never reads ignored `eval/` output.

Review smell:

- Generation hardcodes prompt text inline inside orchestration.
- Generation refers to the eval report history instead of the built-in prompt
  registry.

### Stage Response Parsers

Used by:

- `sentence_generate`
- tests

Contract:

- Accept model response text.
- Extract a YAML or JSON object, tolerating markdown fences and leading/trailing
  prose if practical.
- Return typed records keyed by source ID.
- Reject duplicate IDs.
- Do not accept source fields from the model as trusted data.

Review smell:

- String-splitting stage responses by line.
- Prompt-specific parse rules duplicated in `sentence_generate.rs`.

### Staged Merger

Used by:

- `sentence_generate`
- tests

Contract:

- Input: `PlannedSentenceBatch`, register records, literal records, word records.
- Output: `SentenceBatch` candidate.
- Requires exactly one record per source row per stage.
- Fails on missing, duplicate, or extra stage IDs.
- Copies title, subtitle, Hindi, romanisation, English, source_ref, and tags
  from planner/YAML data only.

Review smell:

- Accepted cards built directly from model output.
- Missing stage data replaced with empty strings or defaults.

### Stage Run Metadata

Used by:

- `run_report`
- `sentence_generate`
- review/debugging

Minimum per-stage fields:

- `stage_id`
- `prompt_version`
- `prompt_fingerprint`
- `model`
- `model_digest`
- `duration_ms`
- `ok`
- `error`

Review smell:

- A failed run report says "validation failed" without naming which stage or
  prompt version produced the bad data.

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `output/sentences/*.json` | `accepted_writer` through `sentence_generate` | viewer, export, audio, planner | Authority for accepted cards. Write only after full staged validation succeeds. |
| `runs/sentences/*.json` | `run_report` through `sentence_generate` | humans, future diagnostics | Diagnostic metadata. Must include stage prompt/model/timing/errors. Safe to delete intentionally. |
| `src/eval_prompts/*.yaml.hbs` or `src/sentence_prompts/*` | developers | generation/eval prompt registry | If generation reuses eval prompt text, it must share the source text or deliberately copy with version/fingerprint changes. |
| `docs/DESIGN.md` / `docs/ROADMAP.md` | developers | humans/agents | Must describe staged generation as the default path after this spec. |

### Drift Scenarios

#### Prompt Drift Between Eval And Generation

**How it happens.** The eval prompt improves but generation keeps an older copy.

**What breaks.** Eval says a prompt is good, but accepted generation uses a
different prompt.

**Detection.** Prompt registry tests compare IDs/versions or generation imports
the same prompt constants used by eval.

**Resolution.** Share prompt constants where possible; otherwise record
generation prompt fingerprints in run reports and update docs when versions
change.

> **Review flag.** Reject manually duplicated register/literal/word prompt text
without an explicit version/fingerprint story.

#### Stage ID Drift

**How it happens.** One stage omits an item, duplicates an item, or returns an
unexpected item ID.

**What breaks.** Cards could be merged with mismatched enrichment.

**Detection.** Staged merger rejects missing, duplicate, and extra IDs before
validation/writes.

**Resolution.** Write failed run report and leave batch pending.

> **Review flag.** Reject `unwrap_or_default()`-style merge behavior for missing
stage data.

#### Full-Enrichment Fallback Drift

**How it happens.** The old single-prompt path remains as a hidden fallback.

**What breaks.** Failures become hard to reproduce and prompt eval no longer
predicts generation behavior.

**Detection.** Grep for the old full-enrichment prompt in generation code and
tests.

**Resolution.** Keep full enrichment in eval only; generation uses the staged
registry.

> **Review flag.** Reject generation code that calls the full-enrichment prompt
after staged validation fails.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| Command surface | New user-facing generation commands or model flags. | Existing `hindi sentences generate --max-batches <n>` behavior with staged internals. |
| Source trust | Model-provided Hindi, romanisation, English, source_ref, or filename. | Rust copies source fields and lineage from planner data. |
| Stage merge | Missing stage data defaulted or silently skipped. | Missing/duplicate/extra IDs fail the batch. |
| Validation | Accepted output written before validation. | Existing validator gates every accepted write. |
| Reports | One opaque prompt hash for a multi-stage run. | Per-stage prompt IDs, fingerprints, timings, and errors. |
| Prompt reuse | Eval and generation prompts drift without visibility. | Shared prompt constants or explicit version/fingerprint records. |
