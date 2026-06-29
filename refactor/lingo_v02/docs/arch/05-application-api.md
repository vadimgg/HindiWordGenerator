# 05 — Application API

## Style

Use cases accept typed requests and return typed reports. They do not print, color, serialize JSON, or inspect clap args.

A public use-case function should read as intent:

```rust
pub fn apply_run(app: &LingoServices<'_>, request: ApplyRequest) -> Result<ApplyReport, AppError> {
    let target = resolve_apply_target(app, request.target)?;
    let packet = load_apply_packet(app, target)?;
    let parsed = parse_stage_reply(app, &packet)?;
    let snapshot = load_validation_snapshot(app, &packet.run)?;
    let preview = validate_reply_against_snapshot(&parsed, &snapshot)?;

    if request.mode == ApplyMode::DryRun {
        return Ok(ApplyReport::dry_run(preview));
    }

    let commit = app.repo.apply_run_transaction(ApplyRunCommit::new(packet, parsed, preview))?;
    Ok(ApplyReport::committed(commit))
}
```

## Service facade

```rust
pub struct ServiceDeps<'a> {
    pub repo: &'a dyn LibraryRepository,
    pub workspace: &'a dyn WorkspacePort,
    pub profiles: &'a dyn ProfileCatalog,
    pub handoff: &'a dyn HandoffPort,
    pub audio: Option<&'a dyn AudioSynthesizer>,
    pub publishers: Option<&'a dyn PublisherRegistry>,
    pub clock: &'a dyn Clock,
    pub ids: &'a dyn IdGenerator,
}

pub struct LingoServices<'a> {
    deps: ServiceDeps<'a>,
}

impl<'a> LingoServices<'a> {
    pub fn new(deps: ServiceDeps<'a>) -> Self { Self { deps } }

    pub fn status(&self, req: StatusRequest) -> Result<StatusReport, AppError>;
    pub fn init(&self, req: InitRequest) -> Result<InitReport, AppError>;
    pub fn prepare_extract(&self, req: PrepareExtractRequest) -> Result<RunPreparedReport, AppError>;
    pub fn prepare_enrich(&self, req: PrepareEnrichRequest) -> Result<RunPreparedReport, AppError>;
    pub fn prepare_qa(&self, req: PrepareQaRequest) -> Result<RunPreparedReport, AppError>;
    pub fn apply(&self, req: ApplyRequest) -> Result<ApplyReport, AppError>;
    pub fn edit_sentence(&self, req: EditSentenceRequest) -> Result<EditSentenceReport, AppError>;
    pub fn approve_sentences(&self, req: ApproveSentencesRequest) -> Result<ApprovalReport, AppError>;
    pub fn generate_audio(&self, req: AudioGenerateRequest) -> Result<AudioGenerateReport, AppError>;
    pub fn publish(&self, req: PublishRequest) -> Result<PublishReport, AppError>;
    pub fn import_package(&self, req: ImportPackageRequest) -> Result<ImportReport, AppError>;
}
```

The facade is a convenience. Use-case modules own behavior.

## Repository port

The repository port is a real boundary because service tests use a fake and SQLite implements the same contract.

```rust
pub trait LibraryRepository {
    fn initialize_schema(&self, req: InitializeSchemaRequest) -> Result<(), RepoError>;
    fn library_identity(&self) -> Result<LibraryIdentity, RepoError>;
    fn library_summary(&self) -> Result<LibrarySummary, RepoError>;
    fn list_decks(&self) -> Result<DeckRows, RepoError>;
    fn get_deck_by_slug(&self, slug: &DeckSlug) -> Result<Deck, RepoError>;
    fn get_sentence(&self, id: &SentenceId) -> Result<Sentence, RepoError>;
    fn query_sentences(&self, query: SentenceQuery) -> Result<SentenceRows, RepoError>;

    fn create_extract_run(&self, command: CreateExtractRun) -> Result<Run, RepoError>;
    fn claim_enrich_run(&self, command: ClaimRun) -> Result<ClaimedRun, RepoError>;
    fn claim_qa_run(&self, command: ClaimRun) -> Result<ClaimedRun, RepoError>;

    fn get_run(&self, id: &RunId) -> Result<RunWithSentences, RepoError>;
    fn pending_runs(&self) -> Result<RunRows, RepoError>;
    fn record_validation_error(&self, run: &RunId, error: ValidationErrorText) -> Result<(), RepoError>;
    fn apply_run_transaction(&self, command: ApplyRunCommit) -> Result<ApplyCommitReport, RepoError>;

    fn edit_sentence(&self, command: EditSentenceCommit) -> Result<EditCommitReport, RepoError>;
    fn approve_sentences(&self, command: ApprovalCommit) -> Result<ApprovalCommitReport, RepoError>;
    fn set_audio(&self, command: SetAudioCommand) -> Result<(), RepoError>;
    fn mark_audio_stale(&self, sentence: &SentenceId) -> Result<(), RepoError>;

    fn import_package_transaction(&self, command: ImportPackageCommit) -> Result<ImportCommitReport, RepoError>;
    fn publish_snapshot(&self, query: PublishSnapshotQuery) -> Result<PublishSnapshot, RepoError>;
}
```

