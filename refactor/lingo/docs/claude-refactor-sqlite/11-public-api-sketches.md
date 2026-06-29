# 11 · Public API sketches

These show the target seams and naming style, not complete implementations.

## Domain: sentence & authority

```rust
pub struct Sentence {
    id: SentenceId,
    collection: CollectionId,
    section: Option<SectionName>,
    order: SentenceOrder,
    target: TargetText,
    romanisation: Option<Romanisation>,
    english: Option<Gloss>,
    literal: Option<Gloss>,
    register: Option<Register>,
    authority: FieldAuthoritySet,
    status: SentenceStatus,
    provenance: SentenceProvenance,
    tags: SentenceTags,
    breakdown: Option<TokenBreakdown>,
    audio: Option<SentenceAudio>,
}

impl Sentence {
    pub fn draft(input: DraftSentenceInput) -> Result<Self, SentenceError>;
    pub fn apply_enrichment(&mut self, enrichment: SentenceEnrichment) -> Result<(), SentenceError>;
    pub fn edit_field(&mut self, field: SentenceField, value: SentenceFieldValue) -> Result<(), SentenceError>;
    pub fn set_section(&mut self, section: Option<SectionName>);
    pub fn set_order(&mut self, order: SentenceOrder);
    pub fn attach_audio(&mut self, audio: SentenceAudio) -> Result<(), SentenceError>;
}
```

```rust
pub enum SentenceField { Target, Romanisation, English, Literal, Register, Breakdown }
impl SentenceField { pub const fn wire_name(self) -> &'static str; }

pub enum FieldAuthority { Human, Ai }

pub struct FieldAuthoritySet { by_field: BTreeMap<SentenceField, FieldAuthority> }
impl FieldAuthoritySet {
    pub fn empty() -> Self;
    pub fn mark(&mut self, field: SentenceField, authority: FieldAuthority);
    pub fn authority(&self, field: SentenceField) -> Option<FieldAuthority>;
    // enforced again at apply time: any human field in `before` must be byte-identical in `proposed`
    pub fn reject_human_field_changes(&self, before: &Sentence, proposed: &SentenceEnrichment) -> Result<(), AuthorityError>;
}

pub enum SentenceStatus { Draft, Enriching, Enriched }   // "ready" is derived (doc 03 §4)
impl SentenceStatus { pub const fn wire_name(self) -> &'static str; }
```

## Application port: library store

```rust
pub trait LibraryStore {
    fn summary(&self) -> Result<LibrarySummary, LibraryFailure>;
    fn list_sentences(&self, query: &SentenceQuery) -> Result<SentencePage, LibraryFailure>;
    fn get_sentence(&self, id: &SentenceId) -> Result<Sentence, LibraryFailure>;

    fn insert_drafts(&self, request: InsertDrafts) -> Result<InsertDraftsReport, LibraryFailure>;

    fn claim_for_enrichment(&self, request: ClaimEnrichment) -> Result<EnrichmentRun, LibraryFailure>;
    fn apply_enrichment(&self, request: ApplyEnrichmentCommit) -> Result<EnrichmentCommitReport, LibraryFailure>;
    fn reset_enrichment_claim(&self, request: ResetEnrichmentClaim) -> Result<ResetReport, LibraryFailure>;

    fn reorder(&self, request: ReorderSentences) -> Result<ReorderReport, LibraryFailure>;
    fn update_sentence(&self, request: UpdateSentence) -> Result<SentenceReport, LibraryFailure>;
    fn set_audio(&self, sentence: &SentenceId, audio: SentenceAudio) -> Result<(), LibraryFailure>; // upserts sentence_audio
    fn list_words(&self, query: &WordQuery) -> Result<WordPage, LibraryFailure>;
}
```

## Application use cases

