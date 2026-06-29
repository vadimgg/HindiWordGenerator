# 00 — Architecture Decisions

## Scope

This document owns the rebuild architecture. It intentionally does not rewrite CLI command docs or UI screens; those should be adjusted by the CLI/UI owner to match the architecture facts here.

The tool is personal and local-first. That lets us make clean breaking changes now instead of carrying prototype vocabulary, old schemas, compatibility branches, or dual-write paths.

## Current-code review summary

The current code already has several good instincts:

- a workspace split into domain/application/adapters/CLI crates;
- typed IDs and domain values in `lingo-domain`;
- app use cases behind ports;
- a filesystem workspace adapter;
- model-facing stages as explicit extract/enrich flows;
- audio and package concepts separated from the CLI.

The rebuild should keep those instincts, but simplify and harden the model:

- remove `collection`, `batch`, and `section` as internal product vocabulary;
- make `deck` the only product unit below library;
- make `active` a real approval gate, not a display status;
- make sentence origin durable instead of deriving it from old runs;
- remove durable `enriching` status from sentences;
- make pending runs and `run_sentences` the only authority for in-flight work;
- remove slug-coupled audio paths;
- make `apply` a single strict gate for extract, enrich, and QA;
- move prompt/reply parsing into typed codecs rather than ad-hoc strings;
- keep CLI renderers as presentation only.

## Decision: SQLite remains the authoring truth

`library.db` is the canonical authoring store. It owns decks, sentences, sentence origin, approval state, field authority, tokens, run state, run claims, QA stamps, and audio provenance.

Derived artifacts:

```text
audio/   generated mp3 files; stale/missing can be repaired
out/     publish artifacts; can be regenerated
runs/*/run.json   portable mirror of DB run row; DB wins on disagreement
```

Input artifacts:

```text
raw/     user-owned source material; Lingo reads but does not rewrite
prompts/ optional user-owned prompt overrides
```

## Decision: no prototype compatibility

There is no production data to protect. Do not keep compatibility aliases like `BatchId`, old `collections`, legacy `sections`, or old folder layouts unless the user explicitly says real data must be migrated.

The clean model is:

```text
Library -> Deck -> Sentence
Sentence -> origin / authority / tokens / approval / audio metadata
Run -> run_sentences
```

## Decision: library identity is durable

Every initialized library gets a generated `meta.library_id`. Package exports include it as `source_library_id`.

Use it only for identity decisions, especially import:

```text
same source_library_id      -> same-library restore/sync rules may preserve approval and QA
different/missing source id -> cross-library import rules reset approval by default
```

`library_id` is not a user-visible title and should not be reused across independent libraries.

## Decision: sentence lifecycle is durable readiness only

Persisted lifecycle:

```rust
pub enum SentenceLifecycle {
    Draft,
    Enriched,
}
```

Do not persist `Enriching`. It is a visible state derived from a pending enrich run claim.

```rust
pub enum VisibleSentenceStatus {
    Draft,
    Enriching { run_id: RunId },
    Enriched,
}
```

This prevents impossible dual truths such as:

```text
sentence.status = enriching but no pending claim exists
pending claim exists but sentence.status = draft
```

The database can always answer visible status by joining `sentences`, `run_sentences`, and pending enrich `runs`.

## Decision: approval and QA are separate axes

Approval:

```sql
sentences.active INTEGER NOT NULL DEFAULT 0 CHECK(active IN (0,1))
CHECK (active = 0 OR status = 'enriched')
```

QA:

```sql
sentences.qa_checked_at TEXT NULL
```

Allowed states include:

```text
draft + inactive
enriched + inactive + not QA'd
enriched + active + not QA'd
enriched + active + QA'd
```

Disallowed state:

```text
draft + active
```

QA remains warn-only. A user can approve an enriched sentence before QA. Study/Anki publish should warn if selected approved sentences are not QA'd, but should not hard-block.

## Decision: approval is the study gate

`active = true` means “approved for study.”

Study-facing exports:

```text
study: active enriched rows by default
anki:  active enriched rows by default
```

Package/db exports are lossless and export everything.

An explicit override such as `--include-unapproved` may include enriched inactive rows for study/Anki, but draft rows are still not studyable because they lack required enrichment/tokens.

## Decision: approval must be invalidated by lifecycle downgrade

Any operation that sets `status = draft` must also set `active = 0` in the same domain operation and database transaction.

This resolves the target-edit tension:

```text
Semantic target edit -> lifecycle draft -> active cleared
Audio-only target edit -> lifecycle unchanged -> active unchanged
No-content target edit -> lifecycle unchanged -> active unchanged
```

Automated model rewrites that change study-facing content after approval should clear approval even if lifecycle remains `enriched`:

```text
enrich --force with changed fields/tokens -> active cleared
QA correction that changes fields/tokens -> active cleared
QA clean stamp only -> active unchanged
```

Manual edits to enriched rows may keep approval because the user is the actor making the change. The edit report should make that visible, and the user may pass the normal inactive/active operation if they want to re-review.

## Decision: sentence origin is durable

Origin is not derived from `run_sentences` because run cleanup can delete those rows.

```rust
pub enum SentenceOrigin {
    Generated(GeneratedOrigin),
    Imported(ImportedOrigin),
    Manual(ManualOrigin),
}
```

Rules:

```text
generated -> optional source_extract_run_id, optional source_label
imported  -> source_library_id + source_package_id + source_sentence_id required
manual    -> no source run/package/sentence fields
```

