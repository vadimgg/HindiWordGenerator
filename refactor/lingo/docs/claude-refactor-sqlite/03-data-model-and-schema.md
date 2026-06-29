# 03 · Data model & SQLite schema

This document is authoritative for the runtime data model and the
`lingo.library/v1` SQLite schema.

## 1. Concept

The **sentence** is the runtime atom. The canonical store is one SQLite database
per deck:

```
my-deck/
  config.toml
  library.db                  ← canonical: sentences, words, occurrences, meanings, audio metadata
  audio/<sentence-id>.mp3      ← referenced by library.db (path + hash)
  raw/, runs/                  ← ephemeral generation scratch
```

Why SQLite: organize and audio are query problems (`ORDER BY`, `WHERE audio IS
NULL`, one-transaction reorder); the lexicon needs relations; Grasp can read the
db natively; `rusqlite` is already in the stack (the Anki `.apkg` exporter writes
SQLite). JSON remains the portable import/export form (see
[08](./08-import-export.md)), not a second runtime truth.

## 2. Identity & provenance

- **Identity is library-scoped and opaque** (`SentenceId`, e.g. ULID) — it does
  **not** encode the batch. Reorganizing across sections and importing from
  multiple packages never collide.
- **Provenance** records where a sentence came from, as a JSON column, with no
  bearing on identity:

```jsonc
// generated
{ "kind": "generated", "run": "extract-1719…" }
// imported
{ "kind": "imported", "package": "sentence_package_01_lingo",
  "source_batch": "complete_hindi_chapter_02_sentences_batch_01", "source_item": "0001" }
```

- **Collection holds the title; sentences hold a section + order.** A deck is
  usually one collection ("Complete Hindi"); sections are the chapters
  ("Chapter 02"). Word identity is scoped to the collection (D3).

## 3. Field authority (the "uncle-ji" guarantee, R4)

Each editable field records who authored it. Enrichment **must** preserve `human`
fields and only fill empty/`ai` ones.

```jsonc
// sentences.authority
{ "english": "human", "romanisation": "human", "literal": null, "breakdown": "ai" }
```

Rules:
- **extract** marks anything the learner supplied as `human`.
- **enrich** reads `authority`; for any `human` field it passes the value through
  byte-for-byte (honorifics/particles like *ji* intact) and never asks the model
  to change it; it generates only for empty fields.
- editing a field in CLI/UI flips it to `human`.

This is enforced in the domain (`FieldAuthoritySet::reject_human_field_changes`)
and again at apply time (see [04](./04-workflows-and-files.md),
[11](./11-public-api-sketches.md)).

## 4. Processing state & batching (R13)

Enrichment is an AI step with a limited context window, so it runs in **bounded
batches** and a sentence is **never sent twice**. The `status` column is the
tracker:

```
draft ──(prompt emitted, rows claimed)──▶ enriching ──(reply applied)──▶ enriched
  ▲                                          │
  └────────────────(reset)───────────────────┘
```

- Selection defaults to `status = 'draft'`; `--limit N` bounds one prompt.
- Emitting a prompt **claims** the next `N` draft rows in one transaction:
  `status = 'enriching'`, `enrich_run = <run id>`. The next call picks the
  following N, so prompts can be fanned out in parallel with no overlap.
- Applying a reply touches only that run's rows and flips them to `enriched`.
- `reset` returns abandoned `enriching` rows to `draft`.
- Re-enrichment of `enriched` rows is explicit (`--force`), still honoring
  field authority.

`status ∈ {draft, enriching, enriched}` (D4). **"Ready to publish" is derived**
(`enriched` AND audio present), never stored. Audio has its own lifecycle (§6,
[04](./04-workflows-and-files.md)).

## 5. Words lexicon (R5, R12)

Word identity is the **normalized surface form**, scoped per collection.
Inflected/gendered variants are simply separate rows — no lemmatizer. The lexicon
is **derived** from committed sentence breakdowns at apply time; if it drifts it
can be rebuilt from the canonical sentences.

## 6. Schema (`lingo.library/v1`)

