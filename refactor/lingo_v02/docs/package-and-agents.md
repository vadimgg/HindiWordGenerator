# Package & Folder Structure (and how coding agents use it)

This page explains every directory Lingo touches, what is *truth* vs *derived* vs
*in-flight*, the on-disk shapes of the exports, and how a coding agent reads and
writes these files. For commands see [`CLI.md`](./CLI.md); for the end-to-end
process see [`workflows.md`](./workflows.md).

---

## The library (workspace) layout

A library is a directory with one SQLite database and a few well-defined folders:

```
my-hindi-library/
  library.db          # SOURCE OF TRUTH — sentences, decks, words, runs
  config.toml         # language, default titles, audio backend, publish dests
  AGENTS.md           # the file-handoff contract for coding agents

  raw/                # USER INPUT — raw material you feed to `extract`
    chapter01.md
  runs/               # IN-FLIGHT — one folder per prompt/reply handoff
    ch01-extract-7f3a/
      task.md         #   the prompt packet (paste this / agent reads this)
      reply.yaml      #   the model's answer goes here
      run.json        #   { stage, deck, status, reply, created }
  audio/              # DERIVED — generated speech, flat by sentence id
    sen-ch01-01.mp3
  out/                # DERIVED — publish artifacts
    ch01/             #   package (json) export of one deck
    study/            #   study (sqlite) export of the whole library
    ch01.apkg         #   anki export of one deck
  prompts/            # OPTIONAL — per-library prompt template overrides
```

### Four categories of files

| Category | Dirs | Rule |
|---|---|---|
| **Truth** | `library.db` | Canonical. Everything else can be rebuilt from it. |
| **Input** | `raw/` | Yours. Lingo only reads it. |
| **In-flight** | `runs/` | Resumable workflow state. A *pending* run holds work not yet committed — don't delete it until applied. |
| **Derived** | `audio/`, `out/` | Regenerable. Safe to delete after runs are applied. |

This is why "back up `library.db`" is enough, and why `lingo runs clean` (without
`--abandoned`) only removes runs whose replies are already committed.

---

## The handoff: `runs/<id>/`

Every model-facing stage (`extract`, `enrich`, `qa`) produces a **run**: a folder
plus a row in the DB. The folder is the portable face of the handoff.

```
runs/ch01-extract-7f3a/
  task.md        # human- and agent-readable prompt with a strict output contract
  reply.yaml     # extract → yaml; enrich/qa → reply.json
  run.json       # manifest that routes the reply without needing the DB
```

`run.json`:

```json
{
  "run": "ch01-extract-7f3a",
  "stage": "extract",
  "deck": "ch01",
  "status": "pending",
  "reply": "reply.yaml",
  "created": "2026-06-29T10:32:00Z"
}
```

**Truth precedence:** the DB `runs` row is authoritative for `status`; `run.json`
is its portable mirror so the handoff works without a live DB connection. If they
ever disagree, the DB wins and `run.json` is re-derived.

`lingo apply <run-dir>` reads `run.json`, learns the stage and deck, validates the
reply against that stage's contract, commits, and marks the run `applied`. That is
why you never type a run id or a stage — the run already carries them.

---

## How a coding agent uses these files

The agent never calls a model API on Lingo's behalf. It *is* the model in the
middle of the loop: it reads a task and writes a reply.

```
1. lingo status --json        → { "next": "...", "pending_runs": ["ch01-enrich-9b2c"] }
2. read runs/ch01-enrich-9b2c/task.md
3. produce the answer following the contract inside task.md
4. write runs/ch01-enrich-9b2c/reply.json
5. lingo apply runs/ch01-enrich-9b2c/
6. repeat from 1 until status.next is "done"
```

Guardrails that make this robust:

- **task.md is a contract, not a hint.** It states exactly which fence
  (` ```yaml ` / ` ```json `), which keys, and which ids to return. `apply`
  validates strictly and rejects anything off-contract with a specific error.
- **Re-apply, don't restart.** A failed `apply` leaves the run `pending`; the
  agent fixes `reply.*` in place and applies again. It must not open a new run.
- **`authority: human` is read-only.** Any field the learner supplied is locked;
  `apply` reports (and refuses) attempts to overwrite it.
- **QA is a fresh, small pass.** Run `lingo qa <deck>` in a clean context so a
  filled-up agent doesn't review its own mistakes blindly.

