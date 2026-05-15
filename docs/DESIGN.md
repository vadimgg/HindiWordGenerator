# Design

Hindi Word Generator turns curated Hindi source material into learner-facing
flashcard JSON, audio references, viewer data, and Anki exports.

The current working implementation is archived Python. The next implementation
is a Rust CLI. Build sentence generation first; words, transcription, source QA,
and review runs come later.

## Current Shape

```text
input/sentences/*.yaml
  -> hindi sentences plan
  -> hindi sentences generate
  -> output/sentences/*.json
  -> hindi sentences audio
  -> audio/sentences/*.mp3
  -> viewer / Anki export
```

The end-state Rust CLI should feel small (M1 ships only `hindi doctor`; the
rest of the surface lands across M2–M6):

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
hindi viewer
```

Sentence commands are explicit from the start so word generation can be added
later without changing the command surface.

## Source Input

Active source input is YAML:

```yaml
title: Complete Hindi
subtitle: Chapter 02
items:
  - id: "0001"
    hindi: क्या आप कमला जी हैं?
    romanisation: kyā āp Kamalā jī haĩ?
    english: Are you Kamala?
    tags:
      - complete-hindi
      - chapter-02
```

Display rule: whenever Hindi is printed in docs, CLI output, reports, or errors,
print romanisation directly below it.

```text
Hindi   क्या आप कमला जी हैं?
Roman   kyā āp Kamalā jī haĩ?
English Are you Kamala?
```

## Source Identity

Rust source files should contain stable item IDs. IDs are assigned once during
YAML migration or by an explicit source repair command.

Rules:

- IDs are stable strings, not positional indexes.
- Inserting a new YAML item must not change existing IDs.
- IDs are scoped to the YAML file. Use short opaque strings such as `0001`.
- Do not embed the filename, title, or chapter in the ID; `source_ref.file`
  already records the file path.
- `source_ref.file + source_ref.item_id` identifies the source row.
- `source_ref.fingerprint` identifies the source content version. Compute it
  as: Unicode NFC-normalize each field, trim leading/trailing whitespace,
  collapse runs of internal whitespace to a single space, preserve case and
  punctuation, then hash `hindi + "\n" + romanisation + "\n" + english`.
- `item_id` is a human-stable handle for finding the item inside its source
  file; never treat it as globally unique without `source_ref.file`.

YAML files are migrated once to add `id` fields (see roadmap M1.5). After that
migration, the planner can identify items by lineage rather than by position.
Old generated JSON without `source_ref` is reported as `missing lineage` by
the planner; lineage is not backfilled onto Python-era cards.

Accepted Rust-generated sentence cards should include:

```json
"source_ref": {
  "file": "input/sentences/complete_hindi_chapter_02_sentences.yaml",
  "item_id": "0001",
  "fingerprint": "sha256:..."
}
```

Older Python-generated cards may not have `source_ref`. `hindi sentences plan`
should report those as `missing lineage`, not as clean. If a lineage-less card
matches a current source row by normalized Hindi, romanisation, and English, the
planner reports it as a content duplicate and generation stops until the old
output is archived or repaired.

## Output Contract

Accepted output lives under `output/` and is the completed-card authority.
Normal generation is append-only: do not overwrite accepted batch files.

Sentence batches:

```json
{
  "title": "Complete Hindi",
  "subtitle": "Chapter 02",
  "sentences": [
    {
      "hindi": "यहाँ",
      "romanisation": "yahā̃",
      "english": "Here.",
      "literal": "here",
      "register": "standard",
      "source_ref": {
        "file": "input/sentences/complete_hindi_chapter_02_sentences.yaml",
        "item_id": "0001",
        "fingerprint": "sha256:..."
      },
      "tokens": [
        {
          "hindi": "यहाँ",
          "roman": "yahā̃",
          "kind": "word",
          "word_id": "w1"
        }
      ],
      "words": [
        {
          "id": "w1",
          "hindi": "यहाँ",
          "roman": "yahā̃",
          "meaning": "here"
        }
      ],
      "anki_tags": []
    }
  ]
}
```

`words[]` entries carry `id` (required, unique within the card), `hindi`,
`roman`, `meaning`, plus optional `gender`, `number`, and `note`. Omit
optional fields rather than emitting `null` or `""`.

Register labels for new Rust output:

- `informal`
- `standard`
- `formal`

`tokens` and `words` contain word entries only. Do not include spaces or
punctuation in either list.

`tokens` are sentence-positioned visible word occurrences. `words` are
explanation entries within the card. Each token references exactly one
`words[]` entry by `word_id`, and every `words[]` entry should be referenced
by at least one token. If the same visible word appears twice, it should have
two tokens and may share one `words[]` explanation entry. Older Python output
may use `word_index` instead of `word_id`; new Rust output emits `word_id`
only and the viewer tolerates both during migration.

`tokens[].roman` must reconstruct the sentence `romanisation` when word tokens
are joined with the spaces and punctuation from the `romanisation` string
itself (not the Hindi text). Compare after Unicode NFC normalization. This is
a validator rule, not only a display preference.

## Model Policy

The CLI should be model-aware, not a model lifecycle manager.

Config picks the expected local model:

```toml
[models]
sentence_generation = "ollama:translategemma:12b"
```

Future model roles for source QA and word generation are intentionally not
required in the initial config.

Normal generation checks that the configured Ollama model is installed and
reachable. If it is not, the CLI should stop before spending model time and
print a direct recovery. Ollama does not expose a single "currently loaded"
model in the way the message might imply; the check is "installed and
reachable", not "in VRAM right now".

```text
Model not ready

