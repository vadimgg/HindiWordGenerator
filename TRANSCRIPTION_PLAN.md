# Transcription Plan

This document plans local Hindi audio transcription before implementation. The
goal is to turn lesson audio or video audio into reviewable sentence data, with
timestamps that can power the viewer.

## Goals

- Run transcription locally without an API key.
- Accept Hindi audio files and extracted video audio.
- Produce timestamped transcript segments.
- Prefer word-level timestamps when the backend supports them well enough.
- Compare the transcript against a reference text when one is available.
- Export transcription results as their own standalone artifacts, separate from
  the sentence-generation pipeline.
- Optionally promote selected reviewed transcript segments into the existing
  sentence input format later:

```text
# Complete Hindi
## Chapter 02
क्या आप कमला जी हैं? (kyā āp Kamalā jī haĩ?);Are you Kamala?
```

## Non-Goals For The First Pass

- Do not generate enriched flashcards directly from raw audio.
- Do not overwrite existing `input/sentences/` files automatically.
- Do not make transcription a requirement for sentence generation.
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
| `transcripts/raw/` | Raw backend transcript output | Generated, reviewable |
| `transcripts/reviewed/` | Corrected transcript or accepted reference alignment | Human-approved generated data |
| `transcripts/exports/` | Standalone transcript exports, such as JSON, SRT, VTT, and TXT | Generated, separate from card generation |
| `transcripts/promoted/` | Optional sentence input CSV drafts made from reviewed transcript segments | Generated draft, not source until approved |

The current `input/sentences/` folder should remain the source of truth for
sentence generation. A transcript export should not become sentence input unless
the user explicitly promotes it after review.

## Proposed CLI

```bash
uv run main.py transcribe check media/input/chapter_02.mp3
uv run main.py transcribe run media/input/chapter_02.mp3 --backend whisper-cpp --model large-v3-turbo
uv run main.py transcribe align transcripts/raw/chapter_02.json --reference references/chapter_02.txt
uv run main.py transcribe export transcripts/reviewed/chapter_02.json --format vtt
uv run main.py transcribe promote transcripts/reviewed/chapter_02.json --title "Complete Hindi" --subtitle "Chapter 02"
```

Command behavior:

- `check` reports backend availability, media duration, output paths, and model
  choice without writing.
- `run` writes raw transcript JSON and never edits sentence inputs.
- `align` compares raw transcript text to a reference transcript and writes a
  review file with proposed corrections.
- `export` writes standalone transcript formats in `transcripts/exports/`.
- `promote` writes optional draft sentence input files in `transcripts/promoted/`
  and never writes directly into `input/sentences/`.

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

## Viewer Integration

After the CLI writes transcript JSON, add a viewer tab or mode for:

- audio playback
- segment list
- click-to-seek segment timestamps
- word timestamp highlighting when available
- reference mismatch review
- standalone transcript export controls
- optional promotion controls for selected reviewed segments

This should stay separate from the current generated-card viewer at first. Once
the transcript review flow is stable, it can share card primitives and audio
controls with the existing UI.

## Implementation Phases

### Phase 1: Backend Spike

- Add a small `transcription/` Python module.
- Define a backend protocol and normalized transcript schema.
- Add backend availability checks.
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

### Phase 4: Optional Sentence Promotion

- Promote selected reviewed transcript segments to draft sentence input files.
- Write drafts under `transcripts/promoted/`, not `input/sentences/`.
- Add validation that every promoted line has Hindi text and a placeholder or
  supplied English translation.

### Phase 5: Viewer Review UI

- Add a transcript review surface.
- Support playback, seek, segment selection, and mismatch flags.
- Show word timestamps when present.
- Export reviewed transcript results separately from card data.
- Offer sentence promotion as an optional action, not the default path.

### Phase 6: Optional Enrichment Bridge

- Let approved promoted transcript drafts feed the existing sentence batch
  workflow.
- Keep enrichment, audio generation, Anki export, and QA using the existing
  generated-card pipeline.

## Open Questions

- Which backend should become the first supported local runtime?
- Should transcript reference files live under `media/reference/` or
  `transcripts/reference/`?
- Do we need English translations during transcript export, or should transcript
  exports produce Hindi-only drafts for later enrichment?
- How strict should automatic reference correction be before requiring manual
  review?
