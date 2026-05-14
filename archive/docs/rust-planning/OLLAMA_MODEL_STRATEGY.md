# Ollama Model Strategy

This document records which local Ollama models we should use for Hindi card
generation and why. It is based on saved experiment results under
`archive/python/experiments/`.

The target machine is a MacBook Pro with 48GB memory. That is enough to run the
larger tested models, but speed still matters because card generation uses many
small model calls.

## How To Recheck Results

Sentence translation and QA report:

```bash
cd archive/python
python3 experiments/translation_report.py all --no-color
```

Word-field summaries are saved under:

```text
archive/python/experiments/ollama_word_fields/results/
```

The sentence report combines exact-match checks, timing, timeout counts, and
evaluator ratings. Exact match is useful, but evaluator score is the more
important quality signal because a translation can be correct without matching
the reference wording exactly.

Evaluator labels:

- `good`: safe to use as learner-facing material.
- `usable`: mostly right, but should be reviewed or cleaned.
- `weak`: likely needs correction before use.
- `bad`: misleading, malformed, or unsafe for learner data.

## What We Tested

Sentence experiments tested:

- Hindi-only translation to English and romanisation.
- Hindi-only word-by-word breakdown.
- Gloss-guided translation.
- Register detection: formal, standard/everyday, informal.
- Source-row cleanup from Hindi + romanisation + English.
- Source-row word breakdown.
- Source-row issue detection for known input problems.

Word experiments tested:

- Raw word translation and romanisation.
- Grammar core fields.
- Syllables and related words.
- Example sentences.
- Delhi/practical usage notes.
- Sound-alikes.
- Etymology notes.
- Source-row synthesis from Hindi + romanisation + English.

## Recommended Model Roles

| Task | Model | Why |
|---|---|---|
| Sentence card generation from YAML source rows | `translategemma:12b` | Best broad local sentence result among Ollama models we tested. It scored 4.0 overall across sentence tasks and did especially well on source-row word breakdown. |
| Optional sentence source/input QA | `gemma4:latest` | Best source-row issue detector in the saved report: 4.8 score, 5/5 exact issue flags, no weak/bad results. |
| Register detection | `translategemma:12b` | Tied the agent baseline on the saved register task with 5/5 exact labels and 4.2 evaluator score. |
| Word-card field generation, quality mode | `gemma4:26b` | Best word-field quality: 4.25 average usefulness, with strong grammar core, sound-alikes, and source-row synthesis. |
| Word-card field generation, quick draft mode | `gemma4:latest` | Faster than `gemma4:26b` and usable for drafts, but weaker on grammar, sound-alikes, and some romanisation. |

## Model Notes

### `translategemma:12b`

Best current default for sentence generation.

Strengths:

- Strong on source-row sentence flows.
- Good source-row word breakdown: 4.8 score in the sentence report.
- Good simple translation and gloss-guided translation.
- Fast enough for iterative local use on this machine.

Weaknesses:

- Poor source-row issue detection in the saved run.
- Strict romanisation prompts produced some weak/bad results.
- Sometimes drops visible honorific cues such as `ji` from English.

Use it for:

- generating sentence translations, literal glosses, register, words, and tokens
  from curated YAML rows.

Do not use it as the only guard for:

- detecting corrupted source input.

### `gemma4:latest`

Best current source-QA model.

Strengths:

- Strongest source-row issue detection result: 4.8 score, 5/5 exact flags.
- Practical speed for small QA passes.
- Good fit for checking whether an input row is awkward, mismatched, or likely
  corrupted before generation.

Weaknesses:

- The latest saved sentence report only includes it for source-row issue
  detection, so it should not replace `translategemma:12b` for full sentence
  generation without another full matrix.
- In word-field tests it was usable but not final-quality: 3.42 average
  usefulness, with weak grammar handling and weak sound-alike behavior.

Use it for:

- source input QA before generation.
- quick word-card drafts when speed matters more than final quality.

### `gemma4:26b`

