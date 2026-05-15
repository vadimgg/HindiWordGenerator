# Testing

## Required Automated Tests

- CLI:
  - parses `hindi viewer`;
  - parses `hindi viewer --help`;
  - parses `hindi export --source "Complete Hindi" --topic "Chapter 02"`;
  - rejects export with missing required options.
- Export:
  - filters sentence batches by title/subtitle;
  - writes a TSV artifact under temp `exports/`;
  - emits the expected Anki headers and columns;
  - builds `[sound:...]` tags from relative audio paths;
  - fails cleanly when no cards match.
- Viewer command:
  - validates command construction and missing `viewer/package.json` behavior
    without starting a long-running process.

## Required Commands

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -- --help
cargo run -- viewer --help
cargo run -- export --help
```

Viewer checks:

```bash
cd viewer && npm run check
```

If dependencies are missing, run `npm install` in `viewer/` first.

## Manual End-To-End Smoke

Run on controlled data first:

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences audio
hindi viewer
hindi export --source "Complete Hindi" --topic "Chapter 02"
```

For real project data, ask before running commands that mutate `output/` or
`audio/`.

## Not Required

- Live AnkiConnect import from Rust.
- Browser screenshot QA.
- Word export.
