# Lingo

Lingo turns raw language-learning material into polished, review-ready study
cards — with romanisation, word-by-word breakdowns, audio, a local viewer, and
Anki export — all from your terminal.

It follows one straight pipeline:

```text
raw text  →  reviewed sentences  →  enriched cards  →  checks  →  audio  →  viewer / package / Anki
```

Paste in something like this:

```text
मैं लड़का हूँ
maĩ laṛkā hū̃
I am a boy.
```

and end up with a finished card: the sentence, its romanisation, an English
gloss, a literal gloss, a per-word explanation, audio, and export-ready
metadata.

Lingo is **language-agnostic**. Today it ships with a Hindi profile; the same
binary and the same commands build a Japanese or Spanish deck tomorrow — you
just `lingo init --lang <language>`. Everything language-specific (script,
romanisation rules, prompts, audio voice) lives in a profile, not in the code.

## How Lingo uses ChatGPT / Claude

Lingo never calls a model on its own. There is no API key for generation, no
local model server, nothing running in the background.

Instead, the two generative steps — `import` and `build` — work as a quick
**three-beat loop** built around your editor:

```text
1. EMIT      You run `lingo build`.
             Lingo writes a complete prompt packet, copies it to your
             clipboard, and opens a blank reply file in $EDITOR.

2. GENERATE  You paste the packet into ChatGPT or Claude and copy the reply.

3. APPLY     You paste the reply into the editor, save, and close.
             Lingo validates it and, only if it's valid, promotes it to your
             canonical card data. If something's off, the editor reopens with
             the problems listed inline so you fix and re-save.
```

That's the whole interaction. You stay in your editor, the prompt is already on
your clipboard, and nothing becomes "real" data until it passes validation.

This is what keeps Lingo local and trustworthy: your accepted cards live on your
machine, every model reply is checked before it's accepted, and audio, packages,
and Anki decks are all derived from that accepted card data.

