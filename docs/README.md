# Project Docs

Hindi Word Generator turns curated Hindi learning material into validated
flashcard JSON, audio references, viewer data, and Anki exports. The current
working implementation is the Rust CLI sentence workflow; the archived Python
runtime remains available as a behavior reference.

## Current State

- Active source input is YAML under `input/words/` and `input/sentences/`.
- Accepted learner-facing output is JSON under `output/`.
- Audio lives under `audio/` and is referenced by accepted JSON.
- Python has moved to `archive/python/` and remains the behavior reference.
- The Rust CLI owns the direct sentence workflow.
- Any displayed Hindi should include romanisation directly underneath it.

## Start Here

For normal use, start with the guided Rust workflow:

```bash
hindi guide
```

For parity checks, the archived Python commands are still available:

```bash
uv run archive/python/runtime/main.py check --type sentences --max-batches 1
uv run archive/python/runtime/main.py run --type sentences --max-batches 1
uv run archive/python/runtime/main.py audio --type sentences
```

The same Rust happy path is also available as separate commands:

```bash
hindi doctor
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences review-output
hindi sentences audio
hindi sentences package --dest /tmp/hindi-sentences-package
hindi export
hindi viewer
```

## Active Docs

1. `DESIGN.md` - architecture, data surfaces, model policy, source identity,
   output contract, and command shape.
2. `ROADMAP.md` - milestone checklist and current implementation status.
3. `ROMANISATION.md` - learner-facing romanisation rules used by prompts,
   validators, repair tools, and output review.
4. `specs/001-m1-rust-cli-skeleton/README.md` - first Rust implementation spec
   for `hindi doctor`.

Older detailed Rust planning drafts live under `archive/docs/rust-planning/`.
They are reference material only; they are not the active contract.