`ApplyRunCommit` is stage-specific internally, but one public transaction method keeps `apply` as the only commit gate.

## Workspace port

```rust
pub trait WorkspacePort {
    fn layout(&self) -> WorkspaceLayoutSnapshot;
    fn read_config(&self) -> Result<LibraryConfig, WorkspaceError>;
    fn write_config(&self, config: &LibraryConfig) -> Result<(), WorkspaceError>;

    fn init_layout(&self, req: InitLayoutRequest) -> Result<InitLayoutReport, WorkspaceError>;
    fn write_agents_contract(&self) -> Result<(), WorkspaceError>;
    fn write_example_raw(&self, profile: &ProfileId) -> Result<WorkspaceRelativePath, WorkspaceError>;

    fn write_run_files(&self, run: &Run, task: &PromptTask) -> Result<RunFileReport, WorkspaceError>;
    fn read_run_manifest(&self, target: &ApplyTarget) -> Result<RunManifest, WorkspaceError>;
    fn write_run_manifest(&self, run: &Run) -> Result<(), WorkspaceError>;
    fn repair_run_manifest(&self, run: &Run) -> Result<(), WorkspaceError>;
    fn read_reply_file(&self, path: &RunRelativePath) -> Result<ReplyBytes, WorkspaceError>;

    fn write_audio_file(&self, sentence: &SentenceId, bytes: &AudioBytes) -> Result<AudioFileWriteReport, WorkspaceError>;
    fn audio_file_state(&self, sentence: &SentenceId) -> Result<AudioFileState, WorkspaceError>;
}
```

The workspace computes audio paths from sentence IDs. It does not store or receive deck slug for internal audio paths.

## Handoff port

```rust
pub trait HandoffPort {
    fn render_task(&self, request: PromptTaskRequest) -> Result<PromptTask, HandoffError>;
    fn parse_reply(&self, stage: RunStage, bytes: &ReplyBytes) -> Result<ParsedReply, HandoffError>;
}

pub enum ParsedReply {
    Extract(ExtractReply),
    Enrich(EnrichReply),
    Qa(QaReply),
}
```

No generic JSON payload crosses into service behavior.

## Audio and publish ports

```rust
pub trait AudioSynthesizer {
    fn synthesize(&self, request: AudioSynthesisRequest) -> Result<SynthesizedAudio, AudioError>;
    fn metadata(&self) -> AudioBackendMetadata;
}

pub trait PublisherRegistry {
    fn publisher_for(&self, format: PublishFormat) -> Option<&dyn Publisher>;
}

pub trait Publisher {
    fn publish(&self, snapshot: PublishSnapshot, request: PublishArtifactRequest) -> Result<PublishArtifactReport, PublishError>;
}
```

Publishers consume a typed snapshot. They do not query `library.db` directly.

## Request examples

```rust
pub struct PrepareEnrichRequest {
    pub deck: Option<DeckSlug>,
    pub limit: SentenceLimit,
    pub mode: EnrichMode,
}

pub enum EnrichMode {
    DraftsOnly,
    ForceReenrich,
}

pub struct EditSentenceRequest {
    pub id: SentenceId,
    pub target: Option<TargetText>,
    pub romanisation: OptionalFieldEdit<Romanisation>,
    pub english: OptionalFieldEdit<NaturalEnglish>,
    pub literal: OptionalFieldEdit<LiteralGloss>,
    pub register: OptionalFieldEdit<Register>,
    pub active: ActiveChange,
    pub move_to: Option<SentencePosition>,
    pub derived_policy: DerivedFieldPolicy,
}

pub enum ActiveChange {
    Unchanged,
    SetActive,
    SetInactive,
}

pub enum DerivedFieldPolicy {
    InvalidateWhenStale,
    PreserveWithWarning,
}
```

No boolean mode arguments in service APIs.

## Approval use case

Approval is exposed as a use case so CLI and UI can share the same rules.

```rust
pub struct ApproveSentencesRequest {
    pub scope: SentenceApprovalScope,
    pub action: ApprovalAction,
}

pub enum SentenceApprovalScope {
    Sentence(SentenceId),
    Deck(DeckSlug),
    Query(SentenceQuery),
}

pub enum ApprovalAction {
    Approve,
    Unapprove,
}

pub struct ApprovalReport {
    pub approved: Vec<SentenceId>,
    pub unapproved: Vec<SentenceId>,
    pub rejected: Vec<ApprovalRejection>,
    pub terminal: TerminalDirective,
}

pub enum ApprovalRejection {
    CannotApproveDraft { id: SentenceId },
    MissingRequiredEnrichment { id: SentenceId, missing: MissingEnrichmentFields },
}
```

