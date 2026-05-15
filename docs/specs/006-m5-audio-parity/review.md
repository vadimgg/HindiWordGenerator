# Review

## Planning Review Notes

- Keep this spec sentence-only. Word audio remains later.
- Do not hide content repair inside audio backfill.
- Real TTS backend choice is intentionally behind a trait so tests stay local
  and deterministic.
- The important safety property is "metadata-only patching": existing learner
  fields must not change.

## Pre-PR Review Checklist

- `hindi sentences audio` help and output match `cli.md`.
- Existing `audio` fields are skipped by default.
- MP3 writes use temp files and rename.
- JSON writes use temp files and rename.
- Failure before JSON patching is recoverable by rerunning the command.
- No source YAML, generation prompts, or sentence learner fields are modified.
