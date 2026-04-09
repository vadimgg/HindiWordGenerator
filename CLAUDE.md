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
  process.py                      # File management utility (manifest, batch splitting)
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

Same format for both words and sentences. One item per line, optional chapter title first:

```
# Complete Hindi, Chapter 01
घर (ghar);home / house
लड़का (laṛkā);boy
```

```
# Complete Hindi, Chapter 01, Dialog 01
क्या आप कमला जी हैं ? (kyā āp Kamalā jī haĩ?);Are you Kamala?
```

- Lines starting with `#` → chapter title
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

Each batch file shares the same `chapter` value — the downstream tool groups by chapter.

Word batch top level: `{ "chapter": "...", "words": [ ] }`
Sentence batch top level: `{ "chapter": "...", "sentences": [ ] }`

---

## Processing workflow

When the user asks to process words or sentences, follow these steps exactly:

### 1. Check what needs processing

```bash
uv run process.py check                    # both words and sentences
uv run process.py check --type words       # words only
uv run process.py check --type sentences   # sentences only
uv run process.py check --force            # reprocess everything
```

Returns a JSON array of pending batches. Each item contains:
- `type` — `"words"` or `"sentences"`
- `stem` — input filename without extension
- `batch_num` / `total_batches`
- `chapter` — parsed from the `#` line
- `csv` — the batch content to send to the agent
- `count` — number of items in this batch

A file is pending if its CSV content or its prompt file has changed since last run.

### 2. Spawn agents in parallel

For each pending batch, spawn one agent with:
- The full contents of the relevant prompt file as the system prompt:
  - Words → `generation_prompt_words.txt`
  - Sentences → `generation_prompt_sentences.txt`
- The `csv` field from the check output as the input
- Instruction to return raw JSON only

All batches run in parallel regardless of type or file.

### 3. Write outputs

Before writing, check each agent's JSON response for these common schema violations and fix them:
- A `forms` entry whose `hindi` value matches the base `hindi` field → remove that entry
- Two `forms` entries with different Devanagari spellings incorrectly merged into one → split them
- A `forms` field on an invariable adjective (like लाल, साफ़, ख़ाली) → remove the entire field
- A `forms` field on a non-inflecting noun (like पिता, आदमी) where all forms = base word → remove it

Then write to:
```
output/words/<stem>_batch_<nn>.json
output/sentences/<stem>_batch_<nn>.json
```

### 4. Update the manifest

Once all batches for a file are written:
```bash
uv run process.py mark-done words     <stem> <total_batches> <total_items>
uv run process.py mark-done sentences <stem> <total_batches> <total_items>
```

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

Required: `hindi`, `romanisation`, `english`, `pos`, `anki_tags`, `syllables`, `collocations`, `related_words`

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
uv run process.py check --batch-size 5
```
