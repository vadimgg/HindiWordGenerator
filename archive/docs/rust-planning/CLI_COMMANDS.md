# CLI Command Spec

This document owns the planned public `hindi` command surface. Commands should
describe user goals, not implementation internals. Provider details such as
Ollama, Whisper, gTTS, or future local models belong in config, status output,
and run reports, not in the command shape.

The Rust CLI does not exist yet. This is the contract to build toward.

## Command Families

```text
hindi doctor
hindi config show
hindi sources list
hindi models status
hindi models prepare <workflow>
hindi sentences check
hindi sentences source-qa
hindi sentences generate
hindi sentences audit
hindi sentences review
hindi sentences accept
hindi sentences audio
hindi sentences audio check
hindi runs list
hindi runs clean
hindi words check
hindi words generate
hindi transcribe check
hindi transcribe run
hindi anki topics
hindi anki export
hindi viewer open
```

Preferred workflow:

```bash
hindi doctor
hindi sentences check --batch-size 3 --max-batches 1
hindi models prepare sentences
hindi sentences generate --batch-size 3 --max-batches 1
hindi sentences audio
hindi viewer open
```

Command grammar is noun-first: `hindi sentences generate`, `hindi anki export`,
`hindi viewer open`. The only top-level verbs are global diagnostics such as
`doctor`.

## Command Maturity

The normal sentence path is the first implementation target. Other command
families are documented so the public shape stays coherent, but they should not
all ship in the first Rust milestone.

| Stage | Commands |
|---|---|
| First skeleton | `doctor`, `config show`, `models status`, `models prepare`, `sentences check`, `runs list` |
| Sentence generation | `sentences generate`, `sentences review`, `sentences accept`, `sentences audio`, `sentences audit` |
| Product parity | `viewer open`, `anki topics`, `anki export` |
| Later | `words ...`, `transcribe ...`, repair/regenerate commands |

## Global Options

| Option | Required | Meaning |
|---|---:|---|
| `--project <path>` | no | Project root. Defaults to current directory or nearest project root. |
| `--config <path>` | no | Config file. Defaults to project config. |
| `--json` | no | Print machine-readable JSON instead of rich terminal output. |
| `--color auto|always|never` | no | Color policy. Defaults to `auto`; `NO_COLOR` also disables color. |
| `--no-color` | no | Alias for `--color never`. |
| `--quiet` | no | Print only errors and final summary. |
| `--yes` | no | Accept safe confirmations. Must not approve destructive overwrites. |
| `--dry-run` | no | Show planned writes or model switches without doing them. |

Durations use a compact suffix format: `30s`, `5m`, `2h`, or `7d`. A bare
number is invalid.

## Automation Contract

Every command must be usable by agents and scripts without hanging.

- `--json` suppresses rich progress output and interactive prompts.
- In `--json` mode, stdout contains one JSON object and stderr contains human
  diagnostics only.
- If a command would need confirmation in `--json` or non-interactive mode, it
  fails with `needs_confirmation` unless a specific non-interactive flag such as
  `--yes`, `--apply`, or `--allow-model-switch` is valid for that operation.
- Destructive overwrites are never approved by generic `--yes`.
- Long-running commands should emit progress events only in rich terminal mode.

JSON envelope:

```json
{
  "ok": true,
  "command": "sentences.check",
  "summary": {},
  "data": {},
  "warnings": []
}
```

Error envelope:

```json
{
  "ok": false,
  "command": "sentences.generate",
  "error": {
    "code": "model_not_ready",
    "message": "Model runtime is not ready for sentence generation.",
    "next": ["hindi models prepare sentences"]
  }
}
```

Exit codes:

| Code | Meaning |
|---:|---|
| `0` | Success, including "nothing to do". |
| `1` | User/config/input error. |
| `2` | Candidate JSON or schema validation failed. |
| `3` | Safety block, such as file collision, stale QA report, or model mismatch. |
| `4` | External dependency/runtime failure, such as Ollama, TTS, ffmpeg, or viewer server failure. |
| `130` | User cancelled. |

## Display Rules

Any command that prints Hindi must print romanisation directly under it:

```text
Hindi   क्या बात है?
Roman   kyā bāt hai?
English What's the matter?
```

Do not print Hindi-only examples in prompts, reports, or errors.

## `hindi doctor`

Checks project readiness.

Inputs:

- no required arguments

Options:

- global options only

Writes:

- nothing

Output:

