# Hindi Word Generator

Hindi Word Generator turns small Hindi CSV lessons into enriched JSON flashcard
batches for a separate study/export app. It handles words and sentences,
validates generated data before writing it, and can create one Hindi MP3 per
card.

The project is built for cautious, append-only data production:

- inspect planned work before spending tokens
- skip cards that already exist in output JSON
- continue batch numbering instead of overwriting old batches
- validate strict schemas before writing generated data
- stop early after failed generation, validation, or audio work
- switch between OpenAI and Anthropic chat models

## Features

- Word-card generation from `input/words/*.csv`
- Sentence-card generation from `input/sentences/*.csv`
- Batch JSON output under `output/words/` and `output/sentences/`
- Append-only planning based on existing output files
- Prompt-hash and CSV-hash tracking in `manifest.json`
- Rich `check` preview before generation
- Strict validation for required fields, optional fields, item counts, and
  sentence token reconstruction
- Per-card audio generation with `gTTS`
- Manual QA prompts for Delhi-native review of generated batches

## Project Layout

```text
HindiWordGenerator/
  main.py                         # Main CLI: check, run, audio
  generate.py                     # LangChain generation runner
  process.py                      # Planning, validation, writing, manifest updates
  audio_generator.py              # MP3 generation and audio path backfill
  generation_prompt_words.txt     # Word generation system prompt
  generation_prompt_sentences.txt # Sentence generation system prompt
  review_prompt_words.txt         # Word QA review prompt
  review_prompt_sentences.txt     # Sentence QA review prompt
  manifest.json                   # Processed file metadata
  input/
    words/                        # Source word CSVs
    sentences/                    # Source sentence CSVs
  output/
    words/                        # Generated word JSON batches
    sentences/                    # Generated sentence JSON batches
  audio/
    words/                        # Generated word MP3s
    sentences/                    # Generated sentence MP3s
```

## Input Format

Words and sentences use the same simple CSV-like format. A chapter title is
optional and starts with `#`. Each content line is:

```text
HINDI (romanisation);English
```

Word example:

```text
# Complete Hindi Chapter 01
घर (ghar);home / house
लड़का (laṛkā);boy
```

Sentence example:

```text
# Complete Hindi Chapter 02
क्या आप कमला जी हैं? (kyā āp Kamalā jī haĩ?);Are you Kamala?
```

If a file has no chapter line, the chapter is derived from the filename. For
example, `complete_hindi_chapter_01_words.csv` becomes
`Complete Hindi Chapter 01`.

## Output Format

Each input CSV produces one JSON file per batch:

```text
input/words/complete_hindi_chapter_01_words.csv
  -> output/words/complete_hindi_chapter_01_words_batch_01.json
  -> output/words/complete_hindi_chapter_01_words_batch_02.json

input/sentences/complete_hindi_chapter_02_sentences.csv
  -> output/sentences/complete_hindi_chapter_02_sentences_batch_01.json
  -> output/sentences/complete_hindi_chapter_02_sentences_batch_02.json
```

Word batch shape:

```json
{
  "chapter": "Complete Hindi Chapter 01",
  "words": []
}
```

Sentence batch shape:

```json
{
  "chapter": "Complete Hindi Chapter 02",
  "sentences": []
}
```

After audio generation, each word or sentence object receives a relative audio
path such as:

```json
"audio": "audio/words/complete_hindi_chapter_01_words_batch_01/01_acchā.mp3"
```

## Quick Start

Use `uv run ...` so the inline script dependencies are available.

Preview all pending work:

```bash
uv run main.py check
```

Preview a small word run:

```bash
uv run main.py check --type words --batch-size 5 --max-items 10
```

Generate the same slice:

```bash
uv run main.py run --type words --batch-size 5 --max-items 10
```

Generate one sentence batch:

```bash
uv run main.py run --type sentences --max-batches 1
```

Backfill audio for existing output:

```bash
uv run main.py audio
uv run main.py audio --type words
uv run main.py audio output/words/some_batch.json
```

Open the local viewer:

```bash
cd viewer
npm install
npm run dev
```

The viewer reads live JSON from `output/words/` and `output/sentences/`, and
serves MP3s from `audio/`. Refresh the browser after generation to pick up new
batches or newly written audio paths.

## Model Configuration

The default model is read from `MODEL` and falls back to:

```text
openai:gpt-5.4-mini
```

You can override it per run:

```bash
uv run main.py run --model openai:gpt-5.4-mini
uv run main.py run --model anthropic:claude-sonnet-4-6
```

You can also store local configuration in `.env`:

```bash
OPENAI_API_KEY=your_openai_key
ANTHROPIC_API_KEY=your_anthropic_key
MODEL=openai:gpt-5.4-mini
```

Model strings use this format:

```text
provider:model-id
```

Supported providers are `openai` and `anthropic`.

## Normal Workflow

1. Add or edit CSV files in `input/words/` or `input/sentences/`.
2. Run `uv run main.py check` to inspect planned, skipped, and deferred work.
3. Start with a small bounded run using `--max-items` or `--max-batches`.
4. Inspect the generated JSON in `output/...`.
5. If quality looks good, run the remaining pending batches.
6. Use `uv run main.py audio` to backfill audio if needed.
7. Use the review prompts or reviewer agents for QA before importing downstream.

## Planning And Append-Only Behavior

