# lingo

`lingo` is a clean-slate, language-neutral CLI for turning raw learning material
into reviewed source data, enriched sentence cards, audio, portable packages,
and Anki decks.

```text
raw text -> source YAML -> card JSON -> deterministic checks
         -> audio -> portable package / Anki APKG
```

ChatGPT or Claude is a manual collaborator, not an embedded provider. `import`
and `build` render complete prompt packets; the user runs a packet in the model
UI of their choice and applies the reply. Rust owns the canonical schemas,
identity, validation, lineage, and publication rules.

This repository intentionally contains no prototype migration layer, legacy
`hindi` binary, `sentences <verb>` command nesting, Ollama integration, generic
plugin system, or dual file readers.

## Architecture

The workspace has seven crates with enforced inward dependencies:

```text
lingo-cli
  -> lingo-workspace-fs
  -> lingo-prompt
  -> lingo-audio
  -> lingo-artifacts
       -> lingo-application
            -> lingo-domain
```

`lingo-domain` owns canonical values and invariants. `lingo-application` owns
use-case policy and ports. Concrete filesystem, prompt, audio, and publication
mechanics stay in adapters and are wired only by `lingo-cli`.

[`ARCHITECTURE.md`](ARCHITECTURE.md) is the implementation contract. Every
listed source file links to a focused document under
[`docs/architecture/files/`](docs/architecture/files/) describing its owner,
scope, dependencies, implementation shape, evidence, and guardrails.

## Requirements

- Rust 1.85 or newer
- Node.js and npm only when rebuilding the Astro viewer
- `uv` for the default gTTS audio backend
- `ELEVENLABS_API_KEY` only when ElevenLabs is selected

## Build and verify

```bash
cargo build --workspace
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

npm --prefix apps/viewer ci
npm --prefix apps/viewer run check
npm --prefix apps/viewer run build
```

The compiled viewer entry point at `apps/viewer/dist/index.html` is committed
because `lingo-cli` embeds it in the read-only local viewer server.

## First workflow

```bash
cargo install --path crates/lingo-cli

mkdir my-deck
cd my-deck
lingo init --lang hindi
printf 'यह एक किताब है।\n' > raw/chapter-01.txt

lingo import raw/chapter-01.txt \
  --batch chapter-01 \
  --title 'Chapter 01' \
  --print > /tmp/import-packet.md

# Run the packet in ChatGPT or Claude and save the YAML reply.
lingo import raw/chapter-01.txt \
  --batch chapter-01 \
  --title 'Chapter 01' \
  --apply /tmp/import-reply.yaml

lingo build --batch chapter-01 --print > /tmp/build-packet.md

# Run the packet and save the JSON reply.
lingo build --batch chapter-01 --apply /tmp/build-reply.json

lingo check --batch chapter-01
lingo audio --batch chapter-01
lingo package --batch chapter-01
lingo export --batch chapter-01
```

Without `--print` or `--apply`, the packet is copied to the clipboard and the
reply file is opened through `$VISUAL` or `$EDITOR`. A reply is first parsed into
a boundary draft, then converted into canonical typed values, validated, and
written atomically. Invalid replies can update the advisory `runs/` journal but
cannot partially modify canonical source or card files.

## Commands

| Command | Responsibility |
| --- | --- |
| `lingo init --lang <profile> [dir]` | Create or repair a workspace |
| `lingo import` | Raw text to canonical `lingo.source/v1` YAML |
| `lingo build` | Source YAML to canonical `lingo.cards/v1` JSON |
| `lingo check` | Deterministic lineage, content, and profile checks |
| `lingo audio` | Synthesize card audio through gTTS or ElevenLabs |
| `lingo package` | Publish a checksummed portable folder |
| `lingo export` | Publish an Anki `.apkg` |
| `lingo status` | Show pipeline state and the next useful command |
| `lingo lang` | Inspect layered profiles and prompt origins |
| `lingo doctor` | Check required local capabilities |
| `lingo viewer` | Serve the read-only local card viewer |

Run `lingo <command> --help` for the complete public grammar.

## Canonical workspace

```text
config.toml
profile.toml                 # optional deck profile override
prompts/                     # optional deck prompt overrides
raw/
input/sentences/             # lingo.source/v1 YAML — canonical
output/sentences/            # lingo.cards/v1 JSON — canonical
audio/sentences/            # derived MP3 bytes
runs/                        # advisory prompt/reply journal
packages/                    # generated portable publications
exports/                     # generated Anki publications
```

Global defaults and profile overrides resolve from the XDG configuration home,
then deck-local configuration wins. Secrets are environment references only;
secret values are never serialized into configuration, reports, or run files.

## Publication guarantees

Portable packages are staged and read back before publication. Their
`manifest.json` records `lingo.package/v1`, language/display metadata, counts,
file lists, and a SHA-256 checksum for every payload file. Package creation
requires complete audio. Anki export writes a real APKG containing
`collection.anki2`, media bytes, and the Anki media map.

## License

Licensed under either Apache-2.0 or MIT, at your option.
