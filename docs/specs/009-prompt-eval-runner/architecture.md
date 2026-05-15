# Architecture

## Part 1 - What Changed And Why

Spec 009 adds a prompt workbench around built-in prompt IDs. Normal generation
still owns accepted output; eval owns temporary prompt experiments under
`eval/`. The main risks are accidental writes to learner data, prompt/grading
template drift, and run metadata that is too vague to compare later.

## Part 2 - Module Ownership

| Module | Owns | Must never |
|---|---|---|
| `src/cli.rs` | Parse `hindi eval run`, `hindi eval grade`, and `hindi eval report`; print help text. | Load YAML, call Ollama, render Handlebars templates, or write eval files. |
| `src/eval.rs` | Eval input loading, prompt ID resolution, template context, artifact writes, grade packet flow, grade parsing, report rendering. | Write `output/`, manage Ollama lifecycle, or print directly from deep helpers. |
| `src/eval_prompts/` | Built-in input/grading templates registered by prompt ID. | Become user run output or accepted learner data. |
| `src/ollama.rs` | `/api/ps` running-model lookup and model generation calls. | Shell out to `ollama ps` unless HTTP support is impossible. |
| `src/main.rs` | Wire parsed commands to eval execution and map success to exit codes. | Own eval rules or construct prompt contexts. |

## Part 3 - Command Internals

### `hindi eval run`

What the user runs:

```text
hindi eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2
```

Internal sequence:

```text
src/cli.rs
  parse EvalRun { input, prompt_id, fields, max_items }

src/eval.rs
  discover project root
  resolve built-in prompt id and paired grading template
  load YAML source
  select top-level fields, defaulting to id,hindi,romanisation,english
  limit selected items
  build Handlebars context
  render input prompt
  ask src/ollama.rs for running models via /api/ps
  require exactly one running model
  send prompt to selected model
  create eval/<prompt-id>/<run-id>/
  write prompt.txt
  write response.txt
  write meta.json
  write summary.txt
  return EvalRunReport

src/main.rs
  print EvalRunReport
```

Point of no return: after Ollama returns and the eval run directory is created.
All writes stay under `eval/`.

### `hindi eval grade`

What the user runs:

```text
hindi eval grade sentence/register/2026-05-15_143012_translategemma_12b [--response grade.yaml]
```

Internal sequence:

```text
src/cli.rs
  parse EvalGrade { run }

src/eval.rs
  resolve run as either eval/... path or prompt-scoped run id
  load meta.json
  resolve paired grading prompt from meta.prompt_id
  render grade prompt with run metadata, source items, prompt.txt, response.txt
  write grade_prompt.txt
  write grade_packet.md
  if --response provided:
    read grader response from file
  else:
    open grade_packet.md in $EDITOR
    extract pasted grader response from grade_packet.md
  write grade_response.txt
  parse response as YAML or JSON
  validate shared grading schema
  write grade.json
  update summary.txt
  return EvalGradeReport

src/main.rs
  print EvalGradeReport
```

The editor packet contains the rendered grading prompt plus a clearly marked
paste area. The command does not call Claude/ChatGPT directly.

### `hindi eval report`

What the user runs:

```text
hindi eval report [--no-color] [--verbose]
```

Internal sequence:

```text
src/cli.rs
  parse EvalReport { color, verbose }

src/eval.rs
  discover project root
  recursively scan eval/ for meta.json files
  load optional grade.json beside each meta.json
  load source YAML referenced by meta.input_path when available
  collect displayed source rows by item id
  render source Hindi, romanisation, and English
  group rows by prompt/test name
  render one model row per run under its test
  hide run folder and raw score detail unless --verbose was passed
  color section headers, scores, times, and verdicts unless --no-color was passed
  render failure notes first, then warnings, then informational notes

src/main.rs
  print EvalSummaryReport
```

The report is read-only. It treats `meta.json` and optional `grade.json` as
structured authority and uses source YAML only for human display context.

## Part 4 - Shared Abstractions

### Prompt Registry

Used by:

- `hindi eval run`
- `hindi eval grade`
- `hindi eval report`

Contract:

- Input: prompt ID such as `sentence/register`.
- Output: prompt metadata, input template, grading template, default fields,
  grading threshold.