Existing output JSON is the source of truth for completed cards. The planner
loads existing batches for a stem, extracts each card identity, and skips matching
CSV lines unless `--force` is used.

For words and sentences, identity is based on:

- `hindi`
- `romanisation` or `roman`
- `english`

New batches continue from the highest existing batch number. Normal runs do not
delete or rewrite old batches except when adding audio paths.

`manifest.json` still records metadata for completed stems:

- CSV content hash
- generation prompt hash
- processed timestamp
- batch count
- item count

## What `check` Shows

`main.py check` is the safest command to run before generation. It reports:

- batches that will be generated now
- items skipped because they already exist in output
- batches deferred by `--max-items` or `--max-batches`
- existing word cards missing `sound_alikes`
- existing sentence cards missing exact `tokens`
- existing cards missing `audio`

Examples:

```bash
uv run main.py check --type words
uv run main.py check --type sentences --max-batches 2
uv run main.py check --batch-size 5 --max-items 50
```

## Generation Behavior

`generate.py` runs batches in bounded parallel waves. By default, it uses a
concurrency of 15 and fail-fast behavior.

During a run it:

- scans pending work through `process.py`
- applies `--max-items` and `--max-batches`
- loads the relevant generation prompt
- creates the selected LangChain chat model
- calls the model for each batch
- parses the response as JSON
- delegates validation and writing to `process.py write`
- generates audio for each successfully written batch
- prints token usage when provider metadata is available

If a batch fails after retries, validation fails, writing fails, or audio
generation fails, the current wave finishes and later waves are skipped unless
`--no-fail-fast` is set.

## Validation

Generated JSON is validated before it is written. The validator checks:

- valid JSON object
- correct top-level shape for `words` or `sentences`
- required fields are present
- required strings are non-empty
- optional fields are omitted instead of empty or null
- no `date_added` anywhere
- no unexpected fields
- generated item count matches the planned batch size
- word `forms` do not duplicate the base Devanagari spelling
- sentence `tokens` reconstruct `hindi` and `romanisation` exactly
- sentence word tokens link back to `words[]` with valid `word_index` values

Validation failure stops the write, so bad model output should not silently enter
`output/`.

## Word Cards

Required word fields:

- `hindi`
- `romanisation`
- `english`
- `pos`
- `anki_tags`
- `syllables`
- `related_words`
- `example_sentence`

Optional fields should be omitted when not useful:

- `gender`
- `transitivity`
- `forms`
- `morphemes`
- `usage_notes`
- `delhi_note`
- `sound_alikes`
- `etymology_journey`
- `origin_note`
- `audio`

Word prompts are tuned for practical daily fluency, Delhi usage notes, simple
English explanations, and optional mnemonic `sound_alikes` from English, Russian,
or Hebrew.

## Sentence Cards

Required sentence fields:

- `hindi`
- `romanisation`
- `english`
- `literal`
- `register`
- `tokens`
- `words`
- `anki_tags`

Sentence cards have two parallel teaching layers:

- `tokens`: exact display tokens, including punctuation, used for UI mapping
- `words`: learner-friendly breakdown entries with meanings and notes

Joining all `tokens[].hindi` values must exactly reproduce `hindi`. Joining all
`tokens[].roman` values must exactly reproduce `romanisation`.

## Audio

Audio is generated with `gTTS` using the Hindi text from each card. Files are
written under:

```text
audio/words/<batch-stem>/
audio/sentences/<batch-stem>/
```

The batch JSON is then rewritten with relative `audio` paths. Because this uses
`gTTS`, audio generation depends on external network and service availability.

## Viewer

The integrated Astro viewer lives in `viewer/`. It is a local study/debug UI for
generated batches:

- Words tab for generated word cards
- Sentences tab for generated sentence cards
- Audio play buttons when a card has an `audio` field
- Search and selection UI from the older `hindiweb` prototype

The viewer intentionally reads the generator's live output directories instead
of keeping a copied `vocab/` folder. This avoids drift: run generation, refresh
the browser, and the new cards appear.

## Manual QA Review

The repo includes reviewer prompts:

- `review_prompt_words.txt`
- `review_prompt_sentences.txt`

Recommended review flow:

1. Choose the output batch files to review.
2. Use the matching review prompt for the batch type.
3. Review each batch as raw JSON.
4. Apply fixes based on the kind of issue.

For broad repeated issues, update the generation prompt and regenerate. For a
single bad card, edit the output JSON directly. Missing `delhi_note` values can
usually be added directly without re-running generation.

## Useful Options

```bash
uv run main.py check --type words
uv run main.py check --type sentences
uv run main.py check --batch-size 5
uv run main.py check --max-items 50
uv run main.py check --max-batches 2

uv run main.py run --type words --batch-size 5 --max-items 50
uv run main.py run --type sentences --max-batches 1
uv run main.py run --force
uv run main.py run --dry-run
uv run main.py run --concurrency 5
uv run main.py run --no-fail-fast
```

## Notes And Gotchas

- Run scripts with `uv run ...`, not plain `python3`.
- Normal generation is append-only; old outputs are skipped, not upgraded.
- If prompts change, new pending work uses the new prompt, but existing output
  files are not automatically rewritten.
- Existing sentence batches may be reported as missing `tokens` if they were
  generated before the current sentence schema.
- `--force` includes all CSV items in planning, but batch numbering still follows
  existing output state.
- Filename typos are reflected in derived chapter names unless the CSV contains
  an explicit `#` chapter title.