> **Prefer a GUI?** The local viewer ships a **Studio** tab that runs this exact
> three-beat loop in your browser — it copies the packet for you, gives you a
> paste box with inline validation, and walks the whole `raw → import → build →
> check → audio` pipeline with buttons instead of commands. It still never calls
> a model; you paste the reply yourself. See
> [Generate in the browser: the Studio](#generate-in-the-browser-the-studio).

## Requirements

| You need | For |
| --- | --- |
| Rust 1.85+ | building / installing Lingo |
| `$EDITOR` (or `$VISUAL`) | the interactive packet loop |
| A clipboard tool (`pbcopy`, `xclip`, …) | auto-copying packets (optional but handy) |
| `uv` | the default gTTS audio backend |
| Node.js + npm | the local viewer (optional) |
| `fzf` | choosing ElevenLabs voices from an interactive list (optional) |

Optional: `ELEVENLABS_API_KEY` if you'd rather use ElevenLabs voices than gTTS.

Check everything at once:

```bash
lingo doctor
```

`doctor` tells you exactly what's missing and the command to fix it. The core
path — import → build → audio → export — needs only an editor and `uv`.

## Install

From this directory:

```bash
make install
```

Then install the viewer's dependencies once (only if you want the viewer):

```bash
cd apps/viewer
npm install
```

Prefer not to install globally? Build and run in place:

```bash
make build
make run ARGS='--help'
```

## Start a deck

A deck is just a folder. Create one and initialize it for your language:

```bash
mkdir ~/decks/hindi-practice
cd ~/decks/hindi-practice
lingo init --lang hindi
```

This scaffolds the workspace:

```text
config.toml          your learner profile, language, audio, and export settings
raw/                 text you paste in from books, articles, transcripts
input/sentences/     reviewed source YAML   (import writes here)
output/sentences/    enriched card JSON     (build writes here)
audio/sentences/     synthesized .mp3 files (audio writes here)
runs/                prompt packets + your replies (working history)
packages/            portable bundles
exports/             Anki .apkg files
```

You'll mostly think in terms of the four data folders — `raw → input → output →
audio` — which mirror the pipeline exactly.

## The walkthrough

Run everything from inside your deck folder.

### 1. Drop in some raw text

Put a small source file under `raw/`. Keep your first batch short — small
batches are easier to review and cheaper to redo.

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

### 2. Import: raw text → reviewed sentences

```bash
lingo import
```

With no arguments, Lingo grabs the next un-imported file in `raw/`, builds the
import packet, copies it to your clipboard, and opens a reply file in `$EDITOR`:

```text
Import  Introduce Yourself

  source     raw/introduce-yourself.md       (0.1 KB)
  packet     runs/import/introduce-yourself/prompt.md   ✓ copied to clipboard
  reply      runs/import/introduce-yourself/reply.yaml  opening in $EDITOR…

  ▸ Paste the packet into ChatGPT or Claude, paste the reply into the editor,
    then save and close.
```

Paste the packet into ChatGPT/Claude, copy its YAML reply, paste that into the
open editor, then **save and close**. Lingo validates and writes:

```text
input/sentences/introduce-yourself.yaml
```

That file is your reviewed source — canonical sentence text, romanisation,
English, tags, and stable IDs. Want to work a specific file instead of the next
pending one? Name it: `lingo import raw/introduce-yourself.md`.

### 3. Build: reviewed sentences → full cards

```bash
lingo build
```

Same loop. With no `--batch`, Lingo picks the next batch in `input/` that has no
up-to-date cards yet. Paste the packet into the model, paste the JSON reply into
the editor, save and close. Lingo validates against the card schema and writes:

```text
output/sentences/introduce-yourself.json
```

That JSON is your accepted card data — the source of truth for the viewer,
audio, packages, and exports.

### 4. Check the cards

```bash
lingo check
```

Deterministic validation — no model involved. It catches missing romanisation,
token romanisation that doesn't reconstruct the sentence, cards that drifted
from their source, missing audio, and malformed structure. Before audio, missing
audio shows as a warning; after audio, the batch should come back clean.

### 5. Generate audio

```bash
lingo audio
```

Synthesizes the missing clips into `audio/sentences/`. The default backend is
gTTS (via `uv`). Re-run `lingo check` afterwards — it should be all green.

### 6. Preview in the viewer

```bash
lingo viewer
```

Opens a local web app where you can read cards, see script and romanisation
together, expand word breakdowns, and play audio. It rebuilds against the deck
you're currently in, so it always reflects this workspace.

```bash
lingo viewer --no-open          # don't auto-open the browser
lingo viewer --port 5000        # use a different port
```

The viewer also has a **Studio** tab — a guided, in-browser version of the whole
generation pipeline. If you'd rather not touch the terminal for `import` /
`build` / `check` / `audio`, jump to
[Generate in the browser: the Studio](#generate-in-the-browser-the-studio).

### 7. Package or export

```bash
lingo package        # portable folder of cards + audio
lingo export         # Anki .apkg
```

Both need complete, accepted card data; export also needs audio for the cards
you're shipping.

## "What do I do next?"

You rarely have to remember the order. `lingo status` is the dashboard:

```text
lingo status

Workspace  hindi-practice          Hindi · 3 batches · 41 sentences

  BATCH                          RAW  IMPORT  BUILD  CHECK  AUDIO
  introduce_yourself              ✓     ✓       ✓      ✓      ✓     done
  chapter_02                      ✓     ✓       ✓      ✓      ●     12/17 audio
  chapter_03                      ✓     ✓       ●      –      –     needs build

Next
  lingo audio                     finish chapter 02 audio
```

Every command also ends with a `Next` hint, so you can just follow the trail.
Use `lingo status --problems` to jump straight to whatever's broken or missing.

## The scriptable alternative: `--print` and `--apply`

The editor loop is the recommended way — it's the fewest steps and keeps you in
one place. But if you'd rather drive the model by hand, script the pipeline, or
capture a packet for a prompt experiment, both `import` and `build` accept:

- `--print` — write the packet to stdout and do nothing else.
- `--apply <reply-file>` — skip the editor and ingest a saved reply directly.

```bash
# Emit the packet to a file, run it in a model UI yourself...
lingo build --batch introduce-yourself --print > packet.md

# ...then feed the saved reply back in.
lingo build --batch introduce-yourself --apply reply.json
```

Same validation, same outputs — just without the editor in the loop.

## Generate in the browser: the Studio

The **Studio** is the third way to drive the pipeline — neither the editor loop
nor hand-scripting, but a guided UI inside the viewer. It's the friendliest path
if you'd rather click than type, and it surfaces every option the `import`,
`build`, `check`, and `audio` commands accept.

Open it the same way you open the viewer, then click the **✎ Studio** tab:

```bash
lingo viewer
```

It talks only to the local viewer server, and — like everything else in Lingo —
**it never calls a model for you**. The generative steps are still the manual
copy-paste loop; the Studio just makes that loop pleasant.

### What you see

- **Pipeline rail** — the live, graphical version of `lingo status`. One row per
  batch with a dot for each stage (`raw · import · build · check · audio`), the
  same "what's next" hint, a **Problems only** filter, and a **⚙ Settings**
  drawer for the per-deck `config.toml` knobs (display lead, audio backend,
  ElevenLabs voice).
- **Stepper** — moves you through the five stages for the selected batch; stages
  you haven't earned yet are locked.

### The five stages

1. **Raw** — paste your source text, drop a file onto the box, or pick an
   existing file from `raw/`. Saving writes `raw/<batch>.md` for you (the one
   step the CLI can't do) and opens the import packet. The batch id, title, and
   subtitle fields map to `--batch`, `--title`, and `--subtitle`.
2. **Import** — the **Packet Exchange**: the import packet is shown read-only and
   auto-copied to your clipboard; paste it into ChatGPT/Claude, paste the YAML
   reply into the box, and hit **Apply**. The server validates it (exactly like
   the CLI) and writes `input/sentences/<batch>.yaml` only if it passes. Bad
   replies come back with the problems listed inline; nothing is overwritten.
3. **Build** — the same Packet Exchange for the build prompt; a valid JSON reply
   becomes `output/sentences/<batch>.json` after passing `check_card_batch`. The
   accepted cards render with their word-by-word breakdown.
4. **Check** — runs the deterministic checks and shows the report (errors block,
   the "missing audio" warning is expected until you do the next step).
5. **Audio** — pick the backend (gTTS or ElevenLabs), optionally a voice and
   **Force**, then synthesize. When it finishes, the batch goes green and hands
   you off to **View in Sentences**, **Package…**, or **Export to Anki…**.

`⌘/Ctrl + Enter` applies the reply in either packet stage. Anything you can do
here you could also do from the terminal — the Studio and the CLI call the same
validated use cases, so you can mix and match freely.

> One caveat: while a long `lingo audio` run is synthesizing, the viewer server
> is busy and won't serve other pages until it finishes. And a batch you generate
> in the Studio appears in the read-only **Sentences** tab only after you restart
> `lingo viewer` (that tab is a snapshot from launch).

For the full design and the API contract behind the Studio, see
[`docs/viewer/`](docs/viewer/).

## Editing the prompts themselves

The packets are rendered from editable prompt templates that live in your
language profile, not in the binary. To tune how Lingo asks the model, open a
prompt in your editor:

```bash
lingo lang edit import          # edit the import prompt
lingo lang edit build --deck    # override just this deck's build prompt
```

Profiles resolve in layers — the built-in default, then your global tweaks in
`~/.config/lingo/profiles/<lang>/`, then per-deck overrides — last one wins.
`lingo lang which` shows exactly which layer each prompt is coming from. Tune
the Hindi prompt once globally and every Hindi deck benefits; override one deck
without touching the rest.

## Audio backends

The default is gTTS — free, and good enough to start:

```toml
[audio]
backend = "gtts"
```

When you have credits, switch the primary to ElevenLabs and keep gTTS as the
automatic fallback so a quota error never blocks a batch:

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
backend  = "elevenlabs"
fallback = "gtts"

[audio.elevenlabs]
voice   = "9BWtsMINqrJLrRacOk9x"
model   = "eleven_multilingual_v2"
api_key = "env:ELEVENLABS_API_KEY"     # read from the env, never stored in config
```

`voice` must be an ElevenLabs voice ID, not only the display name.

```bash
lingo audio                       # uses the configured backend
lingo audio --backend gtts        # force gTTS for one run
lingo audio --force               # re-synthesize even where audio exists
```

## Script-first or roman-first display

On day one you read the romanisation; months later the script becomes your
primary cue. Lingo follows you along that curve with one per-deck setting — no
re-generation, it's purely a display choice over the same cards:

```toml
[display]
lead           = "romanisation"   # "romanisation" (beginner) | "target" (later)
show_secondary = true             # keep the other line visible underneath
```

`lead` decides which line is bold and on top (and which is the front of an Anki
card). Every card stores both `target` and `romanisation` as first-class fields,
so the data never assumes a level. You can also flip it per viewer session with
`lingo viewer --lead target`.

## Command reference

| Command | What it does |
| --- | --- |
| `lingo init --lang <language>` | Create or repair a deck workspace. |
| `lingo status` | Show where every batch is and what to run next. |
| `lingo import` | Raw text → reviewed source YAML (packet loop). |
| `lingo build` | Reviewed source → enriched card JSON (packet loop). |
| `lingo check` | Deterministic validation of the cards. |
| `lingo audio` | Synthesize missing sentence audio. |
| `lingo viewer` | Serve the local review/preview app (incl. the **Studio** generation UI). |
| `lingo package` | Bundle cards + audio into a portable folder. |
| `lingo export` | Build an Anki `.apkg`. |
| `lingo lang` | Inspect / edit language profiles and prompts. |
| `lingo doctor` | Check editor, clipboard, audio tools, and Node. |

Every command has full help:

```bash
lingo <command> --help
```

## Good to know about your data

Lingo is conservative about what becomes canonical:

- `input/sentences/` is written by `import`; `output/sentences/` by `build`.
- Invalid model replies are rejected — they never overwrite good data.
- Audio, packages, and exports are all derived from accepted card JSON.
- Files under `runs/` are working history (your packets and replies), not
  canonical data.

You *can* hand-edit `input/` or `output/` — building straight from
hand-written sentence YAML is fully supported — but run `lingo check`
afterwards before you generate audio or export.

## Troubleshooting

**`lingo doctor` says `uv` is missing.** Install `uv`, then re-run `doctor`.

**The viewer won't build.** Install its dependencies once:
`cd apps/viewer && npm install`, then retry `lingo viewer`.

**The model reply won't apply.** Open the reply file and check: valid YAML for
`import` / valid JSON for `build`, the exact `format` value the packet asked for,
every build card using an exact `source_item` ID from the packet, and no stray
markdown fences or commentary around the reply. Then save again (or re-run the
same `--apply`).

**Cards report missing audio.** Run `lingo audio`, then `lingo check`.

**I'm lost.** Run `lingo status` — it shows the true state and the next command.

**The viewer shows `0 words`.** Expected. New sentence-only decks don't create
word batches; the old Words page just stays available for legacy word data.

## For contributors

```bash
make build
make run ARGS='status'
make install            # add OFFLINE=1 to skip network if deps are cached
cargo test --workspace --all-targets
cd apps/viewer && npm run check
```

More detail:

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — the Rust crate boundaries.
- `docs/architecture/files/` — notes on individual source files.
- `apps/viewer/README.md` — the Astro viewer.

## License

Licensed under either Apache-2.0 or MIT, at your option.
</content>
