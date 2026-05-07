# Architecture

## Overview

The project is split into four responsibilities:

1. Planning
2. Generation
3. Validation and persistence
4. Audio enrichment

The normal entrypoint is `main.py`, which delegates to the other modules.

For cleanup and authority rules across `input/`, `output/`, `audio/`, viewer
build products, demos, and reference material, see `DATA_SURFACES.md`.

For the learner-facing transliteration convention, see `ROMANISATION.md`.

## Modules

### `main.py`

User-facing command router.

Subcommands:
- `check`
- `run`
- `audio`

Responsibilities:
- parse CLI arguments
- delegate check preview rendering to `check_report.py`
- call the generation runner in `generate.py`
- backfill audio for existing batch files

### `check_report.py`

Planning preview and data-health report for `main.py check`.

Responsibilities:
- ask the planner for pending/skipped/deferred batches
- report existing output gaps, such as missing `sound_alikes`, `tokens`, or
  `audio`
- render the Rich preview tables used before generation

### `generate.py`

Generation orchestration runner.

Responsibilities:
- load `.env`
- build pending batch jobs from planner output
- call the LLM in bounded parallel waves
- stop early after a failed wave by default
- hand successful results to `generation_io.py`
- collect batch results and token usage for the final summary

Important design choices:
- provider and prompt mechanics live in `llm_client.py`
- `--max-items` and `--max-batches` limit spending without splitting a batch
- `fail-fast` is enabled by default to avoid wasting tokens after an error

### `generation_types.py`

Shared runtime data structures.

Responsibilities:
- define `BatchJob` for one pending model call
- define `BatchResult` for success/failure reporting

### `llm_client.py`

LangChain provider boundary.

Responsibilities:
- construct the selected model from `<provider>:<model-id>`
- load the relevant generation prompt
- build model messages
- retry transient model failures a limited number of times
- parse model responses as JSON
- extract provider token usage when available

### `generation_io.py`

Subprocess boundary for writing and audio enrichment.

Responsibilities:
- call `process.py check` when the generator needs machine-readable pending
  work
- call `process.py write` for validated output placement
- call `audio_generator.py` after each written batch
- keep process failures explicit for generation summaries

### `process.py`

CLI facade for batch planning, validated writes, and manifest updates.

Responsibilities:
- expose `check`, `write`, and `mark-done`
- route planning to `batch_planner.py`
- route schema checks to `schema_validator.py`
- write output batch files
- route manifest persistence to `manifest_store.py`

Important design choices:
- output files are the source of truth for dedupe
- planning is append-only
- validation happens before persistence

### `batch_planner.py`

Input parsing, source metadata, existing-output scanning, and pending batch
planning.

Responsibilities:
- parse input CSV files
- derive structured `BatchMetadata` from `#` / `##` headings or filenames
- inspect existing output batch files
- decide which items are already generated
- plan only the remaining items
- continue batch numbering from the highest existing batch
- detect non-contiguous existing batches

### `pipeline_config.py`

Shared constants for project paths, pipeline directories, output keys, prompt
files, and the default batch size.

### `schema_validator.py`

Strict schema validator and small fix-up layer.

Responsibilities:
- validate generated JSON before write
- reject unexpected schema drift
- reject unsafe audio paths
- remove duplicate word `forms` entries that repeat the base spelling
- raise explicit `ValidationError` messages for CLI output

### `manifest_store.py`

Manifest persistence helper.

Responsibilities:
- load and save `manifest.json`
- hash input CSV and prompt files
- record completed stems after all expected output batches are present

### `audio_generator.py`

Post-processing utility that enriches a batch file with audio.

Responsibilities:
- read a batch JSON file
- synthesize one MP3 per top-level entry
- store the files under `audio/<type>/<batch-stem>/`
- write a relative `audio` path back into each entry

## Data Flow

```text
input/*.csv
  -> batch_planner.py planning through process.py/check_report.py
  -> generate.py orchestration
  -> llm_client.py model calls
  -> generation_io.py
  -> process.py write
  -> schema_validator.py validation
  -> output/*.json
  -> generation_io.py
  -> audio_generator.py audio generation
  -> audio/*.mp3
  -> output/*.json updated with audio paths
```

## Planning Model

Planning works at the batch level, but dedupe works at the item level.

For each CSV:
- parse all items
- build `BatchMetadata` from source headings or filename fallback
- load existing output batches for the same stem
- compute item identity as:
  - `hindi`
  - `romanisation` or `roman`
  - `english`
- skip identities already present in output
- batch only the remaining items

This allows new words or sentences to be appended without regenerating old ones.

## Batch Naming

Batches are named:

```text
<stem>_batch_<nn>.json
```

Example:

```text
complete_hindi_chapter_01_words_batch_03.json
```

Batch numbering continues from the highest existing batch for that stem.

## Source Metadata

`BatchMetadata` is the structured source metadata object used across planning
and writing.

If a CSV contains a `#` line, that value becomes the source title.

If a CSV contains a `##` line, that value becomes the source subtitle.

If metadata is missing, display metadata is derived from the filename:
- remove `_word`, `_words`, `_sentence`, `_sentences`
- replace `_` and `-` with spaces
- title-case the result

Output batches must use top-level `title` and `subtitle` keys. `chapter` is not
part of the current batch contract.

## Validation Model

Validation is intentionally strict because the project is optimized to avoid
wasted tokens downstream. The validation code lives in `schema_validator.py`,
and `process.py write` is the main CLI gate that applies it before persistence.

The validator checks:
- top-level schema
- required keys
- allowed keys
- non-empty required strings
- non-empty tag lists
- correct batch item count
- no `date_added`
- no empty optional fields

Word-specific validation includes:
- `forms`
- `morphemes`
- `sound_alikes`
- `example_sentence`

Sentence-specific validation includes:
- sentence shape
- exact `tokens` reconstruction of both `hindi` and `romanisation`
- `word_index` links from exact display tokens to `words[]`
- nested `words` shape

## Failure Model

The pipeline is conservative by design.

If a batch fails generation, validation, writing, or audio generation:
- the batch is marked failed
- the current wave finishes
- the run stops before starting later waves, unless `--no-fail-fast` is used

This prevents runaway token spend.

## Why Output Is the Source of Truth

`manifest.json` is useful metadata, but it is not enough for append-only incremental generation.

The system uses output JSON as the real record of completed work because it needs to know:
- which specific items already exist
- where batch numbering currently ends
- which existing entries are missing `sound_alikes` or `audio`

This is why `main.py check` can report quality gaps in already-generated content.

## Sentence Card Model

Sentence cards intentionally store two parallel views:

- `tokens`: exact surface tokens for rendering, tapping, and text selection
- `words`: teaching-oriented breakdown entries with meanings and notes

This split prevents UI bugs where punctuation or token boundaries in the rendered
sentence do not line up cleanly with the teaching breakdown. For example, dashes,
question marks, and `।` can live in `tokens` without polluting the learner-facing
word breakdown.

## Extension Points

Good future extension points:
- stronger heuristics for weak `sound_alikes`
- backfill command for repairing old batches with improved prompts
- richer audio variants or slow-speed outputs
- per-provider model tuning
- optional review or audit sampling mode