```text
Hindi Word Generator
Doctor

Project
  root             /Users/vadim/Projects/Hindi/HindiWordGenerator
  input            ok  input/sentences, input/words
  output           ok  output/sentences, output/words
  docs             ok

Local services
  model runtime    ok  provider configured
  audio backend    ok  service-backed TTS configured

Next
  hindi sentences check --batch-size 3 --max-batches 1
```

## `hindi config show`

Shows effective project config without exposing secrets.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--show-paths` | no | Include resolved project paths. |

Writes:

- nothing

Output:

```text
Config

Models
  sentences        configured
  sentence QA      configured, optional
  word draft       configured
  word quality     configured

Runtime policy
  auto switch      off
  max loaded       1
```

Config lookup order:

1. `--config <path>`
2. `hindi.toml` in the project root
3. built-in safe defaults

Secrets do not belong in `hindi.toml`. Provider credentials, if ever needed,
come from environment variables or provider-specific local config.

## `hindi sources list`

Lists known source files and their user-facing source/topic values.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--type sentences|words` | no | Limit to one source type. |

Writes:

- nothing

Output:

```text
Sources

Type       Source          Topic       File
sentences  Complete Hindi  Chapter 02  input/sentences/complete_hindi_chapter_02_sentences.yaml
sentences  Complete Hindi  Chapter 03  input/sentences/complete_hindi_chapter_03_sentences.yaml
words      Complete Hindi  Chapter 02  input/words/complete_hindi_chapter_02_words.yaml
```

## `hindi models status`

Shows model-runtime status in user terms. It may mention provider names in
status rows, but provider names are not part of the command vocabulary.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--workflow <name>` | no | Limit status to one workflow, such as `sentences`. |

Writes:

- nothing

Output:

```text
Model Status

Runtime
  provider         Ollama
  reachable        yes
  loaded           translategemma:12b
  memory policy    one loaded model

Workflows
  sentences        ready
  sentence QA      not loaded
  words draft      not loaded
  words quality    not loaded

Next
  hindi sentences check --batch-size 3 --max-batches 1
```

## `hindi models prepare <workflow>`

Prepares the local model runtime for a user-facing workflow. This command hides
internal role names. Config maps `sentences` or `words-quality` to the actual
provider/model.

Workflows:

| Workflow | Meaning |
|---|---|
| `sentences` | Prepare sentence generation. |
| `source-qa` | Prepare sentence source QA. |
| `words-draft` | Prepare faster draft word generation. |
| `words-quality` | Prepare slower higher-quality word generation. |

Inputs:

- required: `<workflow>`

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--allow-model-switch` | no | Allow stopping the currently loaded model and loading the needed one. |
| `--dry-run` | no | Show what would switch without switching. |
| `--timeout <duration>` | no | Maximum time to wait for a model to become ready. |

Writes:

- may start or stop local model runtime processes after confirmation or
  `--allow-model-switch`
- does not write project data

Prompt:

```text
Model Prepare

Workflow         sentences
Current runtime  word quality model is loaded
Needed runtime   sentence generation model
Policy           one loaded model

This will stop the current model and prepare sentence generation.

Choose
  [y] switch models
  [n] cancel
```

Output:

```text
Model Prepare

Workflow         sentences
runtime          ready
loaded           sentence generation model
provider         Ollama

Next
  hindi sentences generate --batch-size 3 --max-batches 1
```

Already loaded output:

```text
Model Prepare

Workflow         sentences
runtime          ready
loaded           sentence generation model
changed          no

Next
  hindi sentences generate --batch-size 3 --max-batches 1
```

`models prepare source-qa` and `models prepare sentences` are separate on
purpose. The CLI should not hide a source-QA model switch inside generation on a
single-model memory policy.

`source-qa` appears as a model workflow because it prepares the model runtime.
The actual QA command remains under `hindi sentences source-qa` because it reads
and may repair sentence source YAML.

## `hindi sentences check`

Previews pending sentence work without model calls.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to one source YAML file under `input/sentences/`. |
| `--batch-size <n>` | no | Items per generated batch. |
| `--max-items <n>` | no | Maximum source items planned for this run. |
| `--max-batches <n>` | no | Maximum output batches planned across the whole run, not per source file. |

Writes:

- nothing

Output:

