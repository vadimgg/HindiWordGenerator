# Testing

## Required Automated Tests

- CLI parsing:
  - `hindi sentences audio` parses successfully.
  - `hindi sentences --help` includes `audio`.
- Scanner:
  - empty `output/sentences/` returns a no-output result.
  - existing `audio` fields are skipped.
  - missing `audio` fields produce deterministic planned paths.
- Slug/path:
  - romanisation with diacritics becomes an ASCII-safe filename.
  - paths stay under `audio/sentences/` and end in `.mp3`.
- TTS:
  - fake backend writes deterministic bytes.
  - backend failure does not leave a final MP3.
- JSON patch:
  - missing `audio` is added.
  - existing `audio` is preserved.
  - learner content fields are unchanged after patching.
- Command flow:
  - temp project with one batch and fake backend creates MP3s and patches JSON.
  - command failure leaves accepted JSON unchanged for that batch.

## Required Commands

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- sentences --help
```

Run this only when a real TTS backend is configured:

```bash
cargo run -- sentences audio
```

## Manual Review

After a real audio run, inspect one generated card in JSON:

- `audio` path is project-relative.
- MP3 exists at that path.
- Hindi text and romanisation are unchanged.

When displaying a Hindi sample in review notes, always include romanisation
under it.

## Not Required

- Browser playback smoke is part of M6.
- Anki media export smoke is part of M6.
- Word audio tests are deferred until word generation is active.