- Error: unknown prompt ID or missing paired template.

Review smell:

- Prompt IDs assembled from filesystem paths.
- Input and grading templates registered independently.

### Eval Run Path

Used by:

- `hindi eval run`
- `hindi eval grade`

Contract:

- Path shape: `eval/<prompt-category>/<prompt-name>/<timestamp>_<model-slug>/`.
- A grade run argument may be either that path or
  `<prompt-category>/<prompt-name>/<run-id>`.
- Resolution rule: if the argument starts with `eval/`, resolve it from the
  project root as given; otherwise prepend `eval/` and resolve from the project
  root. Do not probe arbitrary sibling directories before applying this rule.
- Model slug must be filesystem-safe.

Review smell:

- Flat `eval/<run-id>/` folders.
- Prompt ID omitted from the path.

### Grade Schema

Used by:

- Grading prompt templates
- `hindi eval grade`
- `hindi eval report`

Contract for `grade.json`:

```json
{
  "run_id": "sentence/register/2026-05-15_143012_translategemma_12b",
  "grader": "human",
  "graded_at": "2026-05-15T15:00:00Z",
  "scores": {
    "accuracy": { "score": 4, "max": 4, "note": "" },
    "completeness": { "score": 4, "max": 4, "note": "" },
    "format_compliance": { "score": 4, "max": 4, "note": "" },
    "consistency": { "score": 4, "max": 4, "note": "" },
    "confidence": { "score": 4, "max": 4, "note": "" }
  },
  "total": { "score": 20, "max": 20, "pct": 100 },
  "verdict": "pass",
  "item_flags": [],
  "summary": "Accurate and complete."
}
```

The grader may paste YAML or JSON into `grade_packet.md`. The command parses
either format, validates this schema, and writes canonical JSON to `grade.json`.

Axis scale:

- `1` broken
- `2` weak
- `3` acceptable
- `4` good

The grading template for each prompt ID defines task-specific guidance and pass
threshold while preserving the shared schema.

Review smell:

- A prompt-specific grade file that cannot be compared with other prompt IDs.
- A neutral midpoint scale such as 1-5.

### Grade Packet Markers

Used by:

- `hindi eval grade`

Contract:

`grade_packet.md` must contain these exact markers:

````text
## Grading Prompt

<rendered prompt for the user to copy>

## Paste Grader Response Below

```yaml
```
````

After `$EDITOR` closes, the command extracts everything after
`## Paste Grader Response Below`, strips one optional fenced code block, and
parses the remaining text as YAML or JSON.

Review smell:

- The extractor depends on vague prose instead of a stable marker.
- The marker text is duplicated in several files.

### Eval Run Metadata

Used by:

- `hindi eval run`
- `hindi eval grade`
- `hindi eval report`

Contract for `meta.json`:

```json
{
  "run_id": "sentence/register/2026-05-15_143012_translategemma_12b",
  "prompt_id": "sentence/register",
  "input_path": "input/sentences/complete_hindi_chapter_02_sentences.yaml",
  "fields": ["id", "hindi", "romanisation", "english"],
  "max_items": 2,
  "item_count": 2,
  "model": "ollama:translategemma:12b",
  "model_source": "ollama /api/ps",
  "started_at": "2026-05-15T14:30:12Z",
  "finished_at": "2026-05-15T14:30:24Z",
  "timing_ms": {
    "render": 4,
    "model": 12310,
    "total": 12402
  },
  "artifacts": {
    "prompt": "prompt.txt",
    "response": "response.txt",
    "summary": "summary.txt"
  }
}
```

Review smell:

- Timing exists only in terminal output.
- Prompt ID, model, or selected fields are missing from structured metadata.

### Eval Summary

Used by:

- `hindi eval run`
- `hindi eval grade`

Contract:

After `hindi eval run`, `summary.txt` contains:

```text
Eval Run

Prompt
  id        sentence/register
  input     input/sentences/complete_hindi_chapter_02_sentences.yaml
  items     2
  fields    id,hindi,romanisation,english

Model
  selected  ollama:translategemma:12b
  source    Ollama /api/ps

Timing
  render    4ms
  model     12.3s
  total     12.4s

Artifacts
  prompt    prompt.txt
  response  response.txt
  meta      meta.json

Grade
  status    not graded
```

