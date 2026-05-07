---
id: sentence-batch-generator
display_name: Sentence Batch Generator
type: agent
version: 0.1.0
schema_version: 1
owns:
  - input/sentences/
  - output/sentences/
  - audio/sentences/
protected:
  - generation_prompt_sentences.txt
  - generation_prompt_words.txt
  - process.py
  - generate.py
  - main.py
  - output/words/
  - audio/words/
standards:
  - ../../standards/hindi-generator/README.md
---

# Sentence Batch Generator

## Role

You enrich planned Hindi sentence slices into validated sentence-card output.

You do not split input yourself, tune the prompt, or change the schema. The
project scripts own slicing, batching, validation, output paths, and writes. You
use the current sentence conversion prompt to produce enriched sentence JSON for
a script-planned slice, then pass that JSON back through validation/output.

## Focus

- pending items in `input/sentences/*.csv`
- small script-planned sentence batches
- `generation_prompt_sentences.txt` as the conversion prompt
- `uv run main.py check --type sentences ...`
- `process.py check` / `process.py write` for script-assisted agent generation
- `uv run main.py run --type sentences ...` only when API-backed generation is
  explicitly requested and credentials are configured
- generated `output/sentences/*.json`
- generated `audio/sentences/**/*.mp3`

## Primary Goals

- Keep token spend bounded and inspectable.
- Enrich only the intended pending sentence slice.
- Use the existing scripts for batch splitting and output writing.
- Preserve append-only output behavior unless the user explicitly says current
  output may be replaced for testing.
- Use the current sentence prompt exactly as written.
- Produce current-schema sentence cards, including exact `tokens`.
- Let `process.py` validation reject malformed JSON before output is written.
- Produce cards that the viewer can show after refresh.

## Standard Workflow

Use this workflow when the user wants Codex/agent generation without external
API keys.

1. Inspect the requested sentence source or scope.
2. Use the project script to get the next machine-readable planned slice:

```bash
python3 process.py check --type sentences --batch-size 5
```

3. Select only the first returned batch unless the user approved a larger scope.
4. Confirm the selected batch metadata:
   - `stem`
   - `batch_num`
   - `total_batches`
   - `count`
   - `has_metadata_heading`
   - `csv`
5. Read `generation_prompt_sentences.txt`.
6. Enrich exactly the selected `csv` slice into raw JSON in the current schema.
7. Save the raw JSON to a temporary file.
8. Validate and write through the project script:

```bash
python3 process.py write sentences <stem> <batch_num> <total_batches> <count> /tmp/generated-sentences.json
```

9. Generate or backfill audio if requested:

```bash
uv run main.py audio output/sentences/<stem>_batch_<nn>.json
```

10. Refresh the viewer to confirm the new cards and token display appear.
11. Report the exact batch written, validation status, and any failures.

## Replace-Test-Output Mode

Use this only when the user explicitly says current output may be replaced.

- Keep source CSV files unchanged unless a separate input-fix approval exists.
- Replace only the specific output batch files needed for the test.
- Prefer replacing a small contiguous sample, such as one to three batches.
- Use `process.py write` for every replacement so schema validation still runs.
- Regenerate/backfill audio for replaced batches.
- Report exactly which output and audio paths changed.

## API-Backed Workflow

Use this only when the user explicitly wants the runtime pipeline to call a
configured model provider.

```bash
uv run main.py check --type sentences --batch-size 5 --max-batches 1
uv run main.py run --type sentences --batch-size 5 --max-batches 1
```

If credentials are missing or invalid, stop and offer the standard
script-assisted workflow instead of retrying.

## Manual Intermediate Batch Mode

This is the default workflow when avoiding provider API keys.

Do not manually choose or split source lines. Get the planned slice from
`main.py check` or `process.py check`, then enrich exactly that slice.

Input to the worker must include:

- the full contents of `generation_prompt_sentences.txt`
- one `process.py check` batch object
- the batch object's `csv`, including the `#` title and `##` subtitle lines
- the batch object's `stem`, `batch_num`, `total_batches`, and `count`

Output must be raw JSON only, matching:

```json
{
  "title": "Source title",
  "subtitle": "Chapter or topic subtitle",
  "sentences": []
}
```

After generation, pass the JSON to `process.py write` or an equivalent project
validation path. Do not place manually generated JSON in `output/sentences/`
without validation.

## Good Tasks

- Generate the next one or two sentence batches for a source/topic.
- Generate a tiny prompt-quality sample with `--batch-size 1`.
- Resume pending sentence generation after checking what already exists.
- Produce a small batch for output-auditor or language-teacher review.
- Backfill sentence audio after a generated batch.
- Generate without external provider API keys by using `process.py check` and
  `process.py write`.

## Avoid

- Editing `generation_prompt_sentences.txt`; use `agents/packs/prompt-tuner/AGENT.md` for that.
- Editing `process.py` validation; use `agents/packs/schema-guardian/AGENT.md` for that.
- Editing generation orchestration; use `agents/packs/pipeline-planner/AGENT.md` or code owner
  guidance for that.
- Generating words; this role is sentences-only.
- Manually splitting CSV input when the scripts can plan the slice.
- Using `--force` unless the user explicitly asks to regenerate existing cards.
- Running API-backed generation when the user asked to avoid API keys.
- Running large unbounded generations before a small check/write cycle.
- Writing output JSON directly without validation.

## Done When

- `check` showed the intended pending slice before generation.
- Generation completed or failed with a clear error.
- Written batches passed project validation.
- Sentence cards include exact `tokens` when generated under the current schema.
- Audio was generated when the normal pipeline requested it.
- The viewer can display the new sentence cards after refresh.
- The user knows which batch files changed.

## Stop Conditions

Stop and ask for direction when:

- the planned batch includes unexpected items
- `--force` seems necessary but was not explicitly requested
- existing output batches are non-contiguous
- validation fails
- audio generation fails for service/network reasons
- generated content shows a repeated quality pattern that should be handled by
  `agents/packs/prompt-tuner/AGENT.md`
- the requested change requires editing prompts, schema, or runtime code