```text
Sentence Check

Source                        Existing batches  Done items  Plan items  Deferred items
Complete Hindi / Chapter 02   4                 20          3           4

Plan preview
  batch size      3
  max batches     1 total across this command
  output          output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Preview
  Hindi   क्या बात है?
  Roman   kyā bāt hai?
  English What's the matter?

Next
  hindi models prepare sentences
  hindi sentences generate --batch-size 3 --max-batches 1
```

Audio gaps are intentionally not shown in this table. Use
`hindi sentences audio check` for audio readiness.

`hindi sentences check` never writes a plan file. Generation re-derives pending
work from current YAML and accepted output at runtime. This avoids stale plan
files after source edits.

## `hindi sentences source-qa`

Checks sentence source YAML for likely input problems before generation. This is
separate from generation so the QA model and generation model do not have to be
loaded at the same time on a memory-limited machine.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to one source YAML file. |
| `--batch-size <n>` | no | Items per QA batch. |
| `--max-items <n>` | no | Maximum source items checked. |
| `--max-batches <n>` | no | Maximum source batches checked across the whole run. |
| `--max-issues <n>` | no | Stop after reporting this many issues. A capped run is never considered a clean QA pass. |
| `--apply` | no | Apply accepted fixes after confirmation. |

Writes:

- `runs/source-qa/<run-id>/report.json`
- may update `input/sentences/*.yaml` only after explicit approval

Output:

```text
Sentence Source QA

source files     1
items checked    7
issues found     2
report           runs/source-qa/20260508_143000_ollama_gemma4_latest/report.json

Issue 1/2
Hindi   अध्यापक जी, यहाँ कितने विद्यार्थी हैं?
Roman   adhyāpak jī, yahā̃ kitne vidyārthī haĩ?
English Teacher ji, how many students are there here?

Problem
  English sounds duplicated: "there here".

Suggested correction
  english: "Teacher ji, how many students are here?"

Choose
  [a] apply this correction
  [s] skip this correction
  [m] stop for manual edit
  [q] stop and keep report
```

Clean output:

```text
Sentence Source QA

source files     1
items checked    7
issues found     0
report           runs/source-qa/20260508_143000_ollama_gemma4_latest/report.json
status           clean

Next
  hindi models prepare sentences
  hindi sentences generate --batch-size 3 --max-batches 1 --require-source-qa
```

For many issues, the command should summarize first and then let the user review
one issue at a time. It should not force forty prompts without a summary.

Non-interactive behavior:

- without `--apply`, write the report and return success when issues are only
  advisory
- when issues are blocking, exit `3` and include the report path and suggested
  corrections
- never edit YAML in `--json` or non-interactive mode unless `--apply` is
  present
- if `--max-issues` stops the scan early, the report status is `incomplete` even
  when every reported issue was applied

Source-QA report shape:

```json
{
  "source": "input/sentences/complete_hindi_chapter_02_sentences.yaml",
  "source_fingerprint": "sha256:...",
  "selection": {
    "batch_size": 3,
    "max_items": null,
    "max_batches": 1,
    "source": "input/sentences/complete_hindi_chapter_02_sentences.yaml"
  },
  "complete": true,
  "status": "clean",
  "issues": [
    {
      "item_id": "complete_hindi_chapter_02_sentences:0001",
      "severity": "blocking",
      "field": "english",
      "current": "Teacher ji, how many students are there here?",
      "suggested": "Teacher ji, how many students are here?",
      "reason": "English sounds duplicated: \"there here\"."
    }
  ]
}
```

`--require-source-qa` matching logic:

- source file path must match the generation source selection
- source fingerprint must match the current source YAML
- selection limits must cover every source item planned for generation
- report status must be `clean`
- report `complete` must be `true`
- report must have been created by the configured source-QA workflow

If no matching clean report exists, generation exits `3` before model calls and
prints the exact `hindi sentences source-qa ...` command to run.

## `hindi sentences generate`

Generates accepted sentence-card JSON from YAML source.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to one source YAML file. |
| `--batch-size <n>` | no | Items per model batch. |
| `--max-items <n>` | no | Maximum source items processed. |
| `--max-batches <n>` | no | Maximum output batches processed across the whole run. |
| `--require-source-qa` | no | Require the latest matching clean source-QA report before generation. |
| `--review` | no | Write a review run only; do not accept output yet. |

Writes:

- `runs/sentences/<run-id>/`
- validated `output/sentences/*.json`, unless `--review` is set
- never updates source YAML

Output:

