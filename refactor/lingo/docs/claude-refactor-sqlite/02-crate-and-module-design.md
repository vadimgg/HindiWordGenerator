# 02 · Crate & module design

> The layouts below are the **target**, not a scaffolding instruction. Let files
> appear as code needs them; do not create empty modules for symmetry. The point
> is responsibilities and seams, not file count.

## `lingo-domain`

Pure sentence-library model: value objects, closed sets, invariants. No SQL,
filesystem layout, provider APIs, prompt templates, CLI output, or viewer DTOs.

```text
crates/lingo-domain/src/
  lib.rs
  ids.rs                  # CollectionId, SentenceId, WordId, RunId, ProfileId
  text.rs / language.rs   # TargetText, Romanisation, Gloss, LanguageProfile, DisplayPolicy
  audio.rs                # AudioBackendId, AudioFormat, AudioRelativePath, ContentHash, SentenceAudio
  sentence/
    mod.rs
    model.rs              # Sentence aggregate + construction/update operations
    authority.rs          # FieldAuthority, FieldAuthoritySet, SentenceField
    status.rs             # SentenceStatus, EnrichmentClaim
    provenance.rs         # SentenceProvenance (Generated | Imported)
    tags.rs               # SentenceTags
    breakdown.rs          # TokenBreakdown, BreakdownItem, coverage helpers
  word/
    mod.rs
    model.rs              # WordEntry, WordMeaning, WordOccurrence
    key.rs                # WordKey normalization (R12 surface-form identity)
  library/
    mod.rs
    collection.rs         # Collection, CollectionTitle, SectionName, SentenceOrder
    validation.rs         # aggregate validation that needs no adapters
  diagnostic.rs           # ValidationReport, DiagnosticCode, Severity
```

Key models: `CollectionId`, `SentenceId`, `WordId`, `RunId`, `ProfileId`;
`Collection`, `SectionName`, `SentenceOrder`; `Sentence`, `SentenceDraft`,
`SentenceEnrichment`, `SentenceStatus`; `FieldAuthoritySet`, `FieldAuthority`,
`SentenceField`; `SentenceProvenance`; `TokenBreakdown`, `BreakdownItem`,
`SentenceTags`; `WordEntry`, `WordKey`, `WordMeaning`, `WordOccurrence`;
`SentenceAudio` (`AudioRelativePath`, `ContentHash`, backend, format, voice/model).

Key methods: `Sentence::draft(...)`, `Sentence::apply_enrichment(...)`,
`FieldAuthoritySet::reject_human_field_changes(...)`,
`WordKey::from_surface(&TargetText)`, `TokenBreakdown::try_new(...)`,
`SentenceTags::try_from_values(...)`.

Design notes:
- Keep fields private; expose named operations (`set_section`, `attach_audio`,
  `mark_enriched`, `claim_for_enrichment`, `clear_abandoned_claim`).
- Closed sets own their stable names in one place: `SentenceStatus::wire_name()`,
  `FieldAuthority::wire_name()`, `SentenceField::wire_name()`,
  `AudioBackendId::wire_name()`.
- Use typed collections for repeating values with rules (tags, authority map,
  breakdown items, meanings, occurrences).

## `lingo-application`

Workflow orchestration and typed reports. Owns ports, not concrete storage, SQL,
prompts, provider APIs, artifacts, or presentation DTOs.

```text
crates/lingo-application/src/
  lib.rs
  ports/
    mod.rs
    context.rs            # DeckContextProvider, ProfileCatalog
    library.rs            # LibraryStore + query/mutation request models
    audio.rs              # AudioSynthesizer, AudioFileStore
    prompt.rs             # PromptEngine, prompt packets, draft DTOs from adapters
    runs.rs               # RunJournal, prepared/applied run records
    publish.rs            # PackagePublisher, AnkiExporter, PublishMaterial
    environment.rs        # EnvironmentProbe
  reports/                # next_action, counts, status (typed, presentation-neutral)
  extract/                # prepare.rs, apply.rs, accept.rs
  enrich/                 # prepare.rs, apply.rs, accept.rs
  library/                # list.rs, organize.rs, edit.rs
  words.rs
  audio.rs
  package.rs
  export.rs
  status.rs
  lang.rs  doctor.rs  init.rs
```

Cohesive library port (do not create a repository per table):

```rust
pub trait LibraryStore {
    fn summary(&self) -> Result<LibrarySummary, LibraryFailure>;
    fn list_sentences(&self, query: &SentenceQuery) -> Result<SentencePage, LibraryFailure>;
    fn get_sentence(&self, id: &SentenceId) -> Result<Sentence, LibraryFailure>;
    fn insert_drafts(&self, request: InsertDrafts) -> Result<CommitReport, LibraryFailure>;
    fn claim_for_enrichment(&self, request: ClaimEnrichment) -> Result<EnrichmentRun, LibraryFailure>;
    fn apply_enrichment(&self, request: ApplyEnrichmentCommit) -> Result<CommitReport, LibraryFailure>;
    fn reset_enrichment_claim(&self, request: ResetEnrichmentClaim) -> Result<ResetReport, LibraryFailure>;
    fn reorder(&self, request: ReorderSentences) -> Result<ReorderReport, LibraryFailure>;
    fn update_sentence(&self, request: UpdateSentence) -> Result<SentenceReport, LibraryFailure>;
    fn set_audio(&self, sentence: &SentenceId, audio: SentenceAudio) -> Result<(), LibraryFailure>;
    fn list_words(&self, query: &WordQuery) -> Result<WordPage, LibraryFailure>;
}
```

