-- v0.2 architecture schema file.
-- Fresh databases created from this clean schema start at meta.schema_version = 1.

PRAGMA foreign_keys = ON;

CREATE TABLE meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

INSERT INTO meta(key, value) VALUES
  ('format', 'lingo.library/v2'),
  ('schema_version', '1'),
  ('library_id', 'lib-' || lower(hex(randomblob(16)))),
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

  origin TEXT NOT NULL DEFAULT 'generated'
    CHECK (origin IN ('generated', 'imported', 'manual')),
  source_label TEXT,
  source_extract_run_id TEXT,
  source_library_id TEXT,
  source_package_id TEXT,
  source_sentence_id TEXT,

  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register TEXT CHECK (register IS NULL OR register IN ('informal', 'standard', 'formal')),

  target_identity_key TEXT NOT NULL,

  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,

  UNIQUE(deck_id, position),
  CHECK (active = 0 OR status = 'enriched'),
  CHECK (
    (origin = 'generated'
      AND source_library_id IS NULL
      AND source_package_id IS NULL
      AND source_sentence_id IS NULL)
    OR
    (origin = 'imported'
      AND source_library_id IS NOT NULL
      AND source_package_id IS NOT NULL
      AND source_sentence_id IS NOT NULL)
    OR
    (origin = 'manual'
      AND source_extract_run_id IS NULL
      AND source_library_id IS NULL
      AND source_package_id IS NULL
      AND source_sentence_id IS NULL)
  )
) STRICT;

CREATE INDEX sentences_by_deck_order ON sentences(deck_id, position);
CREATE INDEX sentences_by_status ON sentences(status, updated_at);
CREATE INDEX sentences_by_active ON sentences(active, deck_id, position) WHERE active = 1;
CREATE INDEX sentences_needing_qa ON sentences(deck_id, position)
  WHERE status = 'enriched' AND qa_checked_at IS NULL;
CREATE INDEX sentences_by_target_identity ON sentences(deck_id, target_identity_key);
CREATE INDEX sentences_by_origin ON sentences(origin, source_library_id, source_package_id);

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
