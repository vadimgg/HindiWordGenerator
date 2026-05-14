# Data Surfaces

A data surface is a folder or file group with a specific authority level: source
input, accepted learner-facing output, generated projection, archive, or planned
media/transcript data. Use this guide before cleanup, migration, or refactoring.

## Authority Hierarchy

1. Human-curated source input under `input/`
2. Accepted generated learner data under `output/`
3. Audio referenced by accepted JSON under `audio/`
4. Rebuildable projections such as `viewer/dist/`
5. Archive/reference material under `archive/`, `reference/`, and
   `demos-read-only/`

`output/` is the completed-card authority for dedupe. Manifest-like metadata is
only audit data.

## Current Source Inputs

These are authored or intentionally curated. Do not delete or rewrite them as
generated output.

| Path | Authority | Allowed writers |
|---|---|---|
| `input/words/*.yaml` | Source vocabulary input | Human edits, approved repair scripts |
| `input/sentences/*.yaml` | Source sentence input | Human edits, approved repair scripts |
| `generation_prompt_words.txt` | Word generation contract | Human edits |
| `generation_prompt_sentences.txt` | Sentence generation contract | Human edits |
| `review_prompt_words.txt` | Word review contract | Human edits |
| `review_prompt_sentences.txt` | Sentence review contract | Human edits |
| `hindi.toml` | Planned Rust project config: model roles, runtime policy, paths | Human edits |
| `docs/` | Active project contracts and Rust workflow docs | Human edits |
| `agents/` | Active local agent packs and standards | Human edits, approved agent migrations |

YAML is the only active source input format. Legacy source files are archived at
`archive/python/legacy-input/` for audit history only.

## Accepted Learner-Facing Data

These files are generated, but they are important learner-facing artifacts.
Clean or regenerate them only with explicit scope.

| Path | Produced by | Rule |
|---|---|---|
| `output/words/` | Archived Python, planned Rust, approved corrections | Completed word-card authority |
| `output/sentences/` | Archived Python, planned Rust, transcript enrichment, approved corrections | Completed sentence-card authority |
| `audio/words/` | Archived Python audio command, planned Rust audio command | MP3s referenced by word JSON |
| `audio/sentences/` | Archived Python audio command, planned Rust audio command | MP3s referenced by sentence JSON |
| `runs/` | Planned Rust generation commands | Durable run reports, raw model outputs, validation results |
| `runs/source-qa/` | Planned Rust source-QA command | Source-QA reports and suggested corrections |
| `runs/sentences/` | Planned Rust sentence generation/review commands | Raw model output, validation details, accepted/rejected status |
| `exports/anki/` | Planned Rust export command or viewer export UI | Rebuildable Anki export artifacts |

Safe cleanup rules:

- Deleting `output/` discards accepted generated cards and resets dedupe.
- Deleting `audio/` leaves accepted JSON in place but may break playback until
  audio is backfilled.
- Do not edit archive manifests as a substitute for fixing accepted JSON.

## Current Write Paths

| Tool | Current writes |
|---|---|
| `uv run archive/python/runtime/main.py check ...` | No writes |
| `uv run archive/python/runtime/main.py run ...` | Validated output batches and audio paths after generation/audio backfill |
| `uv run archive/python/runtime/main.py audio ...` | `audio/` files and output JSON `audio` fields |
| `python3 archive/python/scripts/check-python-contracts.py` | No writes |
| Astro viewer build/dev commands | Viewer build artifacts only |

Archived Python commands are for parity checks and controlled fallback while
Rust is being built. They still read project-level `input/` and write
project-level `output/`/`audio/`.

## Planned Rust Write Paths

| Planned command | Writes |
|---|---|
| `hindi doctor` | Nothing by default |
| `hindi models status` | Nothing by default |
| `hindi models prepare <workflow>` | May start/stop local models after explicit approval or `--allow-model-switch` |
| `hindi sentences check` | Nothing by default |
| `hindi sentences source-qa` | Source QA run report; source YAML only after explicit approval |
| `hindi sentences generate` | Durable run folder plus validated sentence batches; review mode withholds accepted output |
| `hindi sentences audit` | Nothing by default |
| `hindi sentences audio` | `audio/` files and output JSON `audio` fields |
| `hindi anki export ...` | Export artifacts only |
| Viewer export UI | Export artifacts only |
| `hindi repair ...` / `hindi regenerate ...` | Explicit target files only |

