# Transcription Plan

This document plans local Hindi audio transcription before implementation. The
goal is to turn lesson audio or video audio into reviewable transcript data and,
when useful, generate normal sentence cards that link back to the transcript.

## Goals

- Run transcription locally without an API key.
- Accept Hindi audio files and extracted video audio.
- Produce timestamped transcript segments.
- Prefer word-level timestamps when the backend supports them well enough.
- Compare the transcript against a reference text when one is available.
- Export transcription results as their own standalone artifacts, separate from
  the sentence-generation pipeline.
- Generate normal sentence-card output from selected transcript segments, with
  the enrichment agent filling romanisation, English, literal meaning, tokens,
  and word breakdown.
- Keep transcript-derived cards in `output/sentences/` with the rest of the
  sentence cards.
- Link transcript-derived cards back to the transcript with a minimal
  `transcript_ref`.

## Non-Goals For The First Pass

- Do not generate enriched flashcards directly from raw audio.
- Do not overwrite existing `input/sentences/` files automatically.
- Do not make transcription a requirement for sentence generation.
- Do not require transcript-derived cards to pass through the CSV sentence input
  workflow.
- Do not require OpenAI, Anthropic, or any remote API.
- Do not promise perfect word timestamps; treat them as review aids.

## Recommended Local Backend Strategy

Use a small adapter layer so the project can switch transcription engines
without changing the rest of the workflow.

Start with one backend:

| Backend | Why consider it | Watch-outs |
|---|---|---|
| `whisper.cpp` | Strong local CLI, Apple Silicon friendly, easy to keep outside Python dependencies | Need a small wrapper and a model download step |
| `faster-whisper` | Good Python API, supports segment and word timestamp data | Python dependency stack is heavier |
| `mlx-whisper` | Built for Apple Silicon and likely fastest on an M-series Mac | Need to verify timestamp and JSON behavior before committing |
| OpenAI `whisper` package | Canonical open-source Python implementation | Usually slower locally than newer optimized options |

For this project, I would spike `whisper.cpp` first if we want a durable local
CLI tool, and `faster-whisper` first if we want the cleanest Python integration.
Because your machine is an M4 MacBook Pro with 46 GB RAM, larger local models are
reasonable, but the first implementation should work with a smaller model too.

## Should Whisper Be Installed Separately?

Yes, eventually. Local Whisper is free/open-source, but the runtime and model
files still need to be installed or downloaded on the machine.

Do not install it yet unless you want to experiment manually. The project should
first define a `transcribe` command and adapter contract. After that, installing
one backend is straightforward and we can keep the project from depending on
every Whisper variant at once.

Minimum likely prerequisites:

- `ffmpeg` for audio extraction and format conversion.
- One local Whisper runtime.
- One downloaded Whisper model file or model cache.

## Proposed Data Surfaces

Add these folders when implementation starts:

| Path | Purpose | Authority |
|---|---|---|
| `media/input/` | Optional local audio/video source files | Human-curated source |
| `transcripts/reference/` | Optional plain-text reference transcripts | Human-curated source |
| `transcripts/raw/` | Raw backend transcript output | Generated, reviewable |
| `transcripts/reviewed/` | Corrected transcript or accepted reference alignment | Human-approved generated data |
| `transcripts/exports/` | Standalone transcript exports, such as JSON, SRT, VTT, and TXT | Generated, separate from card generation |

The current `input/sentences/` folder should remain the source of truth for
CSV-based sentence generation. Transcript-derived sentence cards do not need to
be converted into CSV rows first; they can be generated as normal sentence-card
JSON and written to `output/sentences/` after validation.

## Proposed CLI

```bash
uv run main.py transcribe check media/input/chapter_02.mp3
uv run main.py transcribe run media/input/chapter_02.mp3 --backend whisper-cpp --model large-v3-turbo
uv run main.py transcribe align transcripts/raw/chapter_02.json --reference transcripts/reference/chapter_02.txt
uv run main.py transcribe export transcripts/reviewed/chapter_02.json --format vtt
uv run main.py transcribe enrich transcripts/reviewed/chapter_02.json --title "Complete Hindi" --subtitle "Chapter 02"
uv run main.py transcribe enrich transcripts/reviewed/chapter_02.json --segment-id seg_0001
```

Command behavior:

- `check` reports `ffmpeg` availability, backend availability, media duration,
  output paths, and model choice without writing.
- `run` writes raw transcript JSON and never edits sentence inputs.
- `align` compares raw transcript text to a reference transcript and writes a
  review file with proposed corrections.
- `export` writes standalone transcript formats in `transcripts/exports/`.
- `enrich` sends selected transcript segments through the sentence-card
  enrichment workflow and writes validated batches to `output/sentences/`.

`enrich` should default to reviewed transcript segments whose `status` is
`accepted`. CLI selection should also support `--all`, `--status accepted`,
`--status needs_review`, and one or more `--segment-id` values.

## Raw Transcript Shape

Use a project-owned JSON shape even if the backend output differs:

```json
{
  "source": "media/input/chapter_02.mp3",
  "backend": "whisper-cpp",
  "model": "large-v3-turbo",
  "language": "hi",
  "segments": [
    {
      "id": "seg_0001",
      "start": 0.0,
      "end": 2.4,
      "text": "क्या आप कमला जी हैं?",
      "words": [
        { "start": 0.0, "end": 0.3, "text": "क्या" }
      ]
    }
  ]
}
```

## Reviewed Transcript Shape

Alignment and manual review should produce the same project-owned transcript
shape, with review status added to each segment:

