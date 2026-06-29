# 10 · Implementation plan & tests

Each phase ships independently and leaves the app working. Prefer a thin vertical
slice before broad build-out.

## Phase 0 — Guardrails

1. Update the architecture dependency test to include new modules.
2. Add a short architecture map naming the source-of-truth boundary (doc 01).
3. Decide whether prototype batch files need migration. If no real production data
   exists, document that old runtime file shapes are removed (doc 01 §6).

Evidence: `cargo metadata` architecture test passes; a review note answers the
deletion test for `LibraryStore` (and any new crate, which there should not be).

## Phase 0.5 — Walking skeleton (one vertical slice)

Prove the seams end-to-end before building every module:

1. Minimal `Sentence` + `SentenceId` + `CollectionId` in `lingo-domain`.
2. Minimal `SqliteLibraryStore` (open, migrate to `lingo.library/v1`,
   `insert_drafts`, `list_sentences`) under `lingo-workspace-fs/src/library/`.
3. `lingo extract --apply` committing drafts, and `lingo ls` reading them back.

Evidence: `init → extract → ls` works against a real `library.db`; the
`LibraryStore` port has both a SQLite and an in-memory implementation passing the
same conformance test.

## Phase 1 — Domain library model

Add `CollectionId`, `SentenceId`, `WordKey`, `SectionName`, `SentenceOrder`,
`UtcTimestamp`; `Sentence`, `FieldAuthoritySet`, `SentenceStatus`,
`SentenceProvenance`, `TokenBreakdown`, `SentenceTags`, `SentenceAudio`;
`WordEntry`, `WordMeaning`, `WordOccurrence`; domain validation for human-field
preservation and breakdown/token coverage.

Evidence: valid/invalid value-object tests; closed-set wire-name tests;
human-field preservation tests; word-key normalization tests.

## Phase 2 — Application ports & reports

Split `ports.rs` into `ports/`; add `LibraryStore` + query/mutation models; add
typed reports for extract, enrich, library list, words, audio, package, export,
status; add the in-memory fake store for service tests.

Evidence: use-case tests against the fake store; reports contain typed facts and
no JSON DTOs; public-API tests protect exported names.

## Phase 3 — SQLite adapter

`WorkspaceLayout::library_db()`; `library/connection.rs` (PRAGMAs, open/migrate);
migrations (`0001.sql` or embedded) for the doc 03 schema; `SqliteLibraryStore`
with row DTOs, typed codecs, transaction scripts; audio file store.

Evidence: empty-DB migration test; `library_metadata` test; FK/CHECK/`json_valid`
tests; round-trip tests for sentences/words/authority/provenance/audio;
claim/apply/reorder transaction tests; conformance tests shared by the fake and
SQLite stores.

## Phase 4 — extract / enrich

Add extract/enrich prompt templates; strict parse DTOs for `lingo.extract/v1` and
`lingo.enrich/v1`; implement `prepare_extract`, `apply_extract`, `prepare_enrich`,
`apply_enrich`, `reset_enrichment_claim`; update the `RunJournal` stage enum;
remove/isolate old import/build.

Evidence: parser rejects unknown fields, wrong format, empty replies, surrounding
prose; applying enrich with wrong ids writes nothing; applying enrich that changes
human fields writes nothing; bounded claims prevent duplicate prompts.

## Phase 5 — CLI & Studio edges

Add CLI commands (`extract`, `enrich`, `import`, `ls`, `show`, `organize`,
`words`, `audio`, `package`, `export`, `config`, `status`); split Studio into
transport / handlers / DTO / error; make every Studio mutation call a typed use
case; show CLI-equivalent commands in the UI (doc 06 §5).

Evidence: CLI smoke tests; Studio route tests (`JSON → typed request → typed
report → DTO`); no Studio handler imports `rusqlite` or writes package outputs.

## Phase 6 — Publishers

Change package/export to read from `LibraryStore`; JSON package writer outputs one
file per sentence from library rows; add the optional db package writer (filtered
copy); keep Anki separate; verify manifests/checksums after write.

Evidence: package JSON manifest verifies all files/checksums; db package opens
read-only with expected metadata; APKG contains `collection.anki2` + media map;
publishing fails clearly when required audio is missing.

## Phase 7 — Remove old runtime layers

Remove old source/card runtime scanning from status; remove per-batch
create/replace runtime paths; keep a one-time `lingo migrate` only if scoped;
update docs/fixtures to the new canonical format.

Evidence: no code path writes `input/sentences` or `output/sentences` as runtime
truth; old runtime file names appear only in migration/export tests; an e2e test
covers `init → extract → enrich → audio → package/export`.

## Per-phase closeout checklist

- Public functions read as intent.
- Any new abstraction has a deletion-test answer.
- Any durable/raw string vocabulary has a typed owner.
- Any raw JSON at the edge is parsed into a typed request before behavior.
- Any derived output is explicitly derived and verified.
- Touched code is cleaner, or the deferral is documented.

## Test matrix

```text
unit/domain     ids, text values, tags, authority, status, provenance, word keys, breakdown
unit/application extract apply, enrich claim/apply/reset, status next action, package/audio selection
adapter/sqlite  migrations, constraints, row codecs, transactions, foreign keys, json validity, ordering
adapter/audio   fallback, unavailable backend, empty response, provider error classification
adapter/artifacts package JSON, package DB copy, manifest integrity, path safety, APKG integrity
edge/cli        help, parse errors, scriptable outputs, color/no-color, command flow
edge/studio     route errors, DTO mapping, invalid payload, no direct SQL, no business logic in DTOs
e2e             init -> extract -> enrich -> audio -> package -> export
```