`approve_sentences` does not require QA. It can return warnings for unQA'd rows if the CLI wants to surface them.

## Publish request and selection

```rust
pub struct PublishRequest {
    pub scope: PublishScope,
    pub format: PublishFormat,
    pub dest: Option<OutputRelativePath>,
    pub overwrite: OverwritePolicy,
    pub selection: PublishSelectionPolicy,
    pub qa_warning: QaWarningPolicy,
}

pub enum PublishSelectionPolicy {
    Default,
    ApprovedOnly,
    IncludeUnapproved,
}

pub enum QaWarningPolicy {
    Warn,
    SuppressWarning,
}
```

Rules by format:

```text
Package/Db -> ignore selection filter and export losslessly unless caller explicitly asks for a filtered DB copy.
Study/Anki default -> approved enriched rows only.
Study/Anki include-unapproved -> all enriched rows in scope.
Draft rows are never included in Study/Anki.
```

## Import request and approval policy

```rust
pub struct ImportPackageRequest {
    pub package: PackagePath,
    pub mode: ImportMode,
    pub approval_policy: ImportApprovalPolicy,
}

pub enum ImportMode {
    DryRun,
    Commit,
}

pub enum ImportApprovalPolicy {
    Default,
    TrustPackageApproval,
}
```

Default policy:

```text
source_library_id == destination library_id
  -> preserve sentence IDs, active, and qa_checked_at for new-format package restore/sync

source_library_id != destination library_id
  -> allocate local sentence IDs for new imported rows
  -> active = false
  -> qa_checked_at = NULL
  -> store imported origin fields
```

`TrustPackageApproval` may preserve external `active` and `qa_checked_at`, but package validation must still enforce `active => enriched`.

## Terminal directive

Services return a typed directive. CLI decides exact strings.

```rust
pub enum TerminalDirective {
    Next(NextAction),
    Done(DoneReason),
    Blocked(BlockedReason),
}

pub enum NextAction {
    ApplyRun { run_id: RunId },
    ApplyOldestPending,
    ExtractExample { raw: WorkspaceRelativePath, deck: DeckSlug },
    EnrichDeck { deck: DeckSlug },
    QaDeck { deck: DeckSlug },
    ApproveDeck { deck: DeckSlug },
    Audio { deck: Option<DeckSlug> },
    Publish { deck: Option<DeckSlug>, format: PublishFormat, dest: Option<OutputRelativePath> },
    ShowSentence { id: SentenceId },
    Status,
}
```

## Error categories and exit codes

```rust
pub enum AppError {
    Validation(ValidationError),
    Environment(EnvironmentError),
    ChoiceRequired(ChoiceRequiredError),
    NotFound(NotFoundError),
    Internal(InternalError),
}

pub enum LingoExitCode {
    Success = 0,
    ValidationOrUserError = 1,
    EnvironmentOrSetupError = 2,
    ChoiceRequired = 3,
    InternalError = 4,
}
```

CLI maps `AppError` to `LingoExitCode`. Services do not call `std::process::exit`.

## Status use case sketch

```rust
pub fn status(app: &LingoServices<'_>, _req: StatusRequest) -> Result<StatusReport, AppError> {
    let config = app.deps.workspace.read_config()?;
    let profile = app.deps.profiles.get(&config.target.profile)?;
    let summary = app.deps.repo.library_summary()?;
    let decks = app.deps.repo.list_decks()?;
    let pending = app.deps.repo.pending_runs()?;
    let next = next::rank_next_action(&summary, &decks, &pending, profile);

    Ok(StatusReport {
        summary,
        decks,
        pending_runs: pending,
        terminal: next,
    })
}
```

`StatusReport` should include approval counts separately from lifecycle counts.

## Apply use case sketch

```rust
pub fn apply(app: &LingoServices<'_>, req: ApplyRequest) -> Result<ApplyReport, AppError> {
    let target = resolve_apply_target(app, req.target, req.selection)?;
    let run = app.deps.repo.get_run(&target.run_id)?;

    let manifest = app.deps.workspace.read_run_manifest(&target)?;
    if manifest.disagrees_with(&run) {
        app.deps.workspace.repair_run_manifest(&run.run)?;
    }

    let reply = app.deps.workspace.read_reply_file(run.run.reply_path())?;
    let reply_hash = ContentHash::sha256(reply.bytes());

    if let Some(already) = run.run.already_applied_decision(&reply_hash)? {
        return Ok(ApplyReport::already_applied(already));
    }

    let parsed = app.deps.handoff.parse_reply(run.run.stage(), &reply)?;
    let snapshot = app.deps.repo.validation_snapshot(&run.run.id())?;
    let preview = validate_apply(parsed, snapshot, reply_hash)?;

    if req.mode == ApplyMode::DryRun {
        return Ok(ApplyReport::dry_run(preview));
    }

    let commit = app.deps.repo.apply_run_transaction(preview.into_commit(app.deps.clock.now()))?;
    Ok(ApplyReport::applied(commit))
}
```