Normal generation must remain append-only. Existing batch files are not
rewritten unless the user runs an explicit repair or regenerate command.
Generation should always keep a durable `runs/.../report.json` record with
model identity, timing, validation, and accepted/rejected status.
Run folders are useful audit history, but source/output drift checks must not
depend on run folders existing forever. Accepted cards should include durable
source lineage (`source_ref.file`, `source_ref.item_id`,
`source_ref.fingerprint`) when Rust generation starts.

Source QA is a special approved-repair path: it may suggest changes to
`input/sentences/*.yaml`, but it writes only after interactive confirmation or a
clearly named non-interactive flag such as `--apply`. Suggested
source corrections should be recorded in the run report.

When source YAML is edited, completed output does not move automatically.
Planning should compare stable source item identity/fingerprints against
accepted output and report any orphaned or stale accepted cards explicitly.

Audio backfill is the normal exception: it may add a missing `audio` field to an
accepted card after creating the MP3. If the MP3 and JSON `audio` field already
exist, audio commands should skip the card. Overwriting existing MP3s or
rewriting existing `audio` paths requires an explicit repair/regenerate flag.
Audio commands should create MP3s through a temporary path first, verify the file
exists, then update accepted JSON through a temp-file-and-rename flow. If JSON
update fails after MP3 creation, report the orphaned MP3 path so cleanup is
intentional.
Audio filenames should use filesystem-safe ASCII slugs from stable card
identity, not raw romanisation with diacritics.

Export artifacts under `exports/anki/` are rebuildable projections, not accepted
learner data. Export commands may replace an export for the same source/topic,
but should write through a temporary path and rename into place.

## Planned Media And Transcript Paths

Transcription is optional and separate from YAML source input.

| Path | Purpose | Rule |
|---|---|---|
| `media/input/` | User-supplied audio/video for local Whisper transcription | Human-managed source media |
| `transcripts/raw/` | Raw local transcription output | Generated, not learner-facing |
| `transcripts/reviewed/` | Reviewed transcript segments | Curated transcript source for optional enrichment |
| `transcripts/references/` | Optional reference text for alignment | Human-curated reference material |

Transcript-derived cards still land in `output/sentences/`. Accepted cards may
store a lightweight transcript reference, but timings and segment details stay
in transcript files.

## Projections And Build Artifacts

These are rebuildable projections.

| Path | Cleanup rule |
|---|---|
| `viewer/public/audio` | Safe to recreate |
| `viewer/dist/` | Safe to delete/rebuild |
| `viewer/.astro/` | Safe to delete/rebuild |
| `viewer/node_modules/` | Safe to delete/reinstall, but can be slow |
| `__pycache__/` | Safe to delete |
| `exports/anki/` | Safe to recreate from accepted output |

## Archive And Reference Material

| Path | Purpose | Rule |
|---|---|---|
| `archive/python/` | Previous Python runtime, scripts, tests, experiments | Reference/fallback only unless explicitly in scope |
| `archive/python/legacy-input/` | Legacy source files converted to YAML | Audit history only |
| `archive/docs/` | Python-era architecture and planning docs | Reference only |
| `archive/agents/` | Python-specific agent packs and standards | Reference only |
| `demos-read-only/` | Previous demos and interface references | Read-only unless explicitly migrating |
| `reference/` | Reference agents and standards | Read-only unless explicitly adapting |

## Cleanup Checklist

Before deleting or moving data:

```bash
find input output audio docs agents archive viewer demos-read-only reference -maxdepth 2 -type f | sort
```

Then classify each path:

1. source of truth
2. accepted learner-facing data
3. generated projection/build artifact
4. archive/reference material
5. planned media/transcript data

If unsure, do not delete it.
