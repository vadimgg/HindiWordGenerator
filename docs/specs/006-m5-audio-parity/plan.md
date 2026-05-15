# Plan

## Design

Implement audio as a sentence-scoped enrichment command that reads accepted
output, plans missing audio paths, synthesizes missing MP3s behind a backend
trait, and atomically patches accepted JSON only after media writes succeed.
Keep CLI parsing thin and keep all mutation rules inside the sentence audio
domain module.

## Modules

| Module | Responsibility |
|---|---|
| `src/cli.rs` | Parse `hindi sentences audio` and update help text. |
| `src/main.rs` | Dispatch the audio command and map success/failure to exit codes. |
| `src/sentence_audio.rs` | Scan accepted sentence batches, plan missing audio, patch JSON, render command output. |
| `src/tts.rs` | TTS backend trait plus first concrete backend boundary. Tests use a fake backend. |
| `src/accepted_writer.rs` or shared helper | Reuse or extract temp-file-and-rename behavior for JSON and MP3 writes. |
| `src/sentence_schema.rs` | Reuse sentence shape and `audio` field support where helpful. |

## Operation Order

1. Discover the project root.
2. Collect `output/sentences/*.json` in deterministic order.
3. For each batch, parse JSON and inspect `sentences[]`.
4. For each sentence:
   - if `audio` is present and non-empty, mark skipped;
   - otherwise compute `audio/sentences/<batch-stem>/<nn>_<slug>.mp3`;
   - if the MP3 already exists, plan metadata patch only;
   - if the MP3 is missing, plan synthesis plus metadata patch.
5. If no missing audio is found, print a clean no-op summary.
6. Synthesize missing MP3s through the TTS backend using temp paths and rename.
7. After all required MP3s for a batch exist, patch only missing `audio` fields
   in memory.
8. Atomically replace the accepted batch JSON with the patched version.
9. Print scanned batches, generated MP3s, patched cards, skipped cards, and any
   failed batch.

Point of no return: after an MP3 has been written, the command may still fail
before JSON patching. Recovery is to rerun `hindi sentences audio`; existing
MP3s are reused and missing `audio` metadata is patched.

## Work Package Sequence

| WP | Purpose |
|---|---|
| WP02 | Review the active audio contract and archived Python behavior before code edits. |
| WP01 | Implement the accepted sentence scanner and audio plan. |
| WP03 | Add the replaceable TTS backend boundary and fake test backend. |
| WP04 | Implement temp-path MP3 writes and deterministic audio filenames. |
| WP05 | Patch accepted JSON audio metadata atomically without changing learner content. |
| WP07 | Wire `hindi sentences audio`, help text, summaries, and exit behavior. |
| WP06 | Review audio safety, parity, and drift before the PR. |

The work package IDs reflect creation order from the brief tool; follow the
purpose/dependency order above during implementation.

## Risks

| Risk | Mitigation |
|---|---|
| Audio command rewrites learner content. | Patch only missing `audio`; compare data-level fields in tests. |
| Partial MP3 files appear after failed synthesis. | Always write to temp path and rename only after backend success. |
| Network-backed TTS makes tests flaky. | Use a backend trait and fake backend for all automated tests. |
| Existing MP3 exists but JSON is missing audio. | Treat as metadata patch only and do not regenerate. |
| Filename policy drifts from viewer/export expectations. | Keep paths project-relative under `audio/sentences/` and run viewer helper checks when needed. |

## Validation

- `cargo fmt --check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo run -- sentences --help`
- `cargo run -- sentences audio` when real TTS backend is available