Needed   translategemma:12b

The configured sentence model is not installed or not reachable. Run:
  ollama run translategemma:12b
```

Do not build `hindi models prepare` in the first implementation. If we later
need CLI-managed switching, earn it from real workflow pain.

## Generation

Default generation writes accepted output directly, but only after validation.

```bash
hindi sentences generate --max-batches 1
```

Rules:

- Plan from current YAML and existing output at runtime.
- Call the configured sentence model through focused stages.
- Validate JSON before writing.
- Write through temp files and rename.
- Refuse output collisions.
- Keep a run report under `runs/sentences/`.
- If a batch is partially valid, write no accepted output for that batch.

Batch filenames are assigned as the next unused zero-padded sequence number for
the source stem, for example
`output/sentences/complete_hindi_chapter_02_sentences_batch_05.json`.
If the planned target already exists, generation fails before writing.

`--max-batches` limits output files, not source items. One batch means one
accepted output JSON file; batch size is controlled by config.

Prompt contract:

- Each stage call receives one structured source row: `id`, `hindi`,
  `romanisation`, `english`, and optional `tags`. Title and subtitle are not sent — Rust
  copies them itself and the model has no need for them.
- Rust copies trusted source fields from YAML/planner data: title, subtitle,
  Hindi, romanisation, English, tags, `source_ref`, content fingerprint, and
  target filename.
- The model returns enrichment keyed by source ID in focused stages:
  `sentence/register`, `sentence/literal`, and
  `sentence/word-breakdown-from-translation`.
- Rust merges stage outputs by source ID and builds final `literal`,
  `register`, `tokens`, `words`, and optional `anki_tags`.
- Response extraction should tolerate markdown fences and leading/trailing
  prose, but validation still decides whether the result can be written.
- Missing, duplicate, or extra source IDs from any stage fail the batch before
  accepted output is written.
- A generated output file may still contain multiple accepted sentence cards,
  but model prompts are per sentence to keep local context small and reduce
  malformed multi-item responses.

If generation fails validation, inspect the run folder named in the error and
rerun `hindi sentences generate` after fixing the source, prompt, or validator.
Generation re-derives pending work from current source and output, so the failed
batch remains pending until accepted output exists.

Minimal run report:

```json
{
  "command": "sentences.generate",
  "status": "accepted | failed",
  "source_files": ["input/sentences/complete_hindi_chapter_02_sentences.yaml"],
  "targets": ["output/sentences/complete_hindi_chapter_02_sentences_batch_05.json"],
  "model": "ollama:translategemma:12b",
  "model_digest": null,
  "prompt_path": "staged-sentence-generation",
  "prompt_fingerprint": "sha256:...",
  "stages": [
    {
      "stage_id": "sentence/register",
      "prompt_version": "v3",
      "prompt_fingerprint": "sha256:...",
      "model": "ollama:translategemma:12b",
      "model_digest": null,
      "duration_ms": 3200,
      "ok": true
    },
    {
      "stage_id": "sentence/literal",
      "prompt_version": "v2",
      "prompt_fingerprint": "sha256:...",
      "model": "ollama:translategemma:12b",
      "model_digest": null,
      "duration_ms": 2100,
      "ok": true
    },
    {
      "stage_id": "sentence/word-breakdown-from-translation",
      "prompt_version": "v2",
      "prompt_fingerprint": "sha256:...",
      "model": "ollama:translategemma:12b",
      "model_digest": null,
      "duration_ms": 12400,
      "ok": true
    }
  ],
  "started_at_unix": 1778845272,
  "finished_at_unix": 1778845289,
  "validation": {
    "valid": true,
    "errors": []
  },
  "writes": {
    "accepted": [],
    "skipped": []
  }
}
```

The code owns the exact report schema. The design only requires these fields to
exist in some stable form. `model_digest` is best-effort and may be `null` when
Ollama does not expose a stable digest.

`generation_prompt_sentences.txt` remains the archived Python full-card prompt.
The old Rust full-enrichment prompt remains useful for eval comparison, but
normal Rust generation uses the staged prompt path above.

Review mode is optional and later:

```bash
hindi sentences generate --review --max-batches 1
```

Review mode saves validated candidates under `runs/` and waits for an explicit
accept command. It should not be the default path.

## Audio

Audio is a separate enrichment step:

```bash
hindi sentences audio
```

The archived Python implementation used `gTTS`, the Google Text-to-Speech
Python package. Rust should keep audio behind a boundary so the backend can
change later.

Rules:

- Default behavior creates only missing audio.
- Existing MP3s and existing JSON `audio` fields are skipped.
- Repair modes must be explicit.
- MP3 files are written through temp paths before accepted JSON is updated.
- Audio filenames should use filesystem-safe ASCII slugs.

Generation is append-only. Audio may atomically add missing `audio` metadata to
accepted JSON, but it must not change Hindi, romanisation, English, literal,
register, tokens, words, source lineage, or tags unless a future explicit repair
command is used.

## Viewer And Export

The Astro viewer is part of the product workflow. It previews generated cards,
plays audio, supports filtering/selection, and provides interactive export.
`hindi viewer` should prepare the local viewer runtime, serve the Astro app,
print the local URL, and open the browser unless a future `--no-open` flag is
passed.

CLI export is the scripted path. Viewer export and CLI export should share the
same export builder once Rust owns export generation.

Scoping rule: `doctor`, `viewer`, and `export` are cross-cutting commands.
`plan`, `generate`, and `audio` are content-type-scoped, so the first Rust
forms are `hindi sentences plan`, `hindi sentences generate`, and
`hindi sentences audio`.

First simple export shape:

```bash
hindi export --source "Complete Hindi" --topic "Chapter 02"
```

Export `--source` matches YAML `title`; `--topic` matches YAML `subtitle`.

## Data Surfaces

| Path | Role | Rule |
|---|---|---|
| `input/` | Human-curated source | Do not rewrite except explicit repair/migration. |
| `output/` | Accepted learner data | Append-only by default. |
| `audio/` | MP3s referenced by output | Backfillable, but do not overwrite by default. |
| `runs/` | Model outputs and reports | Useful audit/debug data, safe to clean intentionally. |
| `exports/` | Rebuildable projections | Safe to recreate. |
| `viewer/` | Preview/export app | Reads accepted output. |
| `archive/` | Previous implementation/docs | Reference only unless explicitly in scope. |

Mental model:

- `input/` is human source; edit intentionally.
- `output/` is accepted learner data; generation appends.
- `runs/` is diagnostics; safe to delete intentionally and never source of truth.
- `audio/` is generated media referenced by accepted output.
- `exports/` is rebuildable artifacts.
- `viewer/` reads accepted output.

## Rust Code Shape

Start with one binary crate and internal modules. Split into workspace crates
only after the code shows a real boundary.

Possible modules (sketch, not required structure — let the split emerge as
real boundaries appear):

```text
crates/
  hindi-cli/
    src/
      main.rs
      config.rs
      source.rs
      planner.rs
      schema.rs
      models.rs
      writer.rs
      sentences.rs
```

Layering: `sentences.rs` is the orchestration module. It may call `source`,
`planner`, `models`, `schema`, and `writer`; those lower-level modules should
not depend on `sentences`.

Possible later crate extraction points: model providers, export builders,
transcription, and viewer/export sharing.

## Later Features

Keep these out of the first Rust implementation:

- source QA
- review/accept loop
- repair/regenerate commands
- word generation
- local Whisper transcription
- CLI-managed model switching
