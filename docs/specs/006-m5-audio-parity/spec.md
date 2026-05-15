# M5 Audio Parity

## Scope

Add the first Rust audio backfill command for accepted sentence batches:
`hindi sentences audio`. The command scans `output/sentences/*.json`, creates
missing sentence MP3 files under `audio/sentences/`, and atomically patches only
missing `audio` fields into accepted JSON. It preserves the current direct
generation flow and keeps audio as a separate metadata/media enrichment step.

## Problem

M4 can create accepted sentence JSON, but those cards are incomplete for the
viewer/export workflow until they have playable audio paths. The archived Python
runtime can do this with `gTTS`, but the Rust path does not yet have an audio
boundary, a safe JSON patcher, or a sentence-scoped command.

## Goals

- Expose `hindi sentences audio` as the Rust sentence audio backfill command.
- Scan accepted sentence batches and identify entries missing usable `audio`
  metadata.
- Generate one MP3 per missing sentence using the sentence Hindi text.
- Use predictable, filesystem-safe, project-relative paths:
  `audio/sentences/<batch-stem>/<nn>_<slug>.mp3`.
- Skip entries that already have an `audio` field by default.
- Skip MP3 files that already exist at the planned path.
- Patch accepted JSON atomically and only add missing `audio` fields.
- Print a clear summary of scanned batches, generated files, patched cards, and
  skipped cards.

## Non-Goals

- No word audio command yet.
- No `--force`, `--repair`, or regeneration mode.
- No CLI-managed TTS installation.
- No transcription work.
- No viewer or export implementation work beyond preserving their existing
  audio path contract.
- No changes to learner content fields such as Hindi, romanisation, English,
  literal, register, tokens, words, source lineage, or tags.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi sentences audio` is accepted by the CLI and appears in `hindi sentences --help`. |
| AC02 | The command scans `output/sentences/*.json` and reports when no accepted sentence batches exist. |
| AC03 | Entries with existing non-empty `audio` fields are skipped by default and are not modified. |
| AC04 | Entries without `audio` get planned project-relative paths under `audio/sentences/<batch-stem>/`. |
| AC05 | Audio filenames are deterministic, ASCII-safe, and end in `.mp3`. |
| AC06 | MP3 creation goes through a temp path and rename; failed synthesis leaves no final partial MP3. |
| AC07 | Accepted JSON patching goes through temp path and rename. |
| AC08 | JSON patching only adds missing `audio` fields and preserves all existing learner content byte-for-byte at the data level. |
| AC09 | The command fails clearly if the TTS backend is unavailable or synthesis fails, naming the affected batch/card. |
| AC10 | The command is testable with a fake TTS backend; unit/integration tests do not require network access. |
| AC11 | Real backend behavior is isolated behind a trait or boundary so the Google/gTTS-style implementation can be replaced later. |

## Architecture Notes

M5 should reuse existing typed sentence schema parsing where practical, but it
must preserve accepted JSON content rather than regenerate cards. It can parse
into `serde_json::Value` for patching if that keeps unrelated fields stable and
safe.

### Files And Folders Changed

- `src/cli.rs`
- `src/main.rs`
- New Rust audio module(s), likely `src/sentence_audio.rs` and `src/tts.rs`
- Possibly shared atomic write helpers if the accepted writer cannot be reused
- `audio/sentences/` when the command runs successfully
- `output/sentences/*.json` when missing `audio` metadata is patched
- `docs/specs/006-m5-audio-parity/**`

### Workflow State Touched

- Accepted sentence JSON under `output/sentences/`
- Generated MP3 media under `audio/sentences/`
- No source YAML, run reports, model prompts, or generated sentence content

### External Effects And Reuse

- Filesystem writes:
  - temp MP3 then rename into `audio/sentences/...`
  - temp JSON then rename over the existing accepted batch after successful
    patching
- Network/process calls:
  - implementation may call a Google/gTTS-style backend, but the concrete
    backend must sit behind a testable boundary
- Existing helpers to reuse:
  - `ProjectRoot`
  - `SentenceBatch`/sentence schema types where useful
  - atomic temp-file patterns from `accepted_writer`
  - viewer audio path expectations from `viewer/src/utils/audioHelpers.ts`

## Testing Plan

### Unit Tests

- slug generation produces deterministic ASCII-safe filenames.
- scanner classifies existing audio, missing audio, and malformed batch files.
- audio path construction returns project-relative `audio/sentences/...mp3`.
- JSON patching adds only missing `audio` fields.
- fake TTS backend writes expected bytes for missing entries only.

### Integration Tests

- temp project fixture with one accepted sentence batch:
  - one entry already has `audio`
  - one entry has an existing planned MP3 but no `audio`
  - one entry needs a new MP3 and JSON patch
- command run should patch only missing metadata and create only missing files.
- failure fixture where TTS fails should not patch accepted JSON for the failed
  batch.

### Smoke Tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- sentences --help
cargo run -- sentences audio
```

The final smoke may be run only when a real TTS backend is available. Otherwise
the fake-backend tests are the required validation for this spec.

### Drift / Consistency Checks

- Confirm `hindi sentences generate` still suggests `hindi sentences audio`.
- Confirm viewer audio helpers accept the generated relative path shape.
- Confirm no code path modifies Hindi, romanisation, English, literal,
  register, tokens, words, `source_ref`, or tags during audio patching.

### Not Covered In This Spec

- Real TTS quality evaluation is deferred; this spec proves wiring and safety.
- Word audio is deferred until word generation returns to the active roadmap.
- Viewer browser smoke is deferred to M6.

## Open Questions

- Which concrete Rust-side TTS backend should be used first? The archived Python
  implementation uses `gTTS`. M5 may either call a small archived/Python helper
  as an implementation detail or use a Rust HTTP/client backend, as long as the
  boundary is replaceable and tests use a fake backend.
