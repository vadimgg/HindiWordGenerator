# 04 — State Machines

## Durable sentence lifecycle

Only two lifecycle values are stored:

```text
Draft
  ├─ apply valid enrich reply -> Enriched
  ├─ edit semantic target -> Draft
  └─ import incomplete sentence -> Draft

Enriched
  ├─ approve -> Enriched + active
  ├─ unapprove -> Enriched + inactive
  ├─ edit semantic target -> Draft + derived fields invalidated + active cleared
  ├─ enrich --force apply with changed fields/tokens -> Enriched + QA cleared + active cleared
  ├─ apply QA clean result -> Enriched + qa_checked_at set + active unchanged
  ├─ apply QA corrections -> Enriched + qa_checked_at set + active cleared if study-facing fields changed
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

## Approval state machine

Approval is stored as `active`, but the domain name is approval.

```text
Unapproved
  ├─ approve enriched sentence -> Approved
  └─ approve draft sentence -> error CannotApproveDraft

Approved
  ├─ unapprove -> Unapproved
  ├─ semantic target edit -> Unapproved + Draft
  ├─ automated content rewrite -> Unapproved + Enriched
  └─ audio-only edit / no-content edit / QA clean stamp -> Approved
```

Hard invariant:

```text
Approved => lifecycle == Enriched
```

QA is not hard-required for approval. Publish warns, not blocks, when approved selected rows are unQA'd.

## Origin creation transitions

Origin is durable and survives run cleanup.

```text
Apply extract run
  -> create sentence with origin = generated
  -> source_extract_run_id = run id string
  -> source_label = raw path/deck source when available
  -> add run_sentences rows as run participation history

Manual add sentence
  -> create sentence with origin = manual
  -> source_label optional

Import package, cross-library default
  -> create sentence with origin = imported
  -> source_library_id = package.source_library_id
  -> source_package_id = package.package_id
  -> source_sentence_id = package sentence id
  -> active = false
  -> qa_checked_at = NULL

Import package, same-library restore
  -> preserve sentence id, active, qa_checked_at, authority, tokens, and origin
  -> update/restore from package data according to import conflict policy
```

Do not infer origin from the presence or absence of `run_sentences` rows.

## Claim state machine

Claiming work creates a run and run-sentence rows; it does not mutate sentence lifecycle.

```text
No pending claim
  ├─ prepare enrich -> pending enrich run + run_sentences
  └─ prepare QA -> pending QA run + run_sentences

Pending claim
  ├─ apply valid reply -> run applied, rows kept as participation history
  ├─ reset run -> run reset, rows kept or deleted by policy
  ├─ clean abandoned -> run abandoned/deleted, rows cascade if deleted
  └─ apply invalid reply -> run remains pending, validation_error set
```

A sentence must not have two pending enrich claims at once. The following index is **illustrative only** and is not valid SQLite:

```sql
-- illustrative only; invalid SQLite because partial indexes cannot contain subqueries
CREATE UNIQUE INDEX one_pending_enrich_claim_per_sentence
ON run_sentences(sentence_id)
WHERE run_id IN (
  SELECT id FROM runs WHERE stage = 'enrich' AND status = 'pending'
);
```

The real invariant lives in the same `BEGIN IMMEDIATE` transaction that creates the claim:

```sql
SELECT rs.sentence_id
FROM run_sentences rs
JOIN runs r ON r.id = rs.run_id
WHERE r.stage = 'enrich'
  AND r.status = 'pending'
  AND rs.sentence_id IN (...selected...)
LIMIT 1;
```

Implementation evidence must include a double-claim concurrency test.

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
8. Load validation snapshot: deck, sentences, claims, human fields, origin/source context, profile.
9. Validate entire reply without writing.
10. If dry-run, return ApplyPreview.
11. BEGIN IMMEDIATE.
12. Re-check run status/hash and claim set.
13. Apply stage-specific mutations.
14. Enforce approval invariant and clear active if this mutation invalidated approval.
15. Update runs row last: status/applied_at/reply_sha256.
16. COMMIT.
17. Return typed report.
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

QA correction policy:

```text
clean/no corrections      -> active unchanged
correction changes fields -> active cleared, because approval applied to old content
human field rejected      -> active unchanged for that rejected field
```

## Target edit impact classifier

Classifier inputs:

```rust
pub struct TargetEditClassifierInput<'a> {
    pub profile: &'a dyn LanguageProfile,
    pub before: &'a TargetText,
    pub after: &'a TargetText,
    pub audio_backend: Option<AudioBackendId>,
    pub audio_voice: Option<&'a AudioVoice>,
    pub audio_model: Option<&'a AudioModel>,
}
```

Rule:

```text
before_identity = profile.target_identity_key(before)
after_identity  = profile.target_identity_key(after)

if before_identity != after_identity:
    SemanticChange
else if audio_fingerprint_input(before) != audio_fingerprint_input(after):
    AudioOnlyChange
else:
    NoContentChange
```

For a sentence with no existing audio, `AudioOnlyChange` still means enrichment/QA/tokens/approval stay valid; there is simply no stale file yet.

## Target edit invalidation

```text
NoContentChange
  -> keep lifecycle, QA, tokens, audio, active

AudioOnlyChange
  -> keep lifecycle, QA, tokens, active
  -> audio becomes stale by fingerprint mismatch

SemanticChange + default policy
  -> clear AI-authored romanisation, english, literal, register
  -> preserve human-authored fields and warn
  -> clear tokens
  -> clear qa_checked_at
  -> set lifecycle = draft
  -> clear active because active implies enriched
  -> audio becomes stale by fingerprint mismatch
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
    sentence.downgrade_to_draft(); // also clears approval
    sentence.mark_audio_stale_by_fingerprint();
    report.approval_cleared_because_draft();
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

## Publish selection state

Study-facing selection:

```text
Default study/anki selection:
  status = enriched
  active = true

With include-unapproved:
  status = enriched
  active = true or false

Never include draft rows in study/anki.
```

Package/db selection is lossless and can include draft, inactive, unQA'd, and missing-audio rows.

## Status ranking

Default ranking should treat approval as a real gap:

```text
1. pending reply that can be applied
2. draft sentences ready to enrich
3. enriched but not QA'd sentences ready for QA
4. enriched but unapproved sentences ready for approval
5. approved sentences missing or stale audio
6. approved sentences ready to publish
7. done
```

QA remains recommended, not required by approval or schema.

## Publish quality gate

```text
package/db: export everything losslessly
study/anki default: approved enriched rows only
study/anki with include-unapproved: enriched rows regardless of active
study/anki: warn if selected rows are unQA'd; skip/report missing audio
```

No hard block for unQA'd sentences in a personal tool.