`STRICT` tables (requires SQLite ≥ 3.37; the composition root asserts this).
Every connection sets `PRAGMA foreign_keys = ON` and `journal_mode = WAL`, and
applies migrations before use. Schema version is tracked by `PRAGMA user_version`
**and** the self-describing `library_metadata` table.

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
-- PRAGMA user_version = 1;  -- set/checked by migrations

CREATE TABLE library_metadata (
  key   TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;
-- seeded: ('format','lingo.library/v1'), ('schema_version','1'),
--         ('created_by','lingo'), ('created_at', <utc>)

CREATE TABLE collections (
  id         TEXT PRIMARY KEY,            -- ulid
  title      TEXT NOT NULL,               -- "Complete Hindi"
  language   TEXT NOT NULL,               -- "hi"
  created_at TEXT NOT NULL,               -- ISO-8601 UTC
  updated_at TEXT NOT NULL
) STRICT;

CREATE TABLE sentences (
  id            TEXT PRIMARY KEY,         -- ulid, library-scoped (NOT batch:item)
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  section       TEXT,                     -- "Chapter 02" (the old "subtitle")
  ord           INTEGER NOT NULL,         -- arrangement within the collection

  target        TEXT NOT NULL,            -- foreign sentence (Devanagari)
  romanisation  TEXT,
  english       TEXT,
  literal       TEXT,
  register      TEXT CHECK (register IS NULL OR register IN ('informal','standard','formal')),

  authority     TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(authority)),   -- §3
  breakdown     TEXT CHECK (breakdown IS NULL OR json_valid(breakdown)),    -- denormalized for render/export
  tokens        TEXT CHECK (tokens IS NULL OR json_valid(tokens)),          -- ordered tokens for highlighting
  tags          TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),

  status        TEXT NOT NULL DEFAULT 'draft'
                  CHECK (status IN ('draft','enriching','enriched')),       -- §4 / D4
  enrich_run    TEXT,                     -- owning run id while 'enriching'

  provenance    TEXT NOT NULL CHECK (json_valid(provenance)),               -- §2
  created_at    TEXT NOT NULL,
  updated_at    TEXT NOT NULL
) STRICT;

CREATE UNIQUE INDEX sentences_unique_order ON sentences(collection_id, ord);
CREATE INDEX sentences_by_section   ON sentences(collection_id, section, ord);
CREATE INDEX sentences_by_status    ON sentences(status, updated_at);
CREATE INDEX sentences_by_enrich_run ON sentences(enrich_run) WHERE enrich_run IS NOT NULL;

-- One current audio attachment per sentence (D2). A row exists only when audio
-- is present, so "missing audio" = no row. Audio is keyed by sentence id.
CREATE TABLE sentence_audio (
  sentence_id TEXT PRIMARY KEY REFERENCES sentences(id) ON DELETE CASCADE,
  path        TEXT NOT NULL,              -- "audio/<sentence-id>.mp3"
  hash        TEXT NOT NULL,              -- sha256 of the mp3
  backend     TEXT NOT NULL CHECK (backend IN ('gtts','elevenlabs')),
  format      TEXT NOT NULL DEFAULT 'mp3' CHECK (format = 'mp3'),
  voice       TEXT,
  model       TEXT,
  created_at  TEXT NOT NULL
) STRICT;

-- The lexicon. Identity = normalized surface form, scoped per collection (D3).
CREATE TABLE words (
  id            TEXT PRIMARY KEY,         -- ulid
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  key           TEXT NOT NULL,            -- normalized surface form (the identity)
  form          TEXT NOT NULL,            -- display form (Devanagari)
  roman         TEXT,
  kind          TEXT,                     -- best-effort POS, optional
  UNIQUE(collection_id, key)
) STRICT;

CREATE TABLE word_meanings (
  id                     TEXT PRIMARY KEY,
  word_id                TEXT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
  meaning                TEXT NOT NULL,
  meaning_key            TEXT NOT NULL,   -- normalized (case/space) for dedup
  first_seen_sentence_id TEXT REFERENCES sentences(id) ON DELETE SET NULL,
  UNIQUE(word_id, meaning_key)
) STRICT;

