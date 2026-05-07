# Data Surfaces

This project has several folders that look similar during active work but have
different authority levels. Use this guide before cleanup, migration, or
refactoring work.

## Source Of Truth

These files are authored or intentionally curated. Do not delete or rewrite them
as generated output.

| Path | Authority | Allowed writers |
|---|---|---|
| `input/words/` | Source vocabulary input | Human edits, approved repair scripts |
| `input/sentences/` | Source sentence input | Human edits, approved repair scripts |
| `generation_prompt_words.txt` | Word generation contract | Human edits |
| `generation_prompt_sentences.txt` | Sentence generation contract | Human edits |
| `review_prompt_words.txt` | Word review contract | Human edits |
| `review_prompt_sentences.txt` | Sentence review contract | Human edits |
| `agents/` | Local agent pack and standards source | Human edits, approved agent-pack migrations |
| `README.md`, `ARCHITECTURE.md`, `DATA_SURFACES.md`, `BACKLOG.md` | Project docs | Human edits |

## Generated But Learner-Facing

These files are generated, but they are still important learner-facing artifacts.
They may be cleaned or regenerated only when the user approves the scope.

| Path | Produced by | Notes |
|---|---|---|
| `output/words/` | `main.py run`, `process.py write`, approved manual corrections | Source of truth for completed generated cards and dedupe |
| `output/sentences/` | `main.py run`, future transcript enrichment, `process.py write`, approved manual corrections | Source of truth for completed generated sentence cards and dedupe |
| `audio/words/` | `audio_generator.py`, `main.py audio` | MP3 files referenced by generated word JSON |
| `audio/sentences/` | `audio_generator.py`, `main.py audio` | MP3 files referenced by generated sentence JSON |
| `transcripts/reviewed/` | future `main.py transcribe align`, manual review | Reviewed transcript data; separate from sentence generation |
| `transcripts/exports/` | future `main.py transcribe export` | Standalone transcript exports such as JSON, TXT, SRT, or VTT |
| `manifest.json` | `process.py mark-done` | Metadata cache only; output JSON remains the dedupe authority |

Safe cleanup rules:

- It is safe to delete generated `output/` and `audio/` only when the user wants
  to discard generated work or regenerate from `input/`.
- If deleting `output/`, also expect dedupe history to reset because output JSON
  is the completed-card authority.
- If deleting `audio/`, keep output JSON in mind: cards may still point to audio
  paths until `main.py audio` backfills the files.
- Transcript exports are not sentence inputs. Transcript-derived cards may live
  in `output/sentences/` after enrichment and validation, with `transcript_ref`
  pointing back to the reviewed transcript segment.
- Do not edit `manifest.json` as a substitute for fixing output JSON.

## Projections And Build Artifacts

These are rebuildable projections of source or generated data.

| Path | Produced by | Cleanup rule |
|---|---|---|
| `viewer/public/audio` | `viewer/scripts/sync-audio.js` | Symlink/projection to project `audio/`; safe to recreate |
| `transcripts/raw/` | future `main.py transcribe run` | Raw backend output; safe to recreate from original media |
| `viewer/dist/` | `npm run build` | Safe to delete/rebuild |
| `viewer/.astro/` | Astro tooling | Safe to delete/rebuild |
| `viewer/node_modules/` | npm install | Safe to delete/reinstall, but can be slow |
| `__pycache__/` | Python runtime | Safe to delete |

## Read-Only Reference Material

These folders are useful for comparison or migration inspiration. They are not
runtime sources of truth for the active app.

| Path | Purpose | Rule |
|---|---|---|
| `demos-read-only/` | Previous demos and interface references | Read-only unless explicitly migrating/copying into active source |
| `reference/` | Reference agents and standards | Read-only unless explicitly adapting into local `agents/` |

## Allowed Write Paths By Tool

| Tool | Allowed writes |
|---|---|
| `main.py check` | No writes |
| `main.py run` | `output/`, `audio/`, `manifest.json` through delegated tools |
| `main.py audio` | `audio/`, existing `output/` audio fields |
| `process.py write` | One validated output batch and possibly `manifest.json` |
| `audio_generator.py` | Audio files and `audio` fields in one batch JSON |
| `viewer/scripts/sync-audio.js` | `viewer/public/audio` symlink/projection |
| `npm run build` | `viewer/dist/`, Astro cache/build artifacts |

## Before Cleanup

1. Identify whether the target is source, generated learner-facing data,
   projection/build output, or reference material.
2. If the target is source or learner-facing generated data, confirm the user’s
   intent and desired recovery path.
3. Prefer moving or archiving over deleting when the data has not been recently
   regenerated.
4. Run the relevant check afterward:

```bash
python3 scripts/check-python-contracts.py
rtk uv run main.py check --type sentences --batch-size 5 --max-batches 1
cd viewer && npm run check
```