```text
Generate Sentences

workflow         sentences
runtime          ready
source QA        not required
batch size       3
planned batches  1

Batch 1/1 this run
target batch 5  complete_hindi_chapter_02_sentences
  input items     3
  model           ok  18.4s
  json            ok
  schema          ok
  write           output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Summary
  written         1 batch
  sentences       3
  run report      runs/sentences/20260508_143000_ollama_translategemma_12b/report.json
  audio           missing; run hindi sentences audio
```

Partial batch failure:

- accepted output is all-or-nothing per batch file
- if 6 of 7 items validate, no accepted batch is written
- valid candidates and rejected candidates stay under the run folder
- recovery starts with:
  `hindi sentences review runs/sentences/20260508_143000_ollama_translategemma_12b`
- after fixing source YAML, config, or prompt, rerun:
  `hindi sentences generate --batch-size 7 --max-batches 1`

Plan drift:

- `sentences check` is preview only and writes no plan file
- `sentences generate` prints its own plan summary before the first model call
- in an interactive terminal, generation asks for confirmation before model
  calls unless `--yes` is passed
- in `--json` or non-interactive mode, the caller is responsible for passing
  the desired flags directly to `sentences generate`

Runtime mismatch output:

```text
Model runtime is not ready for sentence generation.

Current
  word quality model is loaded

Needed
  sentence generation model

Run
  hindi models prepare sentences
```

`sentences generate` does not start, stop, or replace models. Use
`hindi models prepare sentences` first.

## `hindi sentences audit`

Reports drift between source YAML, accepted output, and audio references.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to one source YAML file or source stem. |
| `--fix` | no | Reserved for future explicit repairs. Not part of the first milestone. |

Writes:

- nothing by default

What it checks:

- accepted cards whose source item no longer exists
- accepted cards whose source fingerprint changed after YAML edits
- accepted cards missing `source_ref`
- output JSON that points to missing MP3 files
- MP3 files no longer referenced by accepted JSON
- duplicated accepted cards for the same source identity

Source identity:

- accepted sentence cards should carry a durable source reference:
  `source_ref.file`, `source_ref.item_id`, and `source_ref.fingerprint`
- the source fingerprint is based on the normalized triple:
  `hindi + romanisation + english`
- whitespace is collapsed before hashing
- title/subtitle and source file path are recorded as context, but the source
  item triple is the dedupe fingerprint

Output:

```text
Sentence Audit

source items     27
accepted cards   27
stale cards      1
missing lineage  20
missing audio    7
orphaned audio   1

Stale card
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json
  item 3

Hindi   क्या बात है?
Roman   kyā bāt hai?
English What's the matter?

Problem
  Source fingerprint changed after this card was generated.

Next
  Review source YAML and regenerate explicitly when repair commands exist.
```

Cards generated before `source_ref` exists are reported as `missing lineage`,
not as clean. Audit must not print `stale cards 0` as proof of cleanliness when
accepted cards have no source lineage.

## `hindi sentences audio check`

Shows missing or broken sentence audio without generating audio.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to one source stem. |

Writes:

- nothing

Output:

```text
Sentence Audio Check

complete_hindi_chapter_02_sentences_batch_05.json
  sentences       7
  linked audio    0
  missing audio   7

Next
  hindi sentences audio
```

## `hindi sentences review`

Shows a saved sentence generation review run.

Inputs:

- required: `<run-path>`

Options:

- global options only

Writes:

- nothing

Output:

```text
Review Run

run              runs/sentences/20260508_143000_ollama_translategemma_12b/
validated        1 batch
rejected         0 batches
target output    output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Next
  hindi sentences accept runs/sentences/20260508_143000_ollama_translategemma_12b/
```

## `hindi sentences accept`

Accepts validated review-run output into `output/sentences/`.

Inputs:

