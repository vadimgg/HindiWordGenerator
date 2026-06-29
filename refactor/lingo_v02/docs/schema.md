# Lingo Schema Overview

This is a prose overview of the target internal `library.db` model. The exact
SQL source of truth is [`arch/schema_v02.sql`](./arch/schema_v02.sql); repository
tests should initialize from that file, not from this page.

This is the v0.2 architecture schema file, but a fresh database starts at
internal `meta.schema_version = 1`.

The authoring model is intentionally decoupled from the **study export schema**
the iOS app codes against (that stable, versioned shape lives in
[`package-and-agents.md`](./package-and-agents.md) under `study`).

Vocabulary is `library → deck → sentence`. New docs and new code should use
`deck`; prototype `batch` naming is migration debris, not part of the v0.2 model.

Core tables in the exact SQL:

- `meta` stores format, schema version, library id, and language profile.
- `decks` stores user-facing study units: slug, title, subtitle, position.
- `sentences` stores stable sentence ids, lifecycle, approval, QA, origin,
  source identity, and study-facing text fields.
- `sentence_field_authority` stores per-field `human` / `ai` ownership.
- `sentence_tokens` stores the ordered word-by-word breakdown.
- `runs` and `run_sentences` store prompt/reply handoff state and claimed rows.
- `sentence_audio` stores audio metadata and input fingerprints; file paths are
  derived as `audio/<sentence-id>.mp3`.

Notes:

- **`sentences.id` is public and permanent.** Both export targets (the iOS app and
  Anki) key study progress to it, so ids must never recycle. Audio files are also
  id-named (`audio/<sentence-id>.mp3`) for the same reason.
- **Lifecycle is only `draft | enriched`.** The user-visible `enriching` state is
  derived from pending `runs` + `run_sentences` claims. `active` is approval, not
  a lifecycle state.
- **`active` means approved for study.** It is valid only when
  `status = enriched`; study/Anki publish use active rows by default.
- **Field authority** records which fields the learner authored (`human`) vs the
  model (`ai`). `enrich` and `qa` never overwrite `human` fields.
- **`qa_checked_at`** is the QA state model: `status = enriched` with
  `qa_checked_at IS NULL` is what `status` calls "needs QA." It is stamped by
  applying a `qa` run and cleared by `enrich --force`.
- **`origin`** records whether a row was generated locally, imported from a
  package, or created manually. Imported rows keep source ids as provenance, even
  though they receive fresh local sentence ids.
- **`sentences.position`** replaces the old section/order machinery; a "section"
  becomes the deck `subtitle`.
- **Run truth precedence:** the `runs` row is authoritative for status; the
  on-disk `runs/<id>/run.json` is its portable mirror and is re-derived from the
  DB if they disagree (see [`package-and-agents.md`](./package-and-agents.md)).
