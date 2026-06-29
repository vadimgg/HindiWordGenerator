# 13 — Refactor Plan

## Principle

Rebuild in vertical slices. Do not migrate every command horizontally before proving one full path through domain, repository, workspace, handoff, service, and CLI.

## Phase 0 — Freeze architecture vocabulary

- Accept this architecture pack as the source for implementation.
- Final user-facing vocabulary is mandatory:
  - `deck`, not batch/group/collection;
  - `slug` for the short user-facing id;
  - `approve` / `unapprove`, not active/inactive/enable/disable;
  - `extract`, not add/import raw;
  - `import`, not add/import package;
  - `apply` as the single model-reply commit gate;
  - `publish`, not package/export.
- CLI docs must match these facts:
  - persisted status is `draft|enriched`;
  - `enriching` is visible/derived;
  - active is approval, not a status;
  - active requires enriched;
  - semantic target edits clear approval because they downgrade to draft;
  - origin is durable and visible in show/import/package reports;
  - study/anki default to approved rows with an include-unapproved override;
  - package round-trip preserves active, QA, authority, tokens, audio, origin;
  - audio path is `audio/<sentence-id>.mp3`;
  - schema sample is v1;
  - apply supports dry-run, oldest/all, no prompt in non-interactive mode.
- Remove old `batch/collection/section` vocabulary from new code. This is
  mandatory cleanup, not optional polish. New code must use `deck` from day one.

## Phase 1 — Minimal CLI spine: schema → init → extract → apply → status → ls → show

Goal: prove one full human/agent workflow through the real domain, SQLite
repository, workspace files, handoff rendering, reply validation, and CLI.

Implement:

- IDs/slugs as opaque typed values, with real sentence IDs as `sen-<ulid>`;
- `LibraryId` and package source identity values;
- `SentenceLifecycle = Draft | Enriched`;
- `VisibleSentenceStatus` derivation;
- `SentenceOrigin = Generated | Imported | Manual`;
- `ApprovalState` and the `active => enriched` invariant;
- `FieldAuthoritySet`;
- `SentenceTokenBreakdown`;
- `RunStage`, `RunStatus`;
- `AudioInputFingerprint`;
- language profile trait + Hindi built-in profile;
- target edit impact classification;
- schema initialization from `schema_v02.sql`;
- connection pragmas;
- row codecs;
- repository fake conformance tests;
- SQLite repository operations for init/status/decks/sentences/runs;
- origin columns and active invariant;
- typed layout;
- safe relative paths;
- config load/write;
- `init --example` file generation;
- run file write/read/repair;
- flat audio path policy;
- atomic write verification;
- built-in template lookup from `crates/lingo-handoff/templates/`;
- workspace override lookup from `prompts/`;
- extract task rendering;
- extract reply parser;
- apply dry-run;
- apply commit transaction;
- generated origin creation;
- idempotency;
- `lingo init`;
- `lingo extract`;
- `lingo apply`;
- `lingo status`;
- `lingo ls`;
- `lingo show`.

Tracer path:

```bash
lingo init tmp --example
lingo extract raw/example.md --deck ch01
# write minimal valid reply.yaml
lingo apply --dry-run runs/<run>/
lingo apply runs/<run>/
lingo status
lingo ls
lingo show <sentence-id>
```

Evidence:

```bash
cargo test -p lingo-domain
cargo test -p lingo-db
cargo test -p lingo-cli init_extract_apply_status_ls_show
```

```text
create temp library.db
insert deck + draft generated sentence
create pending enrich run + run_sentences
query visible status => enriching
attempt active draft => rejected
deck slug rename has no workspace audio operation
run.json repaired from DB snapshot
```

## Phase 2 — Enrich

Implement:

- claim enrich run transaction;
- `run_sentences` claim validation inside `BEGIN IMMEDIATE`;
- double-claim concurrency test;
- enrich reply codec;
- token replacement;
- word-key derivation through profile;
- visible `enriching` status in status/ls reports.

Evidence:

```text
sentence lifecycle remains draft while claimed
visible status says enriching
apply enrich changes lifecycle to enriched and keeps active=false
run_sentences remains as participation history
```

## Phase 3 — QA

Implement:

- QA claim/apply;
- QA checked stamps;
- QA correction approval invalidation;
- `lingo qa`;
- QA reply validation and patching.

Evidence:

```text
QA clean stamp keeps approval unchanged
QA correction that changes study-facing fields clears approval
human-authority overwrite attempts are rejected
```

## Phase 4 — Approval and edit invalidation

Implement:

- edit sentence use case;
- target edit impact classification;
- derived invalidation policy;
- approval use case;
- `lingo approve <deck>`;
- `lingo approve <sentence-id>`;
- `lingo approve <deck> --all`;
- `lingo unapprove <deck|sentence-id>`.

Do **not** implement `approve --interactive` in this phase. Interactive/TUI
approval can wait until after the viewer is rebuilt; Phase 4 needs the
non-interactive approval spine only.

Evidence:

```text
semantic target edit clears AI fields/tokens/QA and active, because row becomes draft
human fields are preserved and reported
approve rejects draft and approves enriched
```

## Phase 5 — Audio

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

## Phase 6 — Publish

Implement:

- publish package first;
- publish study second, including the `fixtures/study-export/` contract;
- publish Anki third;
- DB copy last;

Evidence:

```text
package round-trip preserves sentences/tokens/authority/audio metadata/approval/QA/origin
study/anki default to approved enriched rows only
study/anki include unapproved only with explicit selection
Anki GUID derives from sentence ID
```

## Phase 7 — Import

Implement:

- `lingo import`;
- package validation;
- same-library restore vs cross-library import approval policy;
- slug conflict handling;
- origin preservation.

Evidence:

```text
same-library import preserves sentence IDs, approval, QA, authority, tokens, audio, origin
cross-library import resets approval/QA by default
invalid package states, including approved draft rows, are rejected
```

## Phase 8 — Doctor, runs, and deck management

Implement:

- `lingo doctor`;
- `lingo runs ls`;
- `lingo runs clean`;
- `lingo deck list`;
- `lingo deck show`;
- `lingo deck set`;
- `lingo deck delete`;
- final CLI help/color/result contract tests.

Evidence:

```text
doctor reports schema mismatch, missing tools, stale audio, broken audio metadata, abandoned runs
runs clean never deletes unapplied work without an explicit safe policy
deck delete removes DB rows and flat sentence audio files for that deck
```

## Phase 9 — Delete obsolete vocabulary and compatibility shims

Delete or rewrite prototype code:

- old batch/collection code;
- old JSON breakdown fields;
- old deck-folder audio paths;
- old status enum with active/enriching;
- old apply paths that bypass strict run validation;
- old import code that lacks durable origin/approval policy.

Do not enrich legacy code scheduled for deletion.

## Implementation stop rules

Stop and update architecture docs if implementation discovers:

- a second durable source of truth;
- a crate boundary that becomes pass-through;
- a need to store `enriching` after all;
- a need for `active=true` on draft rows;
- a need to trust cross-library approvals by default;
- audio paths needing deck slug in authoring workspace;
- language profile not being sufficient for Japanese word keys;
- `apply` needing to write outside one DB transaction.
