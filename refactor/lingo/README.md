# Lingo

Lingo is a local command-line tool for turning raw language-learning material
into reviewed sentence cards with romanisation, word breakdowns, audio, a local
viewer, portable packages, and Anki exports.

It is designed for a simple workflow:

```text
raw text -> reviewed source -> enriched cards -> checks -> audio -> viewer/export
```

For Hindi, that means you can paste material such as:

```text
मैं लड़का हूँ
maĩ laṛkā hū̃
I am a boy.
```

and end up with a card that has the sentence, romanisation, English, a literal
gloss, word-by-word explanations, audio, and export-ready metadata.

## What Makes Lingo Different

Lingo does not call ChatGPT, Claude, Ollama, or any other model automatically.
Instead, it creates complete prompt packets for you.

You copy a packet into the model UI you prefer, paste the model reply back into
Lingo, and Lingo validates everything before accepting it.

This keeps the tool local and predictable:

- Your accepted card files live on your machine.
- Model replies are checked before they become canonical data.
- Audio, packages, and Anki decks are generated from accepted card JSON.
- The viewer reads the same accepted output that export uses.

## Requirements

You need:

- Rust 1.85 or newer.
- `uv` for the default gTTS audio backend.
- Node.js and npm for the local viewer.
- An editor configured with `$VISUAL` or `$EDITOR` for the default prompt loop.

Optional:

- `ELEVENLABS_API_KEY` if you want ElevenLabs audio instead of gTTS.
- `fzf` if you want to pick ElevenLabs voices from an interactive terminal list.

Check your setup with:

```bash
lingo doctor
```

## Install

From this directory:

```bash
make install
```

If Cargo already has the dependencies cached and you want to avoid network
access:

```bash
make install OFFLINE=1
```

You can also build and run without installing:

```bash
make build
make run ARGS='--help'
```

The viewer dependencies are installed separately:

```bash
cd apps/viewer
npm install
```

You only need to do that once per checkout.

## Start A New Deck

Create a folder for your deck and initialize it:

```bash
mkdir ~/decks/hindi-practice
cd ~/decks/hindi-practice
lingo init --lang hindi
```

This creates a workspace like this:

```text
config.toml
raw/
input/sentences/
output/sentences/
audio/sentences/
runs/
packages/
exports/
```

The most important folders are:

| Folder | Meaning |
| --- | --- |
| `raw/` | Your copied source text. |
| `input/sentences/` | Reviewed source YAML from `lingo import`. |
| `output/sentences/` | Accepted card JSON from `lingo build`. |
| `audio/sentences/` | MP3 files created by `lingo audio`. |
| `runs/` | Prompt packets and replies for traceability. |
| `packages/` | Portable package output. |
| `exports/` | Anki `.apkg` exports. |

## The First Workflow

This section walks through the whole process.

### 1. Add Raw Text

Put a small source file under `raw/`:

```bash
cat > raw/introduce-yourself.md <<'EOF'
I am a boy
मैं लड़का हूँ
maĩ laṛkā hū̃

I am a girl
मैं लड़की हूँ
maĩ laṛkī hū̃
EOF
```

Keep your first file small. Small batches are easier to review and cheaper to
redo.

### 2. Import Raw Text Into Reviewed Source YAML

Run:

```bash
lingo import raw/introduce-yourself.md \
  --batch introduce-yourself \
  --title "Introduce Yourself"
```

Lingo creates a prompt packet under `runs/`, tries to copy it to your clipboard,
and opens a reply file in `$VISUAL` or `$EDITOR`.

Paste the prompt into ChatGPT or Claude. Then paste the model reply into the
editor window that Lingo opened, save the file, and close the editor.

Lingo writes:

```text
input/sentences/introduce-yourself.yaml
```

That file is the reviewed source. It contains sentence text, romanisation,
English, tags, IDs, and fingerprints.

If your editor does not open, set one before running the command:

```bash
export EDITOR=nano
```

or:

```bash
export VISUAL=code
```

### 3. Build Full Study Cards

Now generate full cards:

```bash
lingo build --batch introduce-yourself
```

Lingo again creates a prompt packet, copies it if possible, and opens a reply
file in your editor.

Paste the prompt into ChatGPT or Claude. Then paste the model reply into the
editor window, save, and close the editor.

Lingo writes:

```text
output/sentences/introduce-yourself.json
```

That file is the accepted card output. It is what the viewer, audio, package,
and export steps use.

### 4. Manual Prompt And Reply Files

The editor flow above is the default user-facing flow.

If you want to manage files yourself, use `--print` and `--apply`.

Create an import prompt packet:

```bash
lingo import raw/introduce-yourself.md \
  --batch introduce-yourself \
  --title "Introduce Yourself" \
  --print > /tmp/import-packet.md
```

Apply an import reply:

```bash
lingo import raw/introduce-yourself.md \
  --batch introduce-yourself \
  --title "Introduce Yourself" \
  --apply /tmp/import-reply.yaml
```

Create a build prompt packet:

```bash
lingo build --batch introduce-yourself --print > /tmp/build-packet.md
```

Apply a build reply:

```bash
lingo build --batch introduce-yourself --apply /tmp/build-reply.json
```

### 5. Check The Cards

Run deterministic checks:

```bash
lingo check --batch introduce-yourself
```

This catches things like:

- Missing romanisation.
- Token romanisation that does not reconstruct the sentence romanisation.
- Cards that no longer match their source sentence.
- Missing audio references after the audio stage.
- Invalid or incomplete card structure.

Before audio, missing audio may appear as warnings. After audio, the batch
should be clean.

### 6. Generate Audio

Generate MP3 files:

```bash
lingo audio --batch introduce-yourself
```

The default backend is gTTS through `uv`.

Audio files are written under:

```text
audio/sentences/introduce-yourself/
```

Then check again:

```bash
lingo check --batch introduce-yourself
```

### 7. Preview In The Viewer

Start the local viewer:

```bash
lingo viewer
```

The viewer opens a local web app where you can:

- Read sentence cards.
- See Hindi and romanisation together.
- Expand word breakdowns.
- Play audio.
- Check data readiness.
- Use the old Words, Sentences, Deliver, and QA tabs.

If you do not want Lingo to open the browser automatically:

```bash
lingo viewer --no-open
```

Use another port if needed:

```bash
lingo viewer --port 5000 --no-open
```

### 8. Package Or Export

Create a portable package:

```bash
lingo package --batch introduce-yourself
```

Create an Anki package:

```bash
lingo export --batch introduce-yourself
```

Both commands require complete, accepted card data. Export also requires audio
for selected cards.

## Common Commands

| Command | What It Does |
| --- | --- |
| `lingo init --lang hindi [dir]` | Create or repair a deck workspace. |
| `lingo status` | Show what is done and what to run next. |
| `lingo doctor` | Check editor, clipboard, audio tools, Node, and workspace config. |
| `lingo import` | Turn `raw/` text into reviewed source YAML. |
| `lingo build` | Turn reviewed source YAML into full card JSON. |
| `lingo check` | Run deterministic validation. |
| `lingo audio` | Generate sentence MP3 files. |
| `lingo viewer` | Open the local review app. |
| `lingo package` | Build a portable folder with cards and audio. |
| `lingo export` | Build an Anki `.apkg`. |
| `lingo lang` | Inspect language profiles and prompt locations. |

Every command has help:

```bash
lingo <command> --help
```

For colored help even when output is captured:

```bash
lingo --help --color always
```

## Understanding The Two Model Steps

Lingo uses two model-assisted stages.

### Import

`lingo import` asks the model to segment raw material and return only:

- Target sentence.
- Romanisation.
- Natural English gloss.
- Tags.

For Hindi, the prompt asks for tilde nasalisation:

```text
मैं इंसान हूँ
maĩ insān hū̃
```

It should not produce word breakdowns yet.

### Build

`lingo build` asks the model to enrich reviewed sentences into full cards:

- Literal gloss.
- Register.
- Tokens.
- Word explanations.
- Tags.

