# 04 — State Machines

## Durable sentence lifecycle

Only two lifecycle values are stored:

```text
Draft
  ├─ apply valid enrich reply -> Enriched
  ├─ edit semantic target -> Draft
  └─ import incomplete sentence -> Draft

Enriched
  ├─ edit semantic target -> Draft + derived fields invalidated
  ├─ enrich --force apply -> Enriched with new AI fields, QA cleared
  ├─ apply QA -> Enriched + qa_checked_at set
  └─ edit active flag -> Enriched, active changed
```

There is no durable `Enriching` sentence state.

## Visible status derivation

```text
Visible Enriching = sentence is claimed by a pending enrich run
```

SQL shape:

```sql
SELECT s.*,
       pr.run_id AS pending_enrich_run_id
FROM sentences s
LEFT JOIN (
  SELECT rs.sentence_id, rs.run_id
  FROM run_sentences rs
  JOIN runs r ON r.id = rs.run_id
  WHERE r.stage = 'enrich' AND r.status = 'pending'
) pr ON pr.sentence_id = s.id;
```

Domain shape:

```rust
pub fn visible_status(
    lifecycle: SentenceLifecycle,
    pending_enrich_run: Option<RunId>,
) -> VisibleSentenceStatus {
    match (lifecycle, pending_enrich_run) {
        (_, Some(run_id)) => VisibleSentenceStatus::Enriching { run_id },
        (SentenceLifecycle::Draft, None) => VisibleSentenceStatus::Draft,
        (SentenceLifecycle::Enriched, None) => VisibleSentenceStatus::Enriched,
    }
}
```

## Claim state machine

Claiming work creates a run and run-sentence rows; it does not mutate sentence lifecycle.

```text
No pending claim
  ├─ prepare enrich -> pending enrich run + run_sentences
  └─ prepare QA -> pending QA run + run_sentences

Pending claim
  ├─ apply valid reply -> run applied, rows kept as provenance
  ├─ reset run -> run reset, rows kept or deleted by policy
  ├─ clean abandoned -> run abandoned/deleted, rows cascade
  └─ apply invalid reply -> run remains pending, validation_error set
```

A sentence must not have two pending enrich claims at once. Enforce with service validation and a SQLite partial unique index if practical.

```sql
CREATE UNIQUE INDEX one_pending_enrich_claim_per_sentence
ON run_sentences(sentence_id)
WHERE run_id IN (
  SELECT id FROM runs WHERE stage = 'enrich' AND status = 'pending'
);
```

SQLite does not allow subqueries in partial index predicates, so enforce this with transaction validation:

```sql
SELECT rs.sentence_id
FROM run_sentences rs
JOIN runs r ON r.id = rs.run_id
WHERE r.stage = 'enrich'
  AND r.status = 'pending'
  AND rs.sentence_id IN (...selected...)
LIMIT 1;
```

## Run lifecycle

Durable run status:

```text
Pending
  ├─ apply --dry-run valid -> Pending
  ├─ apply invalid -> Pending + last_validation_error
  ├─ apply valid -> Applied + reply_sha256 + applied_at
  ├─ reset -> Reset
  └─ abandon/clean -> Abandoned or deleted

Applied
  ├─ apply same reply_sha256 -> idempotent no-op
  └─ apply different reply_sha256 -> error already_applied_different_reply

Reset
  └─ not applied again

Abandoned
  └─ not applied again
```

`failed` is a CLI display state:

```rust
pub enum VisibleRunStatus {
    Pending,
    FailedValidation,
    Applied,
    Reset,
    Abandoned,
}

impl Run {
    pub fn visible_status(&self) -> VisibleRunStatus {
        match (self.status, self.last_validation_error.is_some()) {
            (RunStatus::Pending, true) => VisibleRunStatus::FailedValidation,
            (RunStatus::Pending, false) => VisibleRunStatus::Pending,
            (RunStatus::Applied, _) => VisibleRunStatus::Applied,
            (RunStatus::Reset, _) => VisibleRunStatus::Reset,
            (RunStatus::Abandoned, _) => VisibleRunStatus::Abandoned,
        }
    }
}
```

