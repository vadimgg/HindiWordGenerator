# Research

## Viewer

Relevant files:

- `viewer/package.json`
- `viewer/README.md`
- `viewer/scripts/sync-audio.js`
- `viewer/src/utils/loadGeneratedData.js`
- `viewer/src/utils/audioAssets.js`
- `viewer/src/utils/audioHelpers.ts`

The viewer already reads live data from:

```text
../output/words/*.json
../output/sentences/*.json
../audio/
```

`npm run dev` runs `predev`, which calls `scripts/sync-audio.js`. That sets up
the public audio link.

## Export

Relevant files:

- `viewer/src/scripts/anki/fields/sentence.js`
- `viewer/src/scripts/anki/fields/sentenceBreakdown.js`
- `viewer/src/scripts/anki/media.js`
- `viewer/src/scripts/anki/txtFallback.js`
- `viewer/src/scripts/anki/exportService.js`
- `viewer/scripts/check-anki-preview.js`

The viewer already has live AnkiConnect export and a TXT fallback for words.
M6 Rust export should start with sentence TSV output and mirror the field names
used by `sentenceToAnkiFields`.

## Word ID Compatibility

Viewer support already exists in:

- `viewer/src/components/cards/sections/SentenceTokensSection.astro`
- `viewer/src/scripts/quality/sentenceTokens.js`
- `viewer/scripts/check-sentence-quality.js`

M6 should keep these checks passing.

## Decision

Use the viewer as-is. Add Rust command wrappers and a simple sentence TSV export
artifact. Defer live AnkiConnect and word export until there is real workflow
pressure.
