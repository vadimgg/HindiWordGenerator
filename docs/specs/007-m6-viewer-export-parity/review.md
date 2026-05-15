# Review

## Planning Notes

- Viewer command should be a thin wrapper around `npm run dev`.
- Rust export should write a rebuildable artifact, not send to live Anki.
- Export must not mutate accepted output.
- The first full end-to-end test should use controlled data.

## Pre-PR Checklist

- `hindi viewer` command/help works.
- `hindi export` command/help works.
- Export artifact fields match sentence Anki field names.
- Viewer npm checks pass.
- Worktree has no generated real-data `exports/`, `output/`, or `audio/`
  changes unless the user explicitly requested a real smoke run.