Best quality candidate for word cards, but slow.

Strengths:

- Best word-field score: 4.25 average usefulness.
- Excellent grammar core, sound-alikes, and source-row synthesis in word tests.
- Better than `gemma4:latest` for careful word-card generation.

Weaknesses:

- Slow: the saved word run took about 13.4 minutes for 24 calls.
- Sentence testing was incomplete/slow and produced a timeout in the saved
  report, so it should not be the default sentence generator.
- Still needs review for etymology and some usage/example fields.

Use it for:

- careful word-card field generation in smaller batches.

Do not use it for:

- broad sentence generation until we rerun a full, clean sentence matrix.

### `aya-expanse:8b`

Fast but not reliable enough.

Strengths:

- Very fast in the saved source-row issue detection run.
- Can produce valid JSON.

Weaknesses:

- Weak overall quality for sentence issue detection.
- Word-field average was 2.33 with bad etymology, syllables, sound-alikes, and
  romanisation problems.

Use it for:

- experiments only.

### `qwen3.6:35b`

Quality is not worth the latency for this workflow.

Strengths:

- Usable word-field quality, 3.75 average usefulness.
- Strong example sentences and decent grammar/usage fields.

Weaknesses:

- Too slow for normal local generation: the saved word run took about 52
  minutes for 24 calls.
- Etymology restraint was poor.

Use it for:

- occasional research comparison only, not production generation.

### `ayansh03/hindi-gemma:q8_0`

Not recommended.

Strengths:

- Fast.

Weaknesses:

- Low source-row issue detection score, 1.6.
- Produced weak/bad corrections and unreliable learner-facing output.

### `mashriram/sarvam-1:latest`

Not recommended for this pipeline.

Strengths:

- None shown by the saved source-row issue detection run.

Weaknesses:

- Returned invalid/unstructured result shapes for the requested audit.
- Scored bad on all saved issue-detection cases.

## Recommended Pipeline

Use more than one model. The saved results suggest that one local model does not
win every task.

Sentence generation flow:

1. Optional: `gemma4:latest` checks YAML source rows for likely input problems.
2. `translategemma:12b` generates sentence fields and word breakdowns.
3. The Rust validator builds/checks `tokens[]` deterministically and rejects
   malformed output.
4. A reviewer/evaluator pass samples generated cards before accepting larger
   batches.

Word generation flow:

1. `gemma4:latest` can create quick word-card drafts.
2. `gemma4:26b` should be the quality-mode model for final word-card field
   generation.
3. Etymology and sound-alike fields should remain optional and reviewed because
   every tested model showed some risk there.

## Internal Multi-Model Flow

The CLI should treat model choice as role-based routing, not as a single global
model. A run has a model plan:

```text
sentence_source_qa -> checks source YAML rows
sentence_generation -> generates sentence card fields
word_draft          -> quick word-card drafts
word_quality        -> slower higher-quality word-card fields
```

For sentence work, the internal flow is:

| Step | Key Point | Model | Files Involved | What Happens |
|---|---|---|---|---|
| 1 | Find pending work | none | `input/sentences/*.yaml`, `output/sentences/*.json` | The CLI reads source YAML, checks existing output, and creates a plan for only the sentences that still need cards. |
| 2 | Optional source quality check | `gemma4:latest` through `sentence_source_qa`, only when the user runs `hindi sentences source-qa` | planned YAML rows | The QA model looks for obvious source problems and writes a source-QA report. This is separate from generation so only one local model needs to be loaded at a time. |
| 3 | Review source issues | none | planned YAML rows, source-QA report | If source QA finds issues, the CLI summarizes them and lets the user review corrections one at a time. The user can apply a correction, skip it, stop for manual edit, or keep the report. |
| 4 | Load generation model | `translategemma:12b` through `sentence_generation` | no card files yet | The CLI checks that the sentence-generation model is loaded before asking it to create card data. |
| 5 | Generate card fields | `translategemma:12b` | planned YAML rows, temporary model output | The model creates English, literal translation, register, and `words[]` word breakdowns. |
| 6 | Build exact tokens | none | temporary generated JSON | Rust code builds/checks `tokens[]` from `words[]`. The model should not create space or punctuation tokens. |
| 7 | Validate | none | temporary generated JSON | The validator checks the schema, required fields, token/word alignment, and output safety. Bad output is rejected. |
| 8 | Write accepted output | none | `output/sentences/*.json` | Only validated cards are written. Existing output files are not overwritten. |
| 9 | Record run details | none | run report or CLI audit metadata | The run records which model handled QA, which model handled generation, timings, and any validation failures. |

