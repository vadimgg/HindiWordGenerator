# Appendix — MySQL Schema Translation

This appendix exists because the original architecture request asked for a MySQL schema.

It is **not** part of the CLI implementation plan. Do not add a MySQL crate, MySQL connection config, or a MySQL repository adapter for the personal local CLI.

Use this only if Lingo later becomes a hosted or multi-user system.

## Differences from SQLite

- Use `utf8mb4` for target-language text.
- Use `DATETIME(6)` instead of ISO text timestamps if the hosted service owns conversion.
- Use `ENUM` cautiously; migrations become heavier than SQLite text checks.
- Use `SELECT ... FOR UPDATE` in transactions where SQLite uses `BEGIN IMMEDIATE`.
- WAL guidance does not apply.
- A hosted system would need user/library ownership columns not present in the personal CLI schema.

## DDL

See [`mysql_schema.sql`](./mysql_schema.sql). Embedded below for review.

```sql
CREATE TABLE meta (
  `key` VARCHAR(128) PRIMARY KEY,
  `value` TEXT NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE decks (
  id VARCHAR(128) PRIMARY KEY,
  slug VARCHAR(128) NOT NULL UNIQUE,
  title TEXT,
  subtitle TEXT,
  source_path TEXT,
  position INT NOT NULL,
  created_at DATETIME(6) NOT NULL,
  updated_at DATETIME(6) NOT NULL,
  UNIQUE KEY decks_unique_position(position),
  CHECK (position > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE sentences (
  id VARCHAR(128) PRIMARY KEY,
  deck_id VARCHAR(128) NOT NULL,
  position INT NOT NULL,
  status ENUM('draft','enriched') NOT NULL DEFAULT 'draft',
  active BOOLEAN NOT NULL DEFAULT FALSE,
  qa_checked_at DATETIME(6) NULL,
  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register ENUM('informal','standard','formal') NULL,
  target_identity_key VARCHAR(512) NOT NULL,
  created_at DATETIME(6) NOT NULL,
  updated_at DATETIME(6) NOT NULL,
  UNIQUE KEY sentences_deck_position(deck_id, position),
  KEY sentences_status_updated(status, updated_at),
  KEY sentences_active_deck(active, deck_id, position),
  KEY sentences_target_identity(deck_id, target_identity_key),
  CONSTRAINT fk_sentences_deck FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE CASCADE,
  CHECK (position > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE sentence_field_authority (
  sentence_id VARCHAR(128) NOT NULL,
  field ENUM('target','romanisation','english','literal','register','breakdown') NOT NULL,
  authority ENUM('human','ai') NOT NULL,
  PRIMARY KEY(sentence_id, field),
  KEY sfa_authority(authority, field),
  CONSTRAINT fk_sfa_sentence FOREIGN KEY(sentence_id) REFERENCES sentences(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE sentence_tags (
  sentence_id VARCHAR(128) NOT NULL,
  tag VARCHAR(128) NOT NULL,
  PRIMARY KEY(sentence_id, tag),
  KEY sentence_tags_by_tag(tag, sentence_id),
  CONSTRAINT fk_tags_sentence FOREIGN KEY(sentence_id) REFERENCES sentences(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE sentence_tokens (
  sentence_id VARCHAR(128) NOT NULL,
  position INT NOT NULL,
  surface TEXT NOT NULL,
  roman TEXT,
  gloss TEXT NOT NULL,
  kind VARCHAR(128),
  word_key VARCHAR(512) NOT NULL,
  PRIMARY KEY(sentence_id, position),
  KEY sentence_tokens_by_word_key(word_key),
  CONSTRAINT fk_tokens_sentence FOREIGN KEY(sentence_id) REFERENCES sentences(id) ON DELETE CASCADE,
  CHECK (position > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE runs (
  id VARCHAR(128) PRIMARY KEY,
  stage ENUM('extract','enrich','qa') NOT NULL,
  status ENUM('pending','applied','reset','abandoned') NOT NULL DEFAULT 'pending',
  deck_id VARCHAR(128),
  task_path TEXT NOT NULL,
  reply_path TEXT NOT NULL,
  reply_sha256 CHAR(64),
  last_validation_error TEXT,
  created_at DATETIME(6) NOT NULL,
  applied_at DATETIME(6),
  reset_at DATETIME(6),
  abandoned_at DATETIME(6),
  KEY runs_status_created(status, created_at),
  KEY runs_deck_stage(deck_id, stage, created_at),
  CONSTRAINT fk_runs_deck FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE SET NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE run_sentences (
  run_id VARCHAR(128) NOT NULL,
  sentence_id VARCHAR(128) NOT NULL,
  position INT NOT NULL,
  PRIMARY KEY(run_id, sentence_id),
  UNIQUE KEY run_sentences_run_position(run_id, position),
  KEY run_sentences_by_sentence(sentence_id),
  CONSTRAINT fk_rs_run FOREIGN KEY(run_id) REFERENCES runs(id) ON DELETE CASCADE,
  CONSTRAINT fk_rs_sentence FOREIGN KEY(sentence_id) REFERENCES sentences(id) ON DELETE CASCADE,
  CHECK (position > 0)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE sentence_audio (
  sentence_id VARCHAR(128) PRIMARY KEY,
  file_sha256 CHAR(64) NOT NULL,
  input_fingerprint CHAR(64) NOT NULL,
  backend ENUM('gtts') NOT NULL,
  profile_id VARCHAR(128) NOT NULL,
  language_code VARCHAR(32) NOT NULL,
  voice VARCHAR(255),
  model VARCHAR(255),
  generated_at DATETIME(6) NOT NULL,
  KEY sentence_audio_backend(backend, generated_at),
  KEY sentence_audio_fingerprint(input_fingerprint),
  CONSTRAINT fk_audio_sentence FOREIGN KEY(sentence_id) REFERENCES sentences(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

```

## Hosted-system additions not in this schema

If this ever becomes multi-user, add these deliberately rather than sneaking them into the CLI schema:

```text
users
libraries(user_id, slug, title, ...)
workspace memberships / permissions
audit log
provider credential vault
encrypted secrets
background job table
```

Those are not needed for the current tool.