## Apply phases

```text
1. Resolve target run from argument or single pending run.
2. Load DB run row; DB status is authoritative.
3. Read run.json only for portable paths/stage mirror; repair if stale.
4. Read reply file bytes.
5. Compute reply_sha256.
6. If run is applied:
     same hash -> idempotent AlreadyApplied report
     different hash -> validation error
7. Parse reply into typed stage DTO.
8. Load validation snapshot: deck, sentences, claims, human fields.
9. Validate entire reply without writing.
10. If dry-run, return ApplyPreview.
11. BEGIN IMMEDIATE.
12. Re-check run status/hash and claim set.
13. Apply stage-specific mutations.
14. Update runs row last: status/applied_at/reply_sha256.
15. COMMIT.
16. Return typed report.
```

## Apply decision types

```rust
pub enum ApplyMode {
    DryRun,
    Commit,
}

pub struct ApplyRequest {
    pub target: ApplyTarget,
    pub mode: ApplyMode,
    pub run_selection: ApplyRunSelection,
}

pub enum ApplyRunSelection {
    Explicit(RunId),
    SinglePending,
    OldestPending,
    AllPending,
}

pub enum ApplyOutcome {
    WouldApply(ApplyPreview),
    Applied(ApplyCommitReport),
    AlreadyApplied(AlreadyAppliedReport),
}
```

## Multiple pending runs

No service should prompt. If the caller asks for `SinglePending` and several runs exist:

```rust
pub enum ChoiceRequiredError {
    MultiplePendingRuns { pending: Vec<RunSummary>, suggested: RunId },
}
```

CLI may turn this into a human message. JSON output should expose `multiple_pending_runs` and a copyable suggested command.

## QA state

```rust
pub enum QaState {
    Unchecked,
    Checked { at: UtcTimestamp },
}

impl QaState {
    pub fn needs_qa(lifecycle: SentenceLifecycle, qa: &QaState) -> bool {
        lifecycle == SentenceLifecycle::Enriched && matches!(qa, QaState::Unchecked)
    }
}
```

Applying a QA reply stamps every sentence in the QA run, including clean rows with no corrections.

## Target edit invalidation

```text
NoContentChange
  -> keep lifecycle, QA, tokens, audio

AudioOnlyChange
  -> keep lifecycle, QA, tokens
  -> mark audio stale

SemanticChange + default policy
  -> clear AI-authored romanisation, english, literal, register
  -> preserve human-authored fields and warn
  -> clear tokens
  -> clear qa_checked_at
  -> set lifecycle = draft
  -> mark audio stale
  -> active unchanged
```

Sample domain operation:

```rust
pub fn invalidate_after_semantic_target_change(
    sentence: &mut Sentence,
    report: &mut EditInvalidationReport,
) {
    for field in SentenceField::derived_text_fields() {
        if sentence.authority().may_replace(field) {
            sentence.clear_field(field);
            report.invalidated(field);
        } else {
            report.preserved_human(field);
        }
    }

    sentence.clear_tokens();
    sentence.clear_qa();
    sentence.set_lifecycle(SentenceLifecycle::Draft);
    sentence.mark_audio_stale();
    // active intentionally unchanged
}
```

## Audio state

Audio state is derived from DB metadata, deterministic path policy, file existence, and current fingerprint.

```rust
pub enum AudioState {
    Missing,
    Present { stale: bool, path: AudioRelativePath },
    Broken { expected_path: AudioRelativePath, reason: AudioBrokenReason },
}
```

`lingo audio` default selection is `MissingOrStale`. `--force` selects all rows in scope.

## Publish quality gate

```text
package/db: no QA gate; include everything losslessly
study/anki: warn if unQA'd; skip/report missing audio; --allow-unqa suppresses warning
```

No hard block for unQA'd sentences in a personal tool.