- required: `<run-path>`

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--dry-run` | no | Show target writes without accepting. |

Writes:

- validated `output/sentences/*.json`
- never overwrites existing output files

Output:

```text
Accept Run

run              runs/sentences/20260508_143000_ollama_translategemma_12b/
write            output/sentences/complete_hindi_chapter_02_sentences_batch_05.json
status           accepted
```

If the target output file already exists, accept fails. Use a future explicit
repair/regenerate command for replacement; accepting a run is append-only.

## `hindi sentences audio`

Generates or backfills sentence audio for accepted sentence JSON.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <file>` | no | Limit to output batches from one source stem. |
| `--missing-only` | no | Generate only missing audio. Default behavior. |
| `--repair missing-field|broken-reference|stale-file` | no | Explicitly repair one class of audio problem. |

Writes:

- MP3 files under `audio/sentences/`
- `audio` fields in accepted `output/sentences/*.json`

Output:

```text
Audio Sentences

backend          service TTS
scan             output/sentences
missing audio    7 sentences

complete_hindi_chapter_02_sentences_batch_05.json
  created         7 mp3 files
  updated         7 audio paths
```

Audio filenames should be filesystem-safe ASCII slugs derived from stable card
identity, not raw romanisation with diacritics. The accepted JSON stores the
relative path.

## `hindi runs list`

Lists saved run folders so review output does not accumulate silently.

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--type sentences|source-qa|transcribe` | no | Limit by run type. |

Writes:

- nothing

Output:

```text
Runs

Type        Run                                           Status    Age
sentences   20260508_143000_ollama_translategemma_12b     accepted  2h
source-qa   20260508_142200_ollama_gemma4_latest          clean     3h

Next
  hindi sentences review runs/sentences/20260508_143000_ollama_translategemma_12b
```

## `hindi runs clean`

Deletes old run folders after confirmation. It must never delete accepted
`output/`, `audio/`, source YAML, or export artifacts.

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--older-than <duration>` | yes | Only clean runs older than this age. |
| `--dry-run` | no | Show what would be deleted. |

Prompt:

```text
Runs Clean

matched          12 run folders
oldest           2026-04-12
newest           2026-04-28

This deletes run reports and rejected/model-output scratch data only.
It will not delete input, output, audio, exports, or viewer files.

Choose
  [y] delete these run folders
  [n] cancel
```

## `hindi words check`

Previews pending word-card work. Same planning semantics as
`hindi sentences check`, but reads `input/words/` and writes no data.

Key options:

- `--source <file>`
- `--batch-size <n>`
- `--max-items <n>`
- `--max-batches <n>`

## `hindi words generate`

Generates word-card JSON. Words are planned after sentence generation is stable.

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--mode draft|quality` | no | Draft is faster; quality is slower and more careful. |
| `--source <file>` | no | Limit to one word source YAML. |
| `--review` | no | Write a review run only. |

## `hindi transcribe check`

Checks local transcription prerequisites.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--media <path>` | no | Check one media file. |

Writes:

- nothing

Output:

```text
Transcription Check

media input      ok  media/input
backend          local Whisper configured
ffmpeg           ok

Next
  hindi transcribe run media/input/chapter_02.mp3
```

## `hindi transcribe run <media-path>`

Creates a separate transcript artifact. It does not write sentence-card output.

Inputs:

- required: `<media-path>`

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--reference <path>` | no | Optional reference text for alignment/correction. |
| `--review` | no | Save transcript for manual review before enrichment. |

Writes:

- `transcripts/raw/*.json`
- later reviewed transcripts under `transcripts/reviewed/*.json`

## `hindi anki topics`

Lists available source/topic pairs for export.

Writes:

- nothing

Output:

```text
Anki Topics

Source          Topic       Sentences  Words  Audio
Complete Hindi  Chapter 02  27         0      27 linked
```

## `hindi anki export`

Creates quick Anki export artifacts from accepted output through the CLI. The
web viewer also provides export controls for interactive selection and review;
both export paths should use the same shared export contract.

Inputs:

- no required positional arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--source <title>` | yes | Source title, such as `Complete Hindi`. |
| `--topic <subtitle>` | yes | Subtitle/topic, such as `Chapter 02`. |
| `--sentences` | no | Include sentence cards. Default yes for sentence-only sources. |
| `--words` | no | Include word cards when available. |

Writes:

- `exports/anki/*.tsv`
- `exports/anki/media/`

Exports are rebuildable projections. Re-running the same export may replace the
previous artifact for that source/topic. The command should write through a
temporary file/folder and rename into place so a failed export does not leave a
half-written artifact.

## `hindi viewer open`

Starts or opens the local web app. The viewer previews generated cards, plays
audio, supports filtering/selection, and provides interactive export controls.
It is still a consumer of accepted output; it does not own generated-card truth.

Inputs:

- no required arguments

Options:

| Option | Required | Meaning |
|---|---:|---|
| `--port <n>` | no | Preferred local port. |
| `--no-open` | no | Start server without opening a browser. |

Writes:

- viewer build/dev artifacts only
- export artifacts when the user uses viewer export controls

Output:

```text
Viewer

url              http://127.0.0.1:4321
sentences        5 batches
words            0 batches
audio            linked
exports          available
```