```rust
pub struct ExtractDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
    pub runs: &'a dyn RunJournal,
    pub raw: &'a dyn RawDocumentStore,
}
pub fn prepare_extract(deps: &ExtractDeps<'_>, request: PrepareExtract) -> Result<ExtractPreparation, ExtractError>;
pub fn apply_extract(deps: &ExtractDeps<'_>, request: ApplyExtract) -> Result<ExtractReport, ExtractError>;

pub struct EnrichDeps<'a> {
    pub library: &'a dyn LibraryStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
    pub runs: &'a dyn RunJournal,
}
pub fn prepare_enrich(deps: &EnrichDeps<'_>, request: PrepareEnrich) -> Result<EnrichPreparation, EnrichError>;
pub fn apply_enrich(deps: &EnrichDeps<'_>, request: ApplyEnrich) -> Result<EnrichReport, EnrichError>;
```

## Query types (typed selectors, never SQL across the boundary)

```rust
pub struct SentenceQuery { pub selection: SentenceSelection, pub sort: SentenceSort, pub page: PageRequest }

pub enum SentenceSelection {
    All,
    Collection(CollectionId),
    Section { collection: CollectionId, section: SectionName },
    Status(SentenceStatus),
    MissingAudio,
    ReadyToPublish,          // derived: Enriched AND has audio
    Ids(SentenceIds),
}
pub enum SentenceSort { LibraryOrder, UpdatedDesc }
```

The CLI may parse `--filter status=draft` or `--section "Chapter 02"`, but the
application receives `SentenceSelection`, never a raw SQL predicate.

## Prompt port

```rust
pub trait PromptEngine {
    fn render_extract(&self, request: &ExtractPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_extract_reply(&self, reply: &str) -> Result<ExtractDraft, PromptFailure>;
    fn render_enrich(&self, request: &EnrichPromptRequest) -> Result<PromptPacket, PromptFailure>;
    fn parse_enrich_reply(&self, reply: &str) -> Result<EnrichDraft, PromptFailure>;
}
```

The adapter parses syntax and reply DTO shape. The application validates workflow
invariants (run ids, exact sentence ids, profile, authority preservation).

## Studio DTO mapping (app edge)

```rust
// lingo-cli/src/studio/dto.rs
pub struct SentenceDto {
    pub id: String, pub section: Option<String>, pub order: i64,
    pub target: String, pub romanisation: Option<String>, pub english: Option<String>,
    pub literal: Option<String>, pub status: String, pub audio: bool,
}

impl From<&SentenceReportRow> for SentenceDto {
    fn from(row: &SentenceReportRow) -> Self {
        Self {
            id: row.id().as_str().to_string(),
            section: row.section().map(|s| s.as_str().to_string()),
            order: row.order().get(),
            target: row.target().as_str().to_string(),
            romanisation: row.romanisation().map(|v| v.as_str().to_string()),
            english: row.english().map(|v| v.as_str().to_string()),
            literal: row.literal().map(|v| v.as_str().to_string()),
            status: row.status().wire_name().to_string(),
            audio: row.audio().is_some(),
        }
    }
}
```

DTO mapping may create strings, sourced from typed owners.

## SQLite adapter transaction sketch

```rust
impl SqliteLibraryStore {
    fn with_transaction<T>(&self, f: impl FnOnce(&rusqlite::Transaction<'_>) -> Result<T, LibraryFailure>)
        -> Result<T, LibraryFailure>
    {
        let mut connection = self.connection()?;          // foreign_keys=ON, WAL, migrated
        let tx = connection.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

impl LibraryStore for SqliteLibraryStore {
    fn claim_for_enrichment(&self, request: ClaimEnrichment) -> Result<EnrichmentRun, LibraryFailure> {
        self.with_transaction(|tx| {
            let rows = select_claimable_sentences(tx, &request.selection, request.limit)?; // status='draft'
            mark_rows_enriching(tx, &rows, &request.run_id)?;                                // -> 'enriching' + run id
            load_enrichment_run(tx, &request.run_id)
        })
    }
}
```

SQL helpers stay private to the adapter; the public contract stays typed.
