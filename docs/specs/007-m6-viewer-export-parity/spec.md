# M6 Viewer And Export Parity

## Scope

Make Rust-generated sentence output usable end to end: add `hindi viewer` for
serving the existing Astro preview/export app, and add `hindi export --source
... --topic ...` for scripted rebuildable Anki import artifacts. The work keeps
the existing viewer as the product UI and makes Rust provide the command
wrappers and export data path needed for the first complete workflow test.

## Problem

M4 and M5 can generate accepted sentence JSON and audio, but the Rust CLI does
not yet open the viewer or produce an export artifact. The user still has to
know viewer internals (`cd viewer`, `npm run dev`) and there is no Rust-owned
scripted export path to compare with viewer export behavior.

## Goals

- Add `hindi viewer` as the Rust command that serves the existing Astro viewer.
- Print the viewer URL and the underlying command being run.
- Keep browser opening optional in implementation if platform automation is
  awkward, but the command must at least serve and print a URL.
- Add `hindi export --source <title> --topic <subtitle>`.
- Export matching accepted sentence cards into a rebuildable artifact under
  `exports/`.
- Use the same source/topic grouping semantics as the viewer:
  `--source` matches batch `title`; `--topic` matches batch `subtitle`.
- Preserve audio fields and Anki tags in the export artifact.
- Add smoke checks for the complete Rust happy path on controlled fixture data.

## Non-Goals

- No live AnkiConnect send from Rust.
- No word export unless it is already needed by shared helpers.
- No viewer redesign.
- No browser automation requirement for CI.
- No source QA, repair, review/accept, or model switching.
- No real generation/audio run against broad project data during automated
  tests.

## Acceptance Criteria

| ID | Criteria |
|---|---|
| AC01 | `hindi viewer` is accepted by the CLI and listed in top-level help. |
| AC02 | `hindi viewer` runs the Astro viewer from `viewer/`, ensures audio symlink setup through the viewer's existing npm lifecycle, and prints the local URL. |
| AC03 | `hindi export --source <title> --topic <subtitle>` is accepted by the CLI and listed in top-level help. |
| AC04 | Export reads `output/sentences/*.json`, filters by `title`/`subtitle`, and writes a deterministic artifact under `exports/`. |
| AC05 | Export artifact includes sentence fields needed for Anki import: English, Hindi, Audio, Romanisation, Literal, Register, WordBreakdown, Topic, Tags. |
| AC06 | Export preserves explicit relative audio paths by converting them to Anki `[sound:...]` media filenames using the same path logic as the viewer. |
| AC07 | Export fails clearly when no matching source/topic cards exist. |
| AC08 | Viewer `word_id` support and legacy `word_index` fallback remain covered by viewer checks. |
| AC09 | Controlled end-to-end smoke can exercise doctor, plan, generate or fixture output, audio or fixture audio, viewer check, and export artifact creation without mutating real broad project data. |

## Architecture Notes

M6 should not rewrite the Astro app. Treat `viewer/` as the UI product and add
Rust wrappers around it. The export artifact can be a tab-separated Anki import
file first; live AnkiConnect belongs to the viewer and later scripted work.

### Files And Folders Changed

- `src/cli.rs`
- `src/main.rs`
- New Rust modules such as `src/viewer.rs` and `src/export.rs`
- Possibly shared sentence-output loading helpers
- `exports/` when export is run
- `docs/specs/007-m6-viewer-export-parity/**`

### Workflow State Touched

- Reads `output/sentences/`
- Reads `audio/` indirectly through paths
- Writes `exports/`
- Starts a viewer dev server process during `hindi viewer`

### External Effects And Reuse

- `hindi viewer` may run `npm run dev` in `viewer/`.
- `hindi viewer` may open a browser only if implementation can do so cleanly;
  otherwise it prints the URL and leaves opening to the user.
- `hindi export` writes rebuildable artifacts under `exports/`.
- Reuse viewer contracts:
  - `viewer/src/utils/loadGeneratedData.js`
  - `viewer/src/utils/audioAssets.js`
  - `viewer/src/scripts/anki/fields/sentence.js`
  - `viewer/scripts/check-loader.js`
  - `viewer/scripts/check-audio-assets.js`
  - `viewer/scripts/check-anki-preview.js`

## Testing Plan

### Unit Tests

- CLI parsing for `viewer` and `export`.
- Source/topic filter selection.
- Sentence-to-export-field mapping.
- Audio media filename conversion from relative paths.
- Missing source/topic error.

### Integration Tests

- Temp project fixture with sentence output and audio path exports to a temp
  `exports/` artifact.
- Viewer command construction can be tested without actually launching a long
  running server.

### Smoke Tests

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- viewer --help
cargo run -- export --help
cd viewer && npm run check
```

Optional manual smoke after implementation:

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
hindi viewer
hindi export --source "Complete Hindi" --topic "Chapter 02"
```

Run the full manual smoke on controlled data first, then real data only after
the user approves.

### Drift / Consistency Checks

- Confirm top-level README/happy path mentions `hindi viewer` and `hindi export`
  consistently if docs are touched.
- Confirm viewer checks still pass after any export helper changes.
- Confirm generated export artifacts are under `exports/` and safe to recreate.

### Not Covered In This Spec

- Live AnkiConnect export from Rust.
- Browser visual QA through Playwright.
- Word export parity.

## Open Questions

- Should `hindi viewer` open the browser by default in M6, or should M6 only
  print the URL and leave browser opening for a later platform-specific polish
  pass? The roadmap says open by default, but avoiding GUI side effects may be
  more reliable for the first CLI implementation.
