# Research

## Existing Python Behavior

Archived Python audio lives at:

- `archive/python/runtime/audio_generator.py`
- `archive/python/runtime/main.py`

The archived command generated one MP3 per word or sentence entry and wrote a
relative `audio` path back into the batch JSON. For sentences, it used:

```text
audio/sentences/<batch-stem>/<index>_<safe-name>.mp3
```

The backend was `gTTS` with Hindi language code `hi`.

## Existing Viewer Behavior

The viewer already supports explicit audio paths:

- `viewer/src/utils/audioHelpers.ts`
- `viewer/src/utils/audioAssets.js`
- `viewer/scripts/check-audio-assets.js`

Cards with an `audio` field render a play button. The viewer serves project
root `audio/` through `viewer/public/audio`.

## Decision

M5 keeps the Python behavior conceptually but rewrites the command boundary in
Rust:

- sentence-only;
- missing-only by default;
- fake backend in tests;
- real TTS backend behind a boundary;
- JSON patch limited to missing `audio` fields.

## Open Research Item

The exact real TTS backend can be chosen during implementation. Calling an
archived/Python `gTTS` helper is acceptable for M5 if it is isolated behind the
Rust backend trait; a pure Rust HTTP/client backend is also acceptable.
