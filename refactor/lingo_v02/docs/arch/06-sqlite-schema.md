# 06 — Canonical SQLite Schema

This is the authoring schema for `library.db`. It is not the study-app export schema.

## Connection setup

Every SQLite connection should run:

```sql
PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

Every write use case that mutates library state should use `BEGIN IMMEDIATE` so competing CLI/UI writers fail or wait predictably.

## Design notes

- `meta.schema_version = 1` for the clean rebuild.
- `decks` is the product vocabulary. Do not keep `batches` internally.
- `sentences.status` stores only `draft | enriched`.
- Visible `enriching` is derived from pending enrich runs in `run_sentences`.
- `active` is curation, not lifecycle.
- `qa_checked_at` is QA state, not lifecycle.
- `sentence_field_authority` is normalized.
- `sentence_tokens` is the canonical breakdown.
- `run_sentences` is canonical for run participation and pending claims.
- `sentence_audio` stores provenance/fingerprints, not a path. Internal path is deterministic: `audio/<sentence-id>.mp3`.
- Word lexicon is a projection over `sentence_tokens`.

## DDL

See [`schema_v02.sql`](./schema_v02.sql). The same DDL is embedded below for review.

```sql
PRAGMA foreign_keys = ON;

CREATE TABLE meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

INSERT INTO meta(key, value) VALUES
  ('format', 'lingo.library/v2'),
  ('schema_version', '1'),
  ('created_with_lingo_version', '0.2.0'),
  ('language_profile', 'hindi');

CREATE TABLE decks (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  title TEXT,
  subtitle TEXT,
  source_path TEXT,
  position INTEGER NOT NULL CHECK (position > 0),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(position)
) STRICT;

CREATE TABLE sentences (
  id TEXT PRIMARY KEY,
  deck_id TEXT NOT NULL REFERENCES decks(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position > 0),

  status TEXT NOT NULL DEFAULT 'draft'
    CHECK (status IN ('draft', 'enriched')),
  active INTEGER NOT NULL DEFAULT 0
    CHECK (active IN (0, 1)),
  qa_checked_at TEXT,

  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register TEXT CHECK (register IS NULL OR register IN ('informal', 'standard', 'formal')),

  target_identity_key TEXT NOT NULL,

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,

  UNIQUE(deck_id, position)
) STRICT;

CREATE INDEX sentences_by_deck_order ON sentences(deck_id, position);
CREATE INDEX sentences_by_status ON sentences(status, updated_at);
CREATE INDEX sentences_by_active ON sentences(active, deck_id, position) WHERE active = 1;
CREATE INDEX sentences_needing_qa ON sentences(deck_id, position)
  WHERE status = 'enriched' AND qa_checked_at IS NULL;
CREATE INDEX sentences_by_target_identity ON sentences(deck_id, target_identity_key);

CREATE TABLE sentence_field_authority (
  sentence_id TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  field TEXT NOT NULL CHECK (field IN (
    'target', 'romanisation', 'english', 'literal', 'register', 'breakdown'
  )),
  authority TEXT NOT NULL CHECK (authority IN ('human', 'ai')),
  PRIMARY KEY(sentence_id, field)
) STRICT;

CREATE INDEX sentence_field_authority_by_authority
  ON sentence_field_authority(authority, field);

CREATE TABLE sentence_tags (
  sentence_id TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  tag TEXT NOT NULL,
  PRIMARY KEY(sentence_id, tag)
) STRICT;

CREATE INDEX sentence_tags_by_tag ON sentence_tags(tag, sentence_id);

CREATE TABLE sentence_tokens (
  sentence_id TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position > 0),
  surface TEXT NOT NULL,
  roman TEXT,
  gloss TEXT NOT NULL,
  kind TEXT,
  word_key TEXT NOT NULL,
  PRIMARY KEY(sentence_id, position)
) STRICT;

CREATE INDEX sentence_tokens_by_word_key ON sentence_tokens(word_key);
CREATE INDEX sentence_tokens_by_sentence_word ON sentence_tokens(sentence_id, word_key);