```json
{
  "source": "media/input/chapter_02.mp3",
  "reference": "transcripts/reference/chapter_02.txt",
  "segments": [
    {
      "id": "seg_0001",
      "start": 0.0,
      "end": 2.4,
      "text": "क्या आप कमला जी हैं?",
      "status": "accepted",
      "words": [
        { "start": 0.0, "end": 0.3, "text": "क्या" }
      ]
    }
  ]
}
```

Allowed segment statuses:

- `accepted`: ready for standalone export and transcript-linked enrichment.
- `needs_review`: visible in review tools, excluded from enrichment by default.
- `rejected`: retained for traceability, excluded from export/enrichment by
  default.

Raw transcripts can omit `status`; reviewed transcripts should include it. If an
`enrich` command receives segments without status, it should require `--all` or
explicit `--segment-id` selection.

## Transcript-Linked Sentence Cards

Transcript-derived sentence cards should use the same output collection as
normal sentence cards:

```text
output/sentences/
```

They should also use the normal sentence-card schema, with one optional
provenance field:

```json
{
  "hindi": "क्या आप कमला जी हैं?",
  "romanisation": "kyā āp Kamalā jī haĩ?",
  "english": "Are you Kamala?",
  "literal": "what you Kamala ji are",
  "register": "formal",
  "tokens": [],
  "words": [],
  "anki_tags": ["transcript", "complete-hindi", "chapter-02"],
  "transcript_ref": {
    "path": "transcripts/reviewed/chapter_02.json",
    "segment_id": "seg_0001"
  }
}
```

`transcript_ref` intentionally stores only the transcript path and stable segment
ID. Timings stay in the transcript file, so cards do not go stale if alignment
or segment timing changes.

The enrichment agent may receive Hindi-only transcript segments. It is
responsible for filling romanisation, English, literal meaning, tokens, and word
breakdown just like it does for other generated sentence cards.

Transcript enrichment input is intentionally different from the CSV sentence
input format. CSV inputs include `HINDI (romanisation);English`; transcript
segments may include only Hindi text plus a segment ID. The transcript enrichment
prompt should branch explicitly for this case instead of constructing fake
romanisation or placeholder English.

`transcript_ref` validation should check only local shape and path safety during
normal schema validation:

- `path` is a non-empty project-relative path under `transcripts/`.
- `path` points to a `.json` file.
- `path` has no URL scheme, absolute path, or `..` segment.
- `segment_id` is a non-empty string.
- `transcript_ref` has no timing fields.

Do not require the transcript file to exist during normal card validation.
Resolution checks, such as confirming the file exists and `segment_id` is
present, should live in a separate audit command so ordinary schema validation
does not depend on filesystem state.

## Reference Text Correction

When a reference transcript exists, the workflow should:

1. Normalize punctuation and whitespace for comparison.
2. Align Whisper segments to the reference text.
3. Preserve timestamps from Whisper.
4. Prefer reference wording when the alignment is confident.
5. Flag low-confidence sections for manual review.
6. Never silently replace source Hindi with guessed text.

The reference text should be treated as a correction aid, not as automatic truth,
because lesson books and audio often differ slightly.

Reference transcripts live under `transcripts/reference/` for v1. The initial
format is plain UTF-8 Hindi text, preferably one sentence or utterance per line,
with no timestamps required. Richer reference formats can be added later if the
plain-text workflow proves too limiting.

## Viewer Integration

After the CLI writes transcript JSON, add a viewer tab or mode for:

- audio playback
- segment list
- click-to-seek segment timestamps
- word timestamp highlighting when available
- reference mismatch review
- standalone transcript export controls
- transcript-linked sentence-card generation controls

The transcript review surface can stay separate at first, but generated
transcript-linked cards should appear in the existing sentence-card surfaces
because they live in the same `output/sentences/` collection.

## Implementation Phases

### Phase 1: Backend Spike

- Add a small `transcription/` Python module.
- Define a backend protocol and normalized transcript schema.
- Add `ffmpeg`, backend, model, source media, and planned output availability
  checks.
- Run one short Hindi audio sample through one local backend.
- Save raw JSON under `transcripts/raw/`.

### Phase 2: Reference Alignment

- Add text normalization helpers.
- Add segment-to-reference alignment.
- Produce review JSON with accepted, corrected, and needs-review segments.
- Add tests with small Hindi fixtures.

### Phase 3: Standalone Transcript Export

- Export reviewed transcript segments to standalone transcript formats.
- Start with project JSON, plain text, and either SRT or WebVTT.
- Keep these exports independent from generated cards and Anki export.

### Phase 4: Transcript-Linked Sentence Enrichment

- Enrich selected reviewed transcript segments directly into normal sentence
  cards.
- Add optional `transcript_ref` validation to the sentence-card schema.
- Select `accepted` reviewed segments by default; support explicit CLI segment
  selection before the viewer exists.
- Write transcript-derived batches under `output/sentences/`.
- Ensure existing viewer and Anki export paths tolerate or ignore
  `transcript_ref`.

### Phase 5: Viewer Review UI

- Add a transcript review surface.
- Support playback, seek, segment selection, and mismatch flags.
- Show word timestamps when present.
- Export reviewed transcript results separately from card data.
- Offer transcript-linked sentence-card generation as an optional action.

## Open Questions

- Which backend should become the first supported local runtime?
- How strict should automatic reference correction be before requiring manual
  review?
- Would CSV-like source draft export help collaboration later, or should it stay
  out of scope unless someone needs it?