`source_extract_run_id` is informational, not a foreign key. It may outlive the old run row.

`source_label` is a display label such as `raw/ch01.md` or package title. It is not identity.

No extra source timestamp is stored on the sentence. `sentences.created_at` records when the sentence entered this library. Package manifests preserve package generation timestamps.

## Decision: `run_sentences` is canonical

`run_sentences` records every sentence that belongs to a run.

- For `enrich` and `qa`, rows are inserted when the run is prepared/claimed.
- For `extract`, rows are inserted at apply time for the sentences created by that run.
- Rows may remain after apply/reset as run participation history, but sentence origin does not depend on them.

Do not store these columns on `sentences`:

```text
enrich_run_id
qa_run_id
```

Those are caches at best and dual truth at worst.

## Decision: flat audio path

Internal authoring audio path:

```text
audio/<sentence-id>.mp3
```

No deck slug in the path. Deck slugs are mutable. Sentence IDs are permanent. Audio belongs to a sentence.

Exports may arrange audio differently if useful for a target format, but the authoring workspace should never move audio files merely because a deck slug changed.

## Decision: real sentence IDs are opaque `sen-<ulid>` values

Generated sentence IDs use a slug-free format such as:

```text
sen-01jx9m7q8v6f2x4k9d3p1r0t5w
```

The prefix tells humans the kind of ID. The remaining bytes are opaque. Code must not parse a deck slug, position, or timestamp from the ID. If an example in CLI docs shows `sen-ch01-01`, treat it as illustrative sample output only, not the durable ID format.

Forbidden API:

```rust
sentence_id.deck_slug();
sentence_id.position();
sentence_id.created_at_from_ulid();
run_id.stage();
```

Allowed API:

```rust
sentence_id.as_str();
SentenceId::parse(raw);
```

## Decision: target edits are impact-based

Editing target text does not always mean the same thing. Use profile-aware comparison to classify the edit:

```rust
pub enum TargetEditImpact {
    NoContentChange,
    AudioOnlyChange,
    SemanticChange,
}
```

Classifier:

```text
SemanticChange   = profile.target_identity_key(before) != profile.target_identity_key(after)
AudioOnlyChange  = identity same, but profile/audio fingerprint input changes
NoContentChange  = identity same and audio input/fingerprint same
```

Default policy:

| Impact | Action |
|---|---|
| `NoContentChange` | No invalidation. |
| `AudioOnlyChange` | Keep lifecycle, QA, tokens, and approval; audio becomes stale by fingerprint mismatch. |
| `SemanticChange` | Clear AI-authored derived fields, clear tokens, clear QA, mark audio stale, set lifecycle to `draft`, clear approval. Preserve human fields with warnings. |

## Decision: profile participates in normalization

Language profile participates in:

- target identity normalization;
- word-key derivation;
- audio input fingerprint;
- sentence/token validation rules;
- prompt style rules.

This is needed for Japanese and future languages. A Hindi-only implementation can start small, but signatures should already accept a profile.

## Decision: crates start small

Start with five crates:

```text
lingo-domain
lingo-service
lingo-sqlite
lingo-workspace
lingo-cli
```

Split later only when dependency pressure justifies it:

```text
lingo-handoff   prompt rendering + reply codecs
lingo-audio     TTS adapters
lingo-publish   package/study/anki writers
```

A crate is justified when deleting it would spill concrete dependencies, persistence mechanics, or boundary complexity into the wrong layer.

## Decision: `apply` is the commit gate

`apply` owns strict validation and commit for extract/enrich/QA replies.

Guarantees:

```text
read full reply
parse into typed stage DTO
load validation snapshot
validate entire reply before writing
if dry-run, write nothing
if commit, write inside one SQLite transaction
record reply_sha256 and applied_at
same reply after applied is idempotent
changed reply after applied is rejected
failed validation leaves run pending and records validation error
```

No stage-specific command writes model output directly to tables.

## Decision: one pending enrich claim is transaction-enforced

A sentence must not be in two pending enrich runs at once.

SQLite cannot express this as a valid partial unique index because the predicate needs a join to `runs`. The invariant lives in the `BEGIN IMMEDIATE` claim transaction. The implementation must include a double-claim concurrency test.

## Decision: package is the lossless round-trip format

Package JSON must preserve enough data that backup -> import/restore does not silently downgrade a library:

```text
id
status
active
qa_checked_at
origin and source fields
field authority
tokens/breakdown
tags
audio metadata and optional audio file
created_at/updated_at
```

Study and Anki are one-way study targets. They are allowed to filter and reshape.

## Decision: import defaults to safe re-approval

Import rules depend on source identity:

```text
same package source_library_id as destination meta.library_id
  -> preserve sentence IDs, approval, and QA when restoring/updating the same library

different or missing source_library_id
  -> allocate local sentence IDs for new rows
  -> origin = imported
  -> active = false
  -> qa_checked_at = NULL
  -> store source_library_id/source_package_id/source_sentence_id
```

A future explicit trust option may preserve external approval/QA, but default cross-library import should require local re-approval.

## Decision: approval is a workflow step

The default workflow is now:

```text
extract -> apply -> enrich -> apply -> QA recommended -> approve -> audio -> publish
```

Architecture exposes approval as a use case. CLI can surface it as a dedicated command or as bulk edit/approval behavior, but status ranking should treat “enriched but unapproved” as a real gap before study/Anki publish.