CREATE TABLE runs (
  id TEXT PRIMARY KEY,
  stage TEXT NOT NULL CHECK (stage IN ('extract', 'enrich', 'qa')),
  status TEXT NOT NULL DEFAULT 'pending'
    CHECK (status IN ('pending', 'applied', 'reset', 'abandoned')),
  deck_id TEXT REFERENCES decks(id) ON DELETE SET NULL,

  task_path TEXT NOT NULL,
  reply_path TEXT NOT NULL,

  reply_sha256 TEXT,
  last_validation_error TEXT,

  created_at TEXT NOT NULL,
  applied_at TEXT,
  reset_at TEXT,
  abandoned_at TEXT
) STRICT;

CREATE INDEX runs_by_status_created ON runs(status, created_at);
CREATE INDEX runs_by_deck_stage ON runs(deck_id, stage, created_at);
CREATE INDEX runs_pending_by_stage ON runs(stage, created_at) WHERE status = 'pending';

CREATE TABLE run_sentences (
  run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
  sentence_id TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  position INTEGER NOT NULL CHECK (position > 0),
  PRIMARY KEY(run_id, sentence_id),
  UNIQUE(run_id, position)
) STRICT;

CREATE INDEX run_sentences_by_sentence ON run_sentences(sentence_id);

CREATE TABLE sentence_audio (
  sentence_id TEXT PRIMARY KEY REFERENCES sentences(id) ON DELETE CASCADE,
  file_sha256 TEXT NOT NULL,
  input_fingerprint TEXT NOT NULL,
  backend TEXT NOT NULL CHECK (backend IN ('gtts')),
  profile_id TEXT NOT NULL,
  language_code TEXT NOT NULL,
  voice TEXT,
  model TEXT,
  generated_at TEXT NOT NULL
) STRICT;

CREATE INDEX sentence_audio_by_backend ON sentence_audio(backend, generated_at);
CREATE INDEX sentence_audio_by_fingerprint ON sentence_audio(input_fingerprint);

```

## Visible status query

```sql
SELECT
  s.id,
  s.status,
  pending_enrich.run_id AS pending_enrich_run_id,
  CASE
    WHEN pending_enrich.run_id IS NOT NULL THEN 'enriching'
    ELSE s.status
  END AS visible_status
FROM sentences s
LEFT JOIN (
  SELECT rs.sentence_id, rs.run_id
  FROM run_sentences rs
  JOIN runs r ON r.id = rs.run_id
  WHERE r.stage = 'enrich'
    AND r.status = 'pending'
) pending_enrich ON pending_enrich.sentence_id = s.id;
```

Do not persist `visible_status`.

## Pending-claim validation

SQLite cannot express the pending-claim invariant as a partial index because it requires joining `runs`. Validate inside the same `BEGIN IMMEDIATE` transaction that creates the claim.

```sql
SELECT rs.sentence_id
FROM run_sentences rs
JOIN runs r ON r.id = rs.run_id
WHERE r.stage = 'enrich'
  AND r.status = 'pending'
  AND rs.sentence_id IN (:selected_sentence_ids)
LIMIT 1;
```

If any row exists, reject with `sentence_already_claimed`.

## Word projection query

```sql
SELECT
  word_key,
  MIN(surface) AS form,
  MIN(roman) AS roman,
  COUNT(DISTINCT sentence_id) AS sentence_count,
  GROUP_CONCAT(DISTINCT gloss) AS meanings
FROM sentence_tokens
GROUP BY word_key
HAVING sentence_count >= ?
ORDER BY sentence_count DESC, word_key;
```

If this becomes slow, add a materialized `word_projection` table. That table must be explicitly named as a projection and rebuilt from `sentence_tokens`.

## Apply transaction sketch

```sql
BEGIN IMMEDIATE;

SELECT id, stage, status, reply_sha256
FROM runs
WHERE id = ?;

-- Re-check status and hash after acquiring write lock.
-- Mutate stage-specific tables.
-- Insert/delete sentence_tokens as needed.
-- Update sentence_field_authority as needed.
-- Insert run_sentences for extract-created sentences.
-- Stamp qa_checked_at for QA rows.
-- Update sentence_audio only from audio use case, not apply.

UPDATE runs
SET status = 'applied',
    reply_sha256 = ?,
    applied_at = ?,
    last_validation_error = NULL
WHERE id = ? AND status = 'pending';

COMMIT;
```

## Schema migration policy

Before real production data exists, reset cleanly.

After v0.2 has real data:

```text
1. backup library.db
2. run migration inside transaction where SQLite allows it
3. verify meta.schema_version
4. run doctor checks
5. keep migration fixtures
```

Do not keep old and new schemas live in parallel unless protecting real user data.
