# 00 · Requirements & workflows

## 1. What we have now

### 1.1 Crate / app layout (hexagonal Rust + Astro viewer)

```
crates/
  lingo-domain/         value objects + entities (Card, SourceBatch, ids, …)
  lingo-application/    use cases + port traits (import, build, check, audio, package, export)
  lingo-workspace-fs/   filesystem adapters: layout, codecs, profiles, store, config
  lingo-artifacts/      publishers: Anki .apkg exporter + portable package publisher
  lingo-audio/          audio backends: gTTS, ElevenLabs
  lingo-prompt/         Handlebars prompt engine + packet/reply parsing
  lingo-cli/            CLI + local viewer HTTP server + Studio API
apps/
  viewer/               Astro static UI (Studio, Sentences, Deliver, Words, QA tabs)
```

The hexagonal split is good and we keep it. The problem is **what flows through
it**, not the layering (see [01](./01-architecture.md)).

### 1.2 Workspace layout on disk (today)

```
my-deck/
  config.toml                      deck settings (see §1.4)
  profile.toml                     (optional) deck-local profile override
  prompts/                         (optional) deck-local prompt overrides
  raw/<batch>.md                   pasted raw source text
  input/sentences/<batch>.yaml     reviewed source sentences  (lingo.source/v1)
  output/sentences/<batch>.json    enriched cards             (lingo.cards/v1)
  audio/sentences/<batch>/<item>.mp3
  sentences/<batch>__<item>.json   prototype per-sentence layer (lingo.sentence/v1)
  runs/<stage>/<id>/               prompt + reply packets per run
  packages/  exports/              publish outputs
```

A single sentence currently exists as three on-disk representations
(`input/.../<batch>.yaml`, `output/.../<batch>.json`, `sentences/...`). This
refactor collapses them to one row in `library.db`.

### 1.3 The pipeline (today)

```
raw ── import ──▶ build ──▶ check ──▶ audio ──▶ package / export
(text) (LLM)     (LLM)    (rules)   (TTS)     (publish)
```

`import` and `build` are **manual LLM packet loops**: the CLI/Studio emits a
prompt packet, you paste it into ChatGPT/Claude, paste the reply back, and it is
validated before anything is written. Stage state is tracked per batch and drives
the Studio stepper.

### 1.4 Config (`config.toml`, format `lingo.deck/v1`)

```toml
[target]
profile = "hindi"

[learner]
goal = "practical fluency"
native_languages = ["English"]

[display]
lead = "romanisation"     # romanisation | target
show_secondary = true

[audio]
backend = "gtts"          # gtts | elevenlabs
[audio.gtts]
lang = "hi"
[audio.elevenlabs]
api_key = "env:ELEVENLABS_API_KEY"
model = "eleven_multilingual_v2"
voice = "9BWtsMINqrJLrRacOk9x"

[export]
deck = "Hindi::Sentences"

[package]
destination = "packages/sentences"
```

Secrets resolve from env (`env:VAR`) or `.lingo.secrets.toml` (gitignored).

### 1.5 Prompts (today)

- Built into each language profile, embedded in the binary:
  `crates/lingo-workspace-fs/assets/profiles/hindi/prompts/{import,build}.md.hbs`.
- Rendered by `lingo-prompt` (Handlebars) with context like `{{target.language}}`,
  `{{target.script}}`, `{{romanisation.convention}}`.
- Overridable per deck (`my-deck/prompts/`) or per user
  (`~/.config/lingo/profiles/<id>/prompts/`).

## 2. The workflows we must serve

### Workflow A — Generate from raw text (LLM)

Paste raw text and produce sentences via an LLM prompt, in two stages:

1. **Extract**: raw text → sentences. The raw can be:
   - a foreign-language paragraph from a book, **or**
   - sentences already studied *with the learner's own translation/notes* that
     just need clean formatting (this is English, this is roman, this is foreign).
   At this stage we have at least the **foreign sentence**; translation may or
   may not be present.
2. **Enrich**: sentences → translation (if missing) + romanisation + literal +
   word-by-word breakdown.

The LLM step can be driven by **web** (copy/paste), **file handoff** (the CLI
writes a prompt file, an agent writes the reply file, the CLI reads it), or a
**direct API** call (see [05](./05-cli.md)).

> **Why two stages and why the learner sometimes supplies translations:** AI
> agents quietly drop nuances that teaching books keep — e.g. the Hindi honorific
> *ji* ("uncle-ji"). When the learner provides their own translation/wording, the
> system must **preserve it** and not let enrichment overwrite it. This is the
> field-authority guarantee (see [03 §5](./03-data-model-and-schema.md)).

### Workflow B — Import sentences from another package

Finished sentences already exist in a package (e.g. a Grasp resource folder).
Pull them into the current library and treat them like any other sentences
(organize, re-audio, re-export). Import reads the JSON/db formats we export; a
foreign/old package is converted by a throwaway script first (see
[08](./08-import-export.md)).

### After ingestion (shared by A and B)

- **Organize**: reorder, assign sections (chapters), set the collection title,
  tag, delete — across all sentences, not per batch.
- **Audio**: generate / regenerate audio per sentence with a chosen voice, in a
  dedicated audio workspace (not a pipeline gate).
- **Words**: a lexicon view — each word, how many sentences it appears in, and
  the meanings learned so far.
- **Publish** to two independent targets: **Anki** (`.apkg` / AnkiConnect) and
  **Package** (for Grasp or any other consumer).

## 3. Requirements

| # | Requirement |
|---|-------------|
| R1 | Sentence is the atom; one canonical store (SQLite). |
| R2 | Multiple producers commit into the library (extract, enrich, import). |
| R3 | Multiple independent publishers (Anki, Package) read from the library. |
| R4 | Human-authored fields are preserved through enrichment (field authority). |
| R5 | A queryable words lexicon (counts, meanings) derived from sentences. |
| R6 | Audio is a re-runnable service over the library, with its own workspace. |
| R7 | CLI-first: every operation scriptable; UI is a helper that mirrors files. |
| R8 | UI shows the equivalent CLI command, organized consistently. |
| R9 | UI settings changes persist to `config.toml`. |
| R10 | Per-language prompt sets, customizable and discoverable. |
| R11 | Package + db schema are versioned contracts (Grasp consumes them). |
| R12 | Word identity is simple — normalized surface form; variants are separate entries (no lemma engine). |
| R13 | AI enrichment runs in bounded batches (learner-chosen size), and the system tracks what is already processed so a sentence is never sent in two prompts. |

## 4. Non-goals

- No lemmatization / morphological analysis engine (R12).
- No multi-user / server-hosted deployment; local-first.
- No live shared-DB editing between Lingo and Grasp — Grasp consumes an exported
  copy of the library.
- No automatic re-translation of human-provided content (R4).
- No new abstractions/crates for symmetry (see [01](./01-architecture.md),
  [09](./09-reuse-and-patterns.md)).
