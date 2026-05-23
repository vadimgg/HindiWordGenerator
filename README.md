# Hindi Word Generator

Hindi Word Generator is a local-first project for turning Hindi learning
material into rich flashcard data: sentence cards, word breakdowns, audio,
viewer previews, and Anki exports.

The previous Python implementation has been archived under `archive/python/`.
The active path is the Rust CLI sentence workflow, generated from YAML source
files and checked before export.

## Current Status

- Active source input is YAML under `input/`.
- Accepted learner-facing output is JSON under `output/`.
- Audio lives under `audio/` and is referenced by accepted JSON.
- The Astro viewer previews generated cards and supports export workflows.
- The Rust CLI now owns the sentence workflow; Python remains archived as the
  behavior reference during parity checks.

## Rust Happy Path

Create a workspace anywhere:

```bash
mkdir my-hindi-work
cd my-hindi-work
hindi init
```

Add sentence YAML under `input/sentences/`, then let the CLI walk you through
the process:

```bash
hindi guide
```

If you are starting from copied book or web text, put raw files under `raw/`
and run `hindi sentences raw` to open a ChatGPT/Claude conversion packet in
`$EDITOR`. After you paste valid YAML back into the editor, the CLI writes it
under `input/sentences/`.

For scripts or step-by-step debugging, the same workflow is available as
separate commands:

```bash
hindi doctor
hindi sentences raw raw/chapter.md
hindi sentences plan --max-batches 1
hindi sentences generate --max-batches 1
hindi sentences review-output
hindi sentences audio
hindi sentences package
hindi export
hindi viewer
```

`hindi sentences package` uses `[package].sentences_destination` from
`hindi.toml` when present. Use `--dest /path/to/package` to override it for a
single run.

The CLI should check whether the expected Ollama model is installed and
reachable. If it is not ready, it should print the exact `ollama run ...`
command instead of trying to manage Ollama itself.

## Current Reference Commands

Use the archived Python command when you need the project to work today:

```bash
uv run archive/python/runtime/main.py check --type sentences --max-batches 1
uv run archive/python/runtime/main.py run --type sentences --max-batches 1
uv run archive/python/runtime/main.py audio --type sentences
uv run python archive/python/scripts/check-python-contracts.py
```

## Project Shape

```text
HindiWordGenerator/
  hindi.toml                       # Optional workspace config
  agents/                         # Active local agent packs and standards
  archive/                        # Previous Python runtime, scripts, tests, docs
  docs/                           # Active design, roadmap, romanisation policy
  input/
    sentences/                    # Source sentence YAML
    words/                        # Source word YAML
  output/
    sentences/                    # Accepted generated sentence cards
    words/                        # Accepted generated word cards
  audio/
    sentences/                    # Sentence MP3s
    words/                        # Word MP3s
  viewer/                         # Astro preview/export interface
```

## Source Format

Source YAML and the output JSON schema are defined in
[docs/DESIGN.md](docs/DESIGN.md) (Source Input and Output Contract sections).
DESIGN.md is canonical; this README is intentionally a pointer to keep one
copy of the contract.

Whenever Hindi is shown in docs, CLI output, or reports, romanisation should be
shown directly underneath it.

## Active Docs

- [docs/DESIGN.md](docs/DESIGN.md) - architecture, data surfaces, model policy,
  source identity, output contract, and command shape.
- [docs/ROADMAP.md](docs/ROADMAP.md) - milestone checklist and current status.
- [docs/ROMANISATION.md](docs/ROMANISATION.md) - learner-facing romanisation
  policy.
- [docs/specs/001-m1-rust-cli-skeleton/README.md](docs/specs/001-m1-rust-cli-skeleton/README.md) -
  first Rust implementation spec for `hindi doctor`.

Older detailed planning drafts live under `archive/docs/rust-planning/` and are
reference material only.