Example token breakdown:

```text
मैं लड़का हूँ
maĩ laṛkā hū̃
```

```text
मैं    maĩ     I
लड़का  laṛkā   boy
हूँ    hū̃     am
```

Lingo validates that tokens and words line up before accepting the output.

## Audio Backends

The default audio backend is gTTS:

```toml
[audio]
backend = "gtts"
```

You can force gTTS for a command:

```bash
lingo audio --backend gtts
```

ElevenLabs is available when configured:

When creating the key in ElevenLabs, give it this access:

- Required: `text_to_speech`.
- Optional: `voices_read`, only if you want to use the API to list voices and
  copy voice IDs.

Lingo does not need write access to your workspace, voice cloning, dubbing,
speech-to-text, history, or admin settings.

Pick a voice from an interactive list and save it to the deck:

```bash
export ELEVENLABS_API_KEY="..."
lingo audio voice select
```

Pick a voice and use it only for the current audio run:

```bash
lingo audio voice select --for-run --batch introduce-yourself --force
```

If you do not have `fzf`, list voices and set one manually:

```bash
lingo audio voices
lingo audio voice set EXAVITQu4vr4xnSDxMaL
```

If you already know the voice ID, use it for one run without changing config:

```bash
lingo audio --backend elevenlabs --voice EXAVITQu4vr4xnSDxMaL
```

Check the configured voice:

```bash
lingo audio voice show
```

```toml
[audio]
backend = "elevenlabs"

[audio.elevenlabs]
voice = "9BWtsMINqrJLrRacOk9x"
model = "eleven_multilingual_v2"
api_key = "env:ELEVENLABS_API_KEY"
```

`voice` must be an ElevenLabs voice ID, not only the display name.

Then run:

```bash
lingo audio --backend elevenlabs
```

## Viewer Notes

`lingo viewer` rebuilds the Astro viewer against the current workspace before
serving it. That means the viewer reflects the deck you are currently in.

If the viewer fails because dependencies are missing, run:

```bash
cd apps/viewer
npm install
```

Then retry:

```bash
lingo viewer
```

The viewer currently keeps the old Words page working for older word data. New
Lingo sentence generation does not create word batches yet, so new sentence-only
decks will normally show:

```text
0 words
```

That is expected.

## File Safety

Lingo is conservative about accepted data:

- `input/sentences/` is produced by `import`.
- `output/sentences/` is produced by `build`.
- Invalid model replies are rejected.
- Audio and packages are derived from accepted card JSON.
- Prompt and reply files under `runs/` are advisory history, not canonical data.

Avoid editing `output/sentences/` by hand unless you know exactly what you are
fixing. If you do edit it, run:

```bash
lingo check
```

before generating audio or exporting.

## Troubleshooting

### `lingo doctor` reports missing `uv`

Install `uv`, then rerun:

```bash
lingo doctor
```

### The viewer cannot build

Install viewer dependencies:

```bash
cd apps/viewer
npm install
```

### The model reply does not apply

Open the reply file and check:

- Is it valid YAML for `import`?
- Is it valid JSON for `build`?
- Does it include the exact `format` value requested by the prompt?
- Does every build card use an exact `source_item` ID from the source packet?
- Are there extra markdown fences or explanatory text around the reply?

Then rerun the same `--apply` command.

### The cards have missing audio warnings

Run:

```bash
lingo audio
lingo check
```

### I do not know what to do next

Run:

```bash
lingo status
```

It shows the current pipeline state and the next useful command.

## Development Commands

For contributors:

```bash
make build
make run ARGS='status'
make install
make install OFFLINE=1
```

Run Rust tests:

```bash
cargo test --workspace --all-targets
```

Run viewer checks:

```bash
cd apps/viewer
npm run check
```

## More Detail

- [`ARCHITECTURE.md`](ARCHITECTURE.md) explains the Rust crate boundaries.
- `docs/architecture/files/` documents important source files.
- `apps/viewer/README.md` explains the restored Astro viewer.

## License

Licensed under either Apache-2.0 or MIT, at your option.