After `hindi eval grade`, append or regenerate the grade section:

```text
Grade
  verdict   pass
  score     18/20 (90%)
  summary   Strong output with one register issue.
  details   grade.json
```

`summary.txt` is rebuildable from `meta.json` and optional `grade.json`; never
treat it as structured authority.

## Part 5 - Data And Drift Risks

### Persistent Files

| File | Written By | Read By | Rule |
|---|---|---|---|
| `eval/<prompt-id>/<run-id>/prompt.txt` | `hindi eval run` | humans, `hindi eval grade` | Exact rendered prompt sent to Ollama. |
| `eval/<prompt-id>/<run-id>/response.txt` | `hindi eval run` | humans, `hindi eval grade` | Raw model response, never parsed as authority. |
| `eval/<prompt-id>/<run-id>/meta.json` | `hindi eval run` | humans, `hindi eval grade`, `hindi eval report` | Run provenance: prompt ID, model, timing, input path, fields, item count, artifact paths. |
| `eval/<prompt-id>/<run-id>/summary.txt` | `hindi eval run`, `hindi eval grade` | humans | Human-readable digest; regenerated from run/grade state. |
| `eval/<prompt-id>/<run-id>/grade_prompt.txt` | `hindi eval grade` | humans | Exact grading prompt text. |
| `eval/<prompt-id>/<run-id>/grade_packet.md` | `hindi eval grade`, user editor | humans, `hindi eval grade` | Editor handoff file with prompt and paste area. |
| `eval/<prompt-id>/<run-id>/grade_response.txt` | `hindi eval grade` | humans | Raw pasted grader response. |
| `eval/<prompt-id>/<run-id>/grade.json` | `hindi eval grade` | humans, `hindi eval report` | Parsed shared grade schema. |

All `eval/` files are ignored by git by default.

### Drift Scenarios

#### Input And Grading Prompt Drift

**How it happens.** A prompt ID gets an input template but no matching grading
template, or they use different output expectations.

**What breaks.** `hindi eval grade` cannot produce useful quality data for a
run.

**Detection.** Prompt registry unit test requires every prompt ID to have input
and grading templates.

**Resolution.** Stop with an unknown/missing paired template error.

#### Run Metadata Drift

**How it happens.** `summary.txt` or grade files disagree with `meta.json`.

**What breaks.** Future reports compare the wrong prompt/model/timing.

**Detection.** Grade command loads `meta.json` and updates summary from current
run files.

**Resolution.** Treat `meta.json` and `grade.json` as structured sources;
`summary.txt` is rebuildable.

#### Output Authority Drift

**How it happens.** Eval code writes to `output/` or accepted generation code
reads from `eval/`.

**What breaks.** Experiments become learner data by accident.

**Detection.** Integration test asserts eval writes no files under `output/`.

**Resolution.** Eval writes only under `eval/`; normal generation ignores
`eval/`.

## Part 6 - Code Review Checklist

| Area | Reject | Accept |
|---|---|---|
| CLI naming | `hindi eval --input` or `hindi eval input` in new code/docs. | `hindi eval run`, `hindi eval grade`, and `hindi eval report`. |
| Model selection | `--model`, model switching, shell-only `ollama ps`. | Exactly one running model from `/api/ps`. |
| Prompt storage | User prompt paths as the primary v1 flow. | Built-in prompt IDs with paired input/grading templates. |
| Run layout | Flat `eval/<run-id>/`. | `eval/<prompt-category>/<prompt-name>/<run-id>/`. |
| Eval writes | Any write under `output/`. | Writes only under ignored `eval/`. |
| Grade schema | Free-form prose only. | Shared five-axis schema plus item flags and summary. |
| Editor flow | Ambiguous file opened in `$EDITOR`. | `grade_packet.md` opened, prompt and paste area both visible. |

## Appendix - Files Removed Or Moved

None.

## Appendix - Out-Of-Scope Residue

- Future `hindi eval compare` can add model-to-model analytics over `meta.json`
  and `grade.json`.
- Future non-interactive grade import can reuse the same parser without opening
  `$EDITOR`.
