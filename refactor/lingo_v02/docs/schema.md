# Lingo Schema

The internal `library.db` schema — the source of truth. This is the authoring
model; it is intentionally decoupled from the **study export schema** the iOS app
codes against (that stable, versioned shape lives in
[`package-and-agents.md`](./package-and-agents.md) under `study`).

Vocabulary is `library → deck → sentence`. New docs and new code should use
`deck`; prototype `batch` naming is migration debris, not part of the v0.2 model.

```sql
decks (
  id          text primary key,  -- internal
  slug        text unique,       -- user-facing: "ch01"
  title       text,              -- "Complete Hindi"
  subtitle    text,              -- "Chapter 01"
  source_path text,
  position    integer,
  created_at  text
)

sentences (
  id        text primary key,    -- PUBLIC, PERMANENT (export progress key)
  deck_id   text references decks(id),
  status    text,                -- draft | enriched
  active    integer,             -- 0 | 1; approved for study, implies enriched
  qa_checked_at text,            -- nullable; set by `qa` apply, cleared by enrich --force
  origin    text,                -- generated | imported | manual
  source_library_id text,        -- nullable; for imported rows
  source_package_id text,        -- nullable; for imported rows
  source_sentence_id text,       -- nullable; for imported rows
  target text, romanisation text, english text, literal text, register text,
  authority text,                -- json field -> human|ai (the sacred map)
  breakdown text,                -- json token list
  tags text,
  audio_path text,
  order_in_deck integer
)

runs (
  id         text primary key,   -- "ch01-extract-7f3a"
  deck_id    text references decks(id),
  stage      text,               -- extract | enrich | qa
  status     text,               -- pending | applied | reset | failed
  task_path  text, reply_path text, created_at text
)
```

Notes:

- **`sentences.id` is public and permanent.** Both export targets (the iOS app and
  Anki) key study progress to it, so ids must never recycle. Audio files are also
  id-named (`audio/<sentence-id>.mp3`) for the same reason.
- **Lifecycle is only `draft | enriched`.** The user-visible `enriching` state is
  derived from pending `runs` + `run_sentences` claims. `active` is approval, not
  a lifecycle state.
- **`active` means approved for study.** It is valid only when
  `status = enriched`; study/Anki publish use active rows by default.
- **`authority`** is the sacred map of which fields the learner authored
  (`human`) vs the model (`ai`). `enrich` and `qa` never overwrite `human` fields.
- **`qa_checked_at`** is the QA state model: `status = enriched` with
  `qa_checked_at IS NULL` is what `status` calls "needs QA." It is stamped by
  applying a `qa` run and cleared by `enrich --force`.
- **`origin`** records whether a row was generated locally, imported from a
  package, or created manually. Imported rows keep source ids as provenance, even
  though they receive fresh local sentence ids.
- **`order_in_deck`** replaces the old section/order machinery; a "section"
  becomes the deck `subtitle`.
- **Run truth precedence:** the `runs` row is authoritative for status; the
  on-disk `runs/<id>/run.json` is its portable mirror and is re-derived from the
  DB if they disagree (see [`package-and-agents.md`](./package-and-agents.md)).
