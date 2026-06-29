# 13 — Refactor Plan

## Principle

Rebuild in vertical slices. Do not migrate every command horizontally before proving one full path through domain, repository, workspace, handoff, service, and CLI.

## Phase 0 — Freeze architecture vocabulary

- Accept this architecture pack as the source for implementation.
- Claude updates CLI docs to match these facts:
  - persisted status is `draft|enriched`;
  - `enriching` is visible/derived;
  - active is a badge/flag;
  - audio path is `audio/<sentence-id>.mp3`;
  - schema sample is v1;
  - apply supports dry-run, oldest/all, no prompt in non-interactive mode.
- Remove old `batch/collection/section` vocabulary from new code.

## Phase 1 — Domain core

Implement and test:

- IDs/slugs as opaque typed values;
- `SentenceLifecycle = Draft | Enriched`;
- `VisibleSentenceStatus` derivation;
- `FieldAuthoritySet`;
- `SentenceTokenBreakdown`;
- `RunStage`, `RunStatus`;
- `AudioInputFingerprint`;
- language profile trait + Hindi built-in profile;
- target edit impact classification.

Evidence:

```bash
cargo test -p lingo-domain
```

## Phase 2 — SQLite schema and repository

Implement:

- schema initialization from `schema_v02.sql`;
- connection pragmas;
- row codecs;
- repository fake conformance tests;
- SQLite repository operations for init/status/decks/sentences/runs.

Tracer evidence:

```text
create temp library.db
insert deck + draft sentence
create pending enrich run + run_sentences
query visible status => enriching
```

## Phase 3 — Workspace adapter

Implement:

- typed layout;
- safe relative paths;
- config load/write;
- `init --example` file generation;
- run file write/read/repair;
- flat audio path policy;
- atomic write verification.

Evidence:

```text
deck slug rename has no workspace audio operation
run.json repaired from DB snapshot
```

## Phase 4 — Handoff + apply tracer bullet

Implement only enough handoff for extract first:

- extract task rendering;
- extract reply parser;
- apply dry-run;
- apply commit transaction;
- idempotency.

Tracer path:

```bash
lingo init tmp --example
lingo extract raw/example.md --deck ch01
# write minimal valid reply.yaml
lingo apply --dry-run runs/ch01-extract-*/
lingo apply runs/ch01-extract-*/
lingo status
```

## Phase 5 — Enrich and visible claims

Implement:

- claim enrich run transaction;
- `run_sentences` claim validation;
- enrich reply codec;
- token replacement;
- word-key derivation through profile;
- visible `enriching` status in status/ls reports.

Evidence:

```text
sentence lifecycle remains draft while claimed
visible status says enriching
apply enrich changes lifecycle to enriched
run_sentences remains as provenance
```

## Phase 6 — QA and edit invalidation

Implement:

- QA claim/apply;
- QA checked stamps;
- edit sentence use case;
- target edit impact classification;
- derived invalidation policy;
- active flag operations.

Evidence:

```text
semantic target edit clears AI fields/tokens/QA, keeps active
human fields are preserved and reported
```

## Phase 7 — Audio

Implement:

- gTTS backend metadata;
- deterministic fake synthesizer for tests;
- audio fingerprint;
- stale/missing selection;
- flat file write;
- `sentence_audio` metadata.

Evidence:

```text
audio path does not include deck slug
deck rename leaves audio file untouched
target exact edit marks audio stale
```

## Phase 8 — Publish/import

Implement:

- package export/import first;
- study export second;
- Anki export third;
- DB copy last.

Evidence:

```text
package round-trip preserves sentences/tokens/authority/audio metadata
study skips missing audio and warns on unQA'd
Anki GUID derives from sentence ID
```

## Phase 9 — Cleanup and deletion

Delete or rewrite prototype code:

- old batch/collection code;
- old JSON breakdown fields;
- old deck-folder audio paths;
- old status enum with active/enriching;
- old apply paths that bypass strict run validation.

Do not enrich legacy code scheduled for deletion.

## Implementation stop rules

Stop and update architecture docs if implementation discovers:

- a second durable source of truth;
- a crate boundary that becomes pass-through;
- a need to store `enriching` after all;
- audio paths needing deck slug in authoring workspace;
- language profile not being sufficient for Japanese word keys;
- `apply` needing to write outside one DB transaction.
