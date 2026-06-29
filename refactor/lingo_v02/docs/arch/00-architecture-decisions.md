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
- remove durable `enriching` status from sentences;
- make pending runs and `run_sentences` the only authority for in-flight work;
- remove slug-coupled audio paths;
- make `apply` a single strict gate for extract, enrich, and QA;
- move prompt/reply parsing into typed codecs rather than ad-hoc strings;
- keep CLI renderers as presentation only.

## Decision: SQLite remains the authoring truth

`library.db` is the canonical authoring store. It owns decks, sentences, field authority, tokens, run state, run claims, QA stamps, and audio provenance.

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
Run -> run_sentences
Sentence -> tokens / authority / audio metadata
```

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

## Decision: curation and QA are separate axes

Curation:

```sql
sentences.active INTEGER NOT NULL DEFAULT 0 CHECK(active IN (0,1))
```

QA:

```sql
sentences.qa_checked_at TEXT NULL
```

A sentence can be:

```text
draft + inactive
enriched + inactive + not QA'd
enriched + active + not QA'd
enriched + active + QA'd
```

`active` is not lifecycle. `qa_checked_at` is not lifecycle.

## Decision: `run_sentences` is canonical

`run_sentences` records every sentence that belongs to a run.

- For `enrich` and `qa`, rows are inserted when the run is prepared/claimed.
- For `extract`, rows are inserted at apply time for the sentences created by that run.
- Rows remain after apply/reset as provenance unless the run is intentionally deleted.

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

## Decision: IDs are opaque

IDs may be human-readable at creation time:

```text
sen-ch01-01
ch01-enrich-9b2c
```

But code must not parse meaning out of them. A sentence ID's slug-looking segment records where the sentence was born, not where it currently lives. A deck rename does not rename sentence IDs, run IDs, Anki GUIDs, study progress keys, or audio filenames.

Forbidden API:

```rust
sentence_id.deck_slug();
sentence_id.position();
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

Default policy:

| Impact | Action |
|---|---|
| `NoContentChange` | No invalidation. |
| `AudioOnlyChange` | Mark audio stale; keep enrichment and QA. |
| `SemanticChange` | Clear AI-authored derived fields, clear tokens, clear QA, mark audio stale, set lifecycle to `draft`. Preserve human fields with warnings. |

Do **not** automatically set `active = false`. Curation is a user decision. A user edit might make a sentence more trustworthy, not less.

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
validate against a DB snapshot
if --dry-run: report preview and write nothing
if commit: one SQLite transaction
idempotent same-reply re-apply
reject already-applied different reply
record reply_sha256, applied_at, validation_error
```

No model reply ever writes directly to the database.
