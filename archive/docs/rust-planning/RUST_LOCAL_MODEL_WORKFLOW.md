# Rust Local Model Workflow

This is the planned user-facing workflow for the Rust CLI. The Rust CLI does
not exist yet.

`CLI_COMMANDS.md` owns exact command names, options, prompts, and sample output.
This document is the narrative workflow guide and should not duplicate command
output.

Current working reference commands are still the archived Python commands:

```bash
uv run archive/python/runtime/main.py check --type sentences --max-batches 1
uv run archive/python/runtime/main.py run --type sentences --max-batches 1
uv run archive/python/runtime/main.py audio --type sentences
python3 archive/python/scripts/check-python-contracts.py
```

## Normal Sentence Path

```bash
hindi doctor
hindi sentences check --batch-size 3 --max-batches 1
hindi models prepare sentences
hindi sentences generate --batch-size 3 --max-batches 1
hindi sentences audio
hindi viewer open
```

`hindi sentences check` is a no-write preview. It re-derives pending work from
current `input/sentences/*.yaml` and accepted `output/sentences/*.json`.
`hindi sentences generate` re-derives the same kind of plan at runtime instead
of trusting a saved plan file.

`--max-batches` is total across one command invocation, not per source file.

## Optional Source QA

Source QA is a separate command because it may use a different local model from
generation. On a memory-limited machine this avoids hidden mid-command model
switching.

```bash
hindi models prepare source-qa
hindi sentences source-qa --batch-size 3 --max-batches 1
hindi models prepare sentences
hindi sentences generate --batch-size 3 --max-batches 1 --require-source-qa
```

`--require-source-qa` should use the latest clean source-QA report matching the
current source selection and source fingerprints. It should not require the user
to paste a timestamped report path. A matching report must cover every source
item planned for generation and must not be capped by `--max-issues`.

If source QA finds issues, it writes a report and lets the user review
suggested corrections. Source YAML is rewritten only after explicit approval.

## Data Flow

Input:

```text
input/sentences/complete_hindi_chapter_02_sentences.yaml
```

```yaml
title: Complete Hindi
subtitle: Chapter 02
items:
  - hindi: क्या आप कमला जी हैं?
    romanisation: kyā āp Kamalā jī haĩ?
    english: Are you Kamala?
    tags:
      - complete-hindi
      - chapter-02
```

Accepted output:

```text
output/sentences/complete_hindi_chapter_02_sentences_batch_01.json
audio/sentences/complete_hindi_chapter_02_sentences_batch_01/*.mp3
```

Run reports:

```text
runs/sentences/<timestamp>_<model-slug>/report.json
runs/source-qa/<timestamp>_<model-slug>/report.json
```

`model-slug` is filesystem-safe, for example
`ollama_translategemma_12b`.

## Source Identity

Sentence dedupe and stale-output detection use the normalized source triple:

```text
hindi + romanisation + english
```

Whitespace is collapsed before hashing. Title, subtitle, and source file path
are recorded as context, but the source item triple is the item fingerprint.
Accepted sentence cards should store `source_ref.file`, `source_ref.item_id`,
and `source_ref.fingerprint` so audit can detect drift even after run folders
are cleaned.

When source YAML changes after output exists, accepted output does not move
automatically. `hindi sentences audit` should report stale cards, deleted source
items, accepted cards missing source lineage, missing audio, orphaned audio, and
duplicate accepted cards.

## Failure Rules

- Generation is append-only.
- Existing accepted output files are never overwritten by normal generation.
- Batch acceptance is all-or-nothing per output batch file.
- If 6 of 7 generated candidates validate, no accepted batch is written.
- Valid and rejected candidates stay in the run folder for review.
- Audio filenames use filesystem-safe ASCII slugs from stable card identity,
  not raw romanisation with diacritics.
- If a source correction changes romanisation or Hindi text, old audio may
  become stale; `hindi sentences audit` should report the stale card/audio
  relationship before repair.

## Viewer And Export

The local web viewer is part of the product workflow, not just a debug page. It
previews generated cards, plays linked audio, supports filtering/selection, and
provides interactive export controls.

CLI export and viewer export should share the same export contract so Anki
formatting does not drift.

```bash
hindi viewer open
hindi anki topics
hindi anki export --source "Complete Hindi" --topic "Chapter 02"
```

The viewer is the preferred interactive preview/export surface. CLI export is
the scripted/headless path and should use the same export builder.

## Related Docs

- `CLI_COMMANDS.md` - public command names, options, prompts, and output.
- `RUST_LOCAL_MODEL_DESIGN.md` - implementation details.
- `RUST_MIGRATION_PLAN.md` - migration status and milestone gates.
- `DATA_SURFACES.md` - data authority and cleanup rules.