Use-case functions: `prepare_extract`, `apply_extract`, `prepare_enrich`,
`apply_enrich`, `reset_enrichment_claim`, `list_library`, `reorder_sentences`,
`update_sentence`, `delete_sentences`, `list_words`, `synthesize_audio`,
`package`, `export_anki`, `status`.

Report rules: typed and presentation-neutral; never return `serde_json::Value`;
command strings only behind a small `NextAction`/`CommandHint` type.

## `lingo-workspace-fs`

Local workspace mechanics: config/profile/run files, SQLite connection/migrations,
audio files, health/scan classification. Implements application ports.

```text
crates/lingo-workspace-fs/src/
  lib.rs  root.rs  layout.rs  atomic_file.rs
  config/    profiles/    runs/    audio_files.rs
  library/
    mod.rs
    connection.rs          # open DB, set PRAGMAs, transaction helpers
    migrations.rs          # embedded SQL migrations, user_version + library_metadata
    store.rs               # SqliteLibraryStore implements LibraryStore
    rows.rs                # private DB row structs
    codecs.rs              # JSON columns <-> domain typed values
    queries.rs             # SQL built from typed selections
    transactions.rs        # claim/apply/reorder transaction scripts
    health.rs              # DB/audio mismatch classification
    fixtures.rs            # test-only helpers
```

Key items: `WorkspaceLayout::library_db()`, `WorkspaceLayout::audio_dir()`,
`SqliteLibraryStore::open(path)`, `SqliteLibraryStore::migrate()`,
`AudioFileStore::write_sentence_audio(...) -> SentenceAudio`, `FsRunJournal`.

Design notes: SQL strings stay private to `library/`; row DTOs stay private and
convert immediately to domain values; every multi-table write is one transaction;
every connection enables `PRAGMA foreign_keys = ON`, configures WAL, and applies
migrations before use.

## `lingo-prompt`

Render prompt packets and parse model replies. Owns external prompt DTO shape and
strict parsing, not persistence.

```text
crates/lingo-prompt/src/
  lib.rs  engine.rs  templates.rs  fence.rs  error.rs
  extract_prompt.rs  extract_reply.rs  enrich_prompt.rs  enrich_reply.rs
```

`HandlebarsPromptEngine::strict()`; `render_extract`, `parse_extract_reply ->
ExtractDraft`, `render_enrich`, `parse_enrich_reply -> EnrichDraft`.

Design notes: deny unknown fields in replies; parsers return draft DTOs owned by
application ports — application converts drafts to domain values and checks
run/profile/authority invariants; the three transports (manual, file handoff,
direct API) all pass through the same parser + validation gate (see
[05](./05-cli.md), [07](./07-prompts.md)).

## `lingo-audio`

Audio backend catalog. The current catalog/fallback shape is good.

```text
crates/lingo-audio/src/
  lib.rs  backend.rs  catalog.rs  fallback.rs  model.rs  error.rs  gtts.rs  elevenlabs.rs
```

`AudioCatalog::builder()`, `add_gtts(...)`, `add_elevenlabs(...)`,
`impl AudioSynthesizer for AudioCatalog`. Keep explicit catalog + duplicate-backend
checks; no hidden self-registration; keep provider-specific request fields out of
domain.

## `lingo-artifacts`

Derived publishers. Reads typed `PublishMaterial`, produces JSON/db packages and
Anki exports.

```text
crates/lingo-artifacts/src/
  lib.rs  error.rs  staging.rs  checksum.rs  path.rs
  package/  mod.rs  json.rs  db.rs  manifest.rs  model.rs
  anki/     mod.rs  schema.rs  model.rs  exporter.rs
```

`PortablePackagePublisher`, `ApkgExporter`, `PackageFormat::{Json, Db}`. The Anki
SQLite schema is **not** the Lingo library schema — keep them in separate modules.
Manifests and DB copies are derived and verified after writing.

## `lingo-cli`

App edge, composition root, CLI parsing, terminal output, local HTTP transport,
Studio DTO mapping.

```text
crates/lingo-cli/src/
  main.rs  cli.rs  composition.rs  exit.rs  interaction.rs  secrets.rs
  output/   mod.rs  terminal.rs  json.rs
  commands/ init.rs extract.rs enrich.rs status.rs library.rs words.rs
            audio.rs package.rs export.rs config.rs lang.rs doctor.rs viewer.rs
  studio/   mod.rs  dto.rs  handlers.rs  error.rs  routes.rs
  viewer_server.rs
```

Responsibilities: `cli.rs` clap types only; `commands/*` parse args → typed
requests → use case → render output; `composition.rs` builds concrete
`SqliteLibraryStore`, `FsRunJournal`, prompt engine, audio catalog, publishers,
environment probe; `studio/dto.rs` maps typed reports to stable wire JSON (no SQL,
no business validity); `viewer_server.rs` HTTP transport + static files + route
dispatch only.

Design notes: no direct TOML mutation inside Studio handlers except through a
config use case / narrow config adapter; no raw `serde_json::Value` crossing into
application — parse JSON into request DTOs, then into typed application requests.