See `AGENTS.md` (written into every library by `lingo init`) for the canonical
wording the agent should follow.

---

## Export shapes

`lingo publish` emits four formats: the three below for two audiences
(interchange and study targets), plus `db`, a raw filtered copy of `library.db`
for power users. Full behavior is on [`cli/publish.md`](./cli/publish.md); the
study-facing shapes are below.

### `package` — JSON interchange (`lingo.package/v2`)

Lossless, diffable, round-trippable backup and agent-exchange format. One
self-describing JSON file per sentence, a manifest with sha256 integrity, and an
audio folder. **Never skips a sentence** — missing audio is exported as
`"audio": null`. Package export preserves approval, QA state, field authority,
tokens/breakdown, audio metadata, origin, and import provenance so backup→restore
round-trips exactly.

```
out/ch01/
  manifest.json            # format, language, decks[], counts, sha256 integrity
  sentences/
    sen-ch01-01.json       # full record: target, romanisation, english, literal,
    sen-ch01-02.json       #   register, approval, QA state, authority,
                           #   breakdown, tags, audio, origin/provenance
  audio/
    sen-ch01-01.mp3
  README.txt
```

A package may hold **one deck or many**; the manifest records each deck's slug,
title, and subtitle. `lingo import` reads this straight back, preserves deck slugs
(deduping with `-2` on collision), and dedupes sentences by normalized target.

### `study` — app-shaped SQLite (the iOS app)

A **stable, versioned** schema the app codes against — decoupled from the internal
`library.db` so authoring changes never break the app. Emitted as a **loose
folder**: a `study.sqlite` next to an `audio/` sidecar.

```
out/study/
  study.sqlite
  audio/
    ch01/sen-ch01-01.mp3
```

> Internal audio is stored flat (`audio/<sentence-id>.mp3`); exports are free to
> re-organize it (here, per-deck folders) for the consuming app. The export path
> is export-local, not the authoring path.

```sql
study_meta(study_schema_version, library_title, language, generated_at)
decks(slug primary key, title, subtitle, position)
sentences(id primary key, deck_slug, position,
          target, romanisation, english, literal, register,
          audio)                          -- relative path: audio/<deck>/<id>.mp3
words(key primary key, form, roman)
word_sentences(word_key, sentence_id, position, surface, roman, gloss)
sentence_tags(sentence_id, tag)
```

- **Whole library by default**; `lingo publish <deck> --format study` exports one
  deck.
- **Full-replace sync:** the export is a clean snapshot (no `updated_at`,
  no changelog). The app drops the old db and loads the new one.
- **Stable sentence ids are the contract:** the app keys study progress (SRS
  state) to `sentences.id`, so a fresh export preserves progress as long as ids
  never recycle. This is also why audio files are id-named.
- Carries the **word lexicon** so the app can do "tap a word → every sentence that
  uses it" — something Anki can't.
- Skips (and reports) sentences missing audio, since these feed a study session.

### `anki` — production cards (.apkg via the Anki API)

One note type, one card direction (production: English → target):

```
Note type: "Lingo Production"
Fields: SentenceId, English, Target, Romanisation, Audio, Breakdown(HTML), DeckPath

Card (Production):
  Front: {{English}}
  Back:  {{Target}}        (large)
         {{Audio}}         [sound:…]
         {{Romanisation}}
         {{Breakdown}}     word-by-word HTML table
```

- Anki deck name maps `"<library title>::<deck subtitle or title>"`,
  e.g. `Complete Hindi::Chapter 01`.
- **Note GUID = sentence id**, so re-exporting *updates* existing cards and keeps
  their scheduling instead of duplicating. Export is one-way; Lingo never reads
  Anki state back.
- Skips (and reports) sentences missing audio.

---

## Quick reference: what's safe to delete

| You delete… | Effect |
|---|---|
| `out/` | Nothing lost — re-run `lingo publish`. |
| `audio/` | Lose generated mp3s — re-run `lingo audio`. |
| `runs/<applied run>/` | Nothing lost — it's committed. (`lingo runs clean`.) |
| `runs/<pending run>/` | **Lose in-flight work** (the unapplied reply). |
| `raw/` | Lose your source material (Lingo can't regenerate it). |
| `library.db` | Lose everything. Back this up. |
