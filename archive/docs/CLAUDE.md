# Hindi Word Generator

Generates enriched Hindi vocabulary and sentence flashcard data (JSON) from CSV input files.
The output is consumed by a separate tool the user has already built.

---

## Project structure

```
HindiWordGenerator/
  generation_prompt_words.txt     # System prompt for word cards
  generation_prompt_sentences.txt # System prompt for sentence cards
  review_prompt_words.txt         # System prompt for Delhi-native word card reviewer
  review_prompt_sentences.txt     # System prompt for Delhi-native sentence card reviewer
  process.py                      # File management utility (manifest, batch splitting, write)
  manifest.json                   # Tracks processed files by content hash + prompt hash
  input/
    words/                        # Word CSV files go here
    sentences/                    # Sentence CSV files go here
  output/
    words/                        # Generated word card JSON batches
    sentences/                    # Generated sentence card JSON batches
```

---

## Input format

Same format for both words and sentences. One item per line, source title and
optional subtitle first:

```
# Complete Hindi
## Chapter 01
घर (ghar);home / house
लड़का (laṛkā);boy
```

```
# Complete Hindi
## Chapter 01, Dialog 01
क्या आप कमला जी हैं ? (kyā āp Kamalā jī haĩ?);Are you Kamala?
```

- Lines starting with `#` → source title
- Lines starting with `##` → chapter/topic subtitle
- Content lines → `HINDI (romanisation);English`

---

## Output format

Each input file produces one JSON file per batch:

```
input/words/hindi_01.csv
  → output/words/hindi_01_batch_01.json   (items 1–10)
  → output/words/hindi_01_batch_02.json   (items 11–20)

input/sentences/hindi_01.csv
  → output/sentences/hindi_01_batch_01.json
  → output/sentences/hindi_01_batch_02.json
```

Each batch file shares the same `title` and `subtitle` values.

Word batch top level: `{ "title": "...", "subtitle": "...", "words": [ ] }`
Sentence batch top level: `{ "title": "...", "subtitle": "...", "sentences": [ ] }`

---

## Processing workflow

The easiest entrypoint is now:

```bash
uv run main.py check
uv run main.py run
```

Useful options:

```bash
uv run main.py check --type words --batch-size 5 --max-items 50
uv run main.py run --type words --batch-size 5 --max-items 50
uv run main.py run --type sentences --max-batches 1
uv run main.py run --model anthropic:claude-sonnet-4-6
uv run main.py run --model openai:gpt-5.4-mini
uv run main.py audio
```

Local API keys can live in a project-root `.env` file, for example:

```bash
OPENAI_API_KEY=your_key_here
ANTHROPIC_API_KEY=your_key_here
MODEL=openai:gpt-4o-mini
```

What the runner does:
- Uses `process.py check` to find only pending batches
- Reads the relevant generation prompt
- Calls the selected LangChain chat model in bounded parallel waves
- Validates the returned JSON schema before writing anything
- Writes one output JSON file per batch
- Generates one audio MP3 per card after each batch is written
- Writes a relative `audio` path back into each word/sentence object
- Stops early after a failed wave by default, to avoid wasting tokens
- Updates the manifest only when all batch files for a stem are present

What `main.py check` shows:
- what will be processed this run
- what will be skipped because output already exists
- what is deferred because of `--max-items` or `--max-batches`
- missing `sound_alikes` in existing word cards
- missing `audio` in existing outputs

How limits work:
- `--batch-size` controls how many input lines go into each LLM call
- `--max-items` limits the total number of input items processed in one run
- `--max-batches` limits the total number of batch files processed in one run
- `--dry-run` shows what would be processed without making API calls

How append-only processing works:
- Existing output batch files are treated as the source of truth for what has already been generated
- If a CSV has no `#` / `##` metadata lines, display metadata is derived from the filename
- New runs skip entries already present in output JSON and continue batch numbering from the highest existing batch number
- Outputs are not wiped during normal runs

What `process.py write` validates:
- Valid JSON
- Correct top-level schema for `words` vs `sentences`
- Required fields and item counts
- No `date_added`
- No empty optional fields
- Word `forms` entries whose spelling duplicates the base word are removed

Output JSON is the source of truth for completed cards. `manifest.json` records
CSV hashes, prompt hashes, timestamps, batch counts, and item counts as audit
metadata. Prompt changes affect future pending generation; existing output is
not rewritten unless an explicit repair or regeneration workflow is used.

---

## Manifest

`manifest.json` has two sections — `words` and `sentences` — each tracking per file:
- `csv_hash` — SHA-256 of the CSV content
- `prompt_hash` — SHA-256 of the relevant prompt file
- `processed_at` — ISO timestamp
- `batches` / `count`

A file is skipped if both hashes match. This means:
- **Adding items to a CSV** → that file reprocesses
- **Updating a prompt** → all files of that type reprocess
- **Neither changed** → skipped, no tokens spent

---

## Iterating on prompts

When output quality needs adjustment:
1. Edit `generation_prompt_words.txt` or `generation_prompt_sentences.txt`
2. Spawn a single test agent on one word/sentence to review the result
3. Once satisfied, run the full pipeline — changed prompt hash triggers reprocessing

---

## Manual QA review workflow

When the user asks to review a batch (or all batches of a type):

### 1. Identify the batch files to review

Find the relevant output files — e.g. all word batches for a stem, or all batches of a type:
```
output/words/<stem>_batch_<nn>.json
output/sentences/<stem>_batch_<nn>.json
```

### 2. Spawn reviewer agents in parallel

For each batch file, spawn one agent with:
- The full contents of the relevant review prompt as the system prompt:
  - Words → `review_prompt_words.txt`
  - Sentences → `review_prompt_sentences.txt`
- The full JSON content of the batch file as the input
- Instruction to return raw JSON only

All batches run in parallel.

### 3. Present the results

Show the user each reviewer's output clearly — which batch, verdict, and any issues found.

### 4. Acting on review findings

If a reviewer flags issues:
- **Pattern across multiple cards** → edit the generation prompt and reprocess
- **One-off error** → edit the output batch file directly with the correction
- **Missing `delhi_note`** → add it directly to the output batch file

Do NOT re-run the generation agent just to fix a single card unless the user asks.

---

## Word card schema (key fields)

Required: `hindi`, `romanisation`, `english`, `pos`, `anki_tags`, `syllables`, `related_words`, `example_sentence`

Optional (omit entirely when not applicable — never use null):
`gender`, `transitivity`, `forms`, `morphemes`, `usage_notes`, `delhi_note`, `sound_alikes`, `etymology_journey`, `origin_note`

## Sentence card schema (key fields)

Required: `hindi`, `romanisation`, `english`, `literal`, `register`, `words`, `anki_tags`

Each word in `words` array: `hindi`, `roman`, `meaning` always present.
Optional per word (omit when not applicable): `gender`, `number`, `note`

---

## Batch size

Default: 10 items per batch. Configurable:
```bash
python3 process.py check --batch-size 5
```