CREATE TABLE sentence_words (
  sentence_id  TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  word_id      TEXT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
  position     INTEGER NOT NULL,          -- order within the sentence
  surface_form TEXT NOT NULL,             -- exact form as it appears
  gloss        TEXT,                      -- meaning in this context
  PRIMARY KEY (sentence_id, position)
) STRICT;
CREATE INDEX sentence_words_by_word ON sentence_words(word_id);
```

### Notes on hardening

- **Closed sets** (`status`, `register`, `backend`) are guarded by both Rust
  enums (which own `wire_name()`) and SQL `CHECK`s.
- **JSON columns** (`authority`, `breakdown`, `tokens`, `tags`, `provenance`) get
  `json_valid()` checks; the adapter decodes each into a domain type immediately —
  arbitrary `serde_json::Value` never flows inward.
- **Order uniqueness** is enforced (`sentences_unique_order`). Because SQLite
  unique indexes are not deferrable, reorder runs as a transaction that parks
  affected rows at sentinel/negative `ord` values before writing finals; use
  sparse integers and compact intentionally if drag-reorder becomes hot.
- **Audio as a table** (not inline columns) so the audio service can iterate
  voice/model and so "missing audio" is simply the absence of a row.
- **Timestamps** are ISO-8601 UTC strings; the domain exposes a `UtcTimestamp`
  type, never raw `String`.
- **Normalize later only on real query pressure**: e.g. `sentence_tags`,
  `sentence_field_authority`, or `sentence_breakdown_items` tables if tag
  filtering / per-field audit / per-token SQL becomes necessary.

## 7. Example queries

```sql
-- a section, in order, for rendering
SELECT * FROM sentences WHERE collection_id = ? AND section = ? ORDER BY ord;

-- sentences missing audio (the Audio page's worklist)
SELECT s.* FROM sentences s
LEFT JOIN sentence_audio a ON a.sentence_id = s.id
WHERE s.collection_id = ? AND a.sentence_id IS NULL ORDER BY s.ord;

-- how many sentences contain a word
SELECT COUNT(DISTINCT sentence_id) FROM sentence_words WHERE word_id = ?;

-- meanings learned for a word
SELECT meaning FROM word_meanings WHERE word_id = ? ORDER BY meaning;

-- words seen only once (study candidates)
SELECT w.form, COUNT(*) c FROM words w
JOIN sentence_words sw ON sw.word_id = w.id
GROUP BY w.id HAVING c = 1;
```

## 8. Example: one sentence

| column | value |
|---|---|
| id | `01J8ZQ3K9C…` |
| collection_id | `01J8Z…` ("Complete Hindi") |
| section | `Chapter 02` |
| ord | `1` |
| target | `अध्यापक जी, यहाँ कितने विद्यार्थी हैं?` |
| romanisation | `adhyāpak jī, yahā̃ kitne vidyārthī haĩ?` |
| english | `Teacher ji, how many students are here?` |
| authority | `{"english":"human"}` |
| status | `enriched` |
| provenance | `{"kind":"imported", …}` |

Plus one `sentence_audio` row (`audio/01J8ZQ3K9C….mp3`) and 6 `sentence_words`
rows feeding `words` / `word_meanings` (e.g. `जी` → "honorific (ji)").

## 9. `lingo dump --json` (review / VCS snapshot)

Because the db is binary, a derived JSON dump keeps history reviewable
(`my-deck/library.json`, generated; the db stays canonical). Same data, see
[08](./08-import-export.md) for the per-sentence export shape.

## 10. Schema tests required

- migration from empty DB creates the schema + `library_metadata`;
- opening a DB with unsupported `library_metadata.format` fails loudly;
- `PRAGMA foreign_keys` is ON per connection;
- invalid `status` / `register` / `backend` / invalid JSON are rejected;
- claiming enrichment rows is atomic and prevents double-claiming;
- applying an enrichment reply updates sentence + status + words + meanings +
  occurrences in one transaction;
- reorder preserves the unique-order invariant;
- `sentence_audio` cannot reference a missing sentence;
- an exported db copy opens read-only and passes an integrity check.
