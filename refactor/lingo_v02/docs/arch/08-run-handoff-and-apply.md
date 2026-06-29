# 08 — Run Handoff and Apply

## Run directory

```text
runs/<run-id>/
  task.md
  reply.yaml      # extract by default
  reply.json      # enrich/qa by default
  run.json
```

`task.md` is human-readable and contract-bearing. `reply.*` is the model output. `run.json` is a mirror. The DB `runs` row is authoritative.

## Run creation

All model-facing commands create the same durable run shape:

```rust
pub struct CreateRunRequest {
    pub stage: RunStage,
    pub deck: Option<DeckId>,
    pub reply_name: RunReplyName,
    pub selected_sentences: RunSentenceSelection,
}
```

For enrich/QA, `selected_sentences` becomes `run_sentences` immediately. For extract, selected sentences are unknown until apply.

## Claiming enrich work

```rust
pub fn prepare_enrich(app: &LingoServices<'_>, req: PrepareEnrichRequest) -> Result<RunPreparedReport, AppError> {
    let config = app.deps.workspace.read_config()?;
    let profile = app.deps.profiles.get(&config.target.profile)?;

    let claim = app.deps.repo.claim_enrich_run(ClaimRun {
        deck: req.deck,
        limit: req.limit,
        mode: req.mode,
        created_at: app.deps.clock.now(),
        run_id: app.deps.ids.new_run_id(req.deck.as_ref(), RunStage::Enrich),
    })?;

    let task = app.deps.handoff.render_task(PromptTaskRequest::enrich(&claim, profile))?;
    let files = app.deps.workspace.write_run_files(&claim.run, &task)?;

    Ok(RunPreparedReport {
        run: claim.run,
        task_path: files.task_path,
        reply_path: files.reply_path,
        claimed: claim.sentences.len(),
        terminal: TerminalDirective::Next(NextAction::ApplyRun { run_id: claim.run.id().clone() }),
    })
}
```

## Reply parsing

`apply` parses by stage from the DB run row, not by CLI argument.

```rust
match run.stage() {
    RunStage::Extract => parse_extract_reply(bytes),
    RunStage::Enrich => parse_enrich_reply(bytes),
    RunStage::Qa => parse_qa_reply(bytes),
}
```

## Validation snapshot

Before validating, load all facts needed to reject bad replies without writing:

```rust
pub struct ApplyValidationSnapshot {
    pub run: RunWithSentences,
    pub deck: Option<Deck>,
    pub library: LibraryIdentity,
    pub sentences: Vec<Sentence>,
    pub field_authority: FieldAuthorityRows,
    pub profile: Box<dyn LanguageProfileSnapshot>,
}
```

The snapshot should be immutable to validation code.

## Extract validation

Validate that:

- reply contains only the expected top-level format/version;
- every sentence has target text;
- optional learner-provided fields are marked with correct authority;
- no duplicate target identity exists inside the reply;
- generated IDs are assigned by Lingo, not trusted from model unless explicitly part of package import;
- raw source/deck context matches the run;
- generated sentence origin can be constructed from the run id and source label.

Extract should not mix in package-import duplicate rules. Import is a separate direct workflow.

## Enrich validation

Validate that:

- every returned sentence ID belongs to the run;
- no unclaimed sentence is present;
- required enrichment fields are present unless a human field already exists;
- tokens cover the target in order according to profile policy;
- `register` is one of the typed register variants;
- human-authored fields are not overwritten;
- word keys derive through the profile.

## QA validation

Validate that:

- every correction is for a sentence in the QA run;
- corrections are field-scoped;
- human-authored fields are not touched;
- clean rows may be omitted or explicitly marked clean, according to the QA reply contract;
- applying the reply stamps `qa_checked_at` for every sentence in the run.

## Dry-run

```rust
pub struct ApplyPreview {
    pub run: RunId,
    pub stage: RunStage,
    pub reply_sha256: ContentHash,
    pub would_create_sentences: usize,
    pub would_update_sentences: usize,
    pub would_update_tokens: usize,
    pub rejected_human_overwrites: Vec<HumanOverwriteAttempt>,
    pub warnings: Vec<ApplyWarning>,
}
```

`--dry-run` writes nothing, including no validation error. Invalid dry-run can still return a validation error to the caller.

## Idempotency

```rust
impl Run {
    pub fn already_applied_decision(&self, incoming_hash: &ContentHash) -> Result<Option<AlreadyAppliedReport>, ApplyError> {
        if self.status() != RunStatus::Applied {
            return Ok(None);
        }

        match self.reply_sha256() {
            Some(existing) if existing == incoming_hash => Ok(Some(AlreadyAppliedReport::same_reply(self.id().clone()))),
            Some(existing) => Err(ApplyError::AlreadyAppliedDifferentReply {
                run: self.id().clone(),
                existing: existing.clone(),
                incoming: incoming_hash.clone(),
            }),
            None => Err(ApplyError::AppliedRunMissingHash { run: self.id().clone() }),
        }
    }
}
```

## Transaction shape

```rust
pub fn apply_run_transaction(&self, commit: ApplyRunCommit) -> Result<ApplyCommitReport, RepoError> {
    let tx = self.begin_immediate()?;
    let run = tx.get_run_for_update(&commit.run_id)?;
    run.reject_if_not_pending_or_idempotent(&commit.reply_sha256)?;

    match commit.stage_payload {
        ApplyStageCommit::Extract(payload) => tx.apply_extract(payload)?, // creates origin=generated
        ApplyStageCommit::Enrich(payload) => tx.apply_enrich(payload)?,
        ApplyStageCommit::Qa(payload) => tx.apply_qa(payload)?,
    }

    tx.mark_run_applied(&commit.run_id, &commit.reply_sha256, commit.applied_at)?;
    tx.commit()?;
    Ok(commit.report)
}
```

## Approval invalidation during apply

Stage commits must explicitly report whether they changed study-facing content that had been approved.

```rust
pub enum ApprovalInvalidation {
    None,
    ClearedBecauseDraft,
    ClearedBecauseAutomatedContentChanged,
}
```

Rules:

```text
extract apply -> new rows are unapproved by default
enrich apply from draft -> rows become enriched but unapproved
enrich --force with changed fields/tokens -> clear active
QA clean stamp -> keep active
QA corrections that change fields/tokens -> clear active
```

The transaction must leave no row with `active = true` and `status = draft`.

## Validation failure

On commit-mode validation failure, record the error on the run and leave it pending:

```rust
repo.record_validation_error(run_id, ValidationErrorText::from(&error))?;
```

The agent fixes the same reply file and re-applies the same run.

## Runs cleanup

`runs clean` without abandoned mode deletes only applied run folders. DB rows may remain as run participation history. Sentence origin does not depend on them.

`runs clean --abandoned` may mark pending/reset runs abandoned or delete them, depending on chosen retention policy. If deleting DB rows, `run_sentences` cascades. This must not erase sentence origin because origin is stored on `sentences`. Never delete a pending run with a reply file silently; the report should name it.