Short version: `hindi sentences source-qa` can check the source first with
`gemma4:latest`; then `hindi sentences generate` uses `translategemma:12b` to
generate cards and Rust validates/writes only clean output.

Default non-interactive source-QA behavior should be conservative: write a
report and do not edit YAML unless the user passes an explicit flag such as
`--apply`.

For word generation, the same pattern applies, but the role can be selected by
mode:

```text
hindi words generate --mode draft   -> word_draft
hindi words generate --mode quality -> word_quality
```

## Model Loading Policy

The CLI should manage model-runtime readiness through provider-neutral
`hindi models ...` commands. It should not silently start or stop models inside
a generation run.

Default behavior:

- run `ollama ps` or the Ollama API to discover currently loaded models
- print which model is loaded for each role
- refuse to run a role if its configured model is not loaded
- refuse to run if an unexpected model is loaded and the project policy says
  `max_loaded_models = 1`
- tell the user exactly which command to run, for example
  `hindi models prepare sentences`
- start/stop models only through explicit `hindi models prepare <workflow>`
  approval

Reasoning:

- Starting a large model is slow and visible to the user.
- Stopping another model can interrupt work in another terminal.
- On a 48GB MacBook Pro, the user may intentionally keep one model warm while
  testing another. The CLI should not guess memory policy.

Project config should own model roles and memory policy:

```toml
[workflows]
sentences = "ollama:translategemma:12b"
source-qa = "ollama:gemma4:latest"
words-draft = "ollama:gemma4:latest"
words-quality = "ollama:gemma4:26b"

[model_runtime]
allow_model_switch = false
max_loaded_models = 1
```

`hindi models prepare <workflow>` may stop the current model and start the
configured model after confirmation. A non-interactive pipeline may pass
`--allow-model-switch`, but the run report should record every model switch.
Generation and source-QA commands themselves should not switch models.

## Switching Between Models

Because normal Ollama usage usually keeps one model loaded at a time, source QA
and generation should be separate CLI steps. Do not hide multiple model switches
inside the normal generation command.

```text
source QA stage      -> separate command; require gemma4:latest loaded
generation stage     -> require translategemma:12b loaded
validation/write     -> no model required
audio/export/viewer  -> no model required
```

If a workflow needs two model roles, split it into separate commands:

```bash
hindi models prepare source-qa
hindi sentences source-qa --batch-size 3 --max-batches 1
hindi models prepare sentences
hindi sentences generate --batch-size 3 --max-batches 1 --require-source-qa
```

Do not assume both models can stay resident. The run report should record:

- requested model for each role
- loaded model seen before each role
- whether the CLI started a model explicitly
- model call timing per role
- timeout or mismatch failures

## First Rust Defaults

Use these defaults for the first Rust implementation:

```text
sentences         = "ollama:translategemma:12b"
source-qa         = "ollama:gemma4:latest"
words-draft       = "ollama:gemma4:latest"
words-quality     = "ollama:gemma4:26b"
```

The CLI should allow overriding each model, but it should print the loaded model
and refuse a mismatch by default. The source-QA default is configured but should
only run when the user enables source QA or project config turns it on.

`gemma4:latest` is a floating tag. It is acceptable for experiments, but run
metadata should record the Ollama model ID so we know which exact local model
produced the result. Prefer a pinned model tag or digest once we settle the Rust
defaults.
