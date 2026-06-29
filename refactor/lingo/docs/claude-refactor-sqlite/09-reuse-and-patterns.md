# 09 · Reuse & patterns

## 1. Existing code to reuse

**Domain value-object mechanics** — reuse the parsing style of `BatchId`, `RunId`,
`ProfileId`, `CardId` for new `CollectionId`, `SentenceId`, `WordId`; reuse
`TargetText`, `Romanisation`, `Gloss`, `LanguageCode`, `LanguageProfile`,
`DisplayPolicy`, `AudioBackendId`, `AudioFormat`, `AudioRelativePath`,
`ContentHash`, `Register`, `WordKind`, `GrammarTag`, and the
`source_fingerprint`/`content_hash` mechanics. Reuse the
`ValidationReport`/`Diagnostic`/`DiagnosticCode`/`Severity` style for
domain-owned diagnostics. Keep new values domain-specific even if they share
parsing mechanics — no global bag of IDs/tags.

**Application use-case structure** — the current pattern is clean and testable;
apply it to extract/enrich/library/words/audio/package/export/status:

```rust
pub struct UseCaseDeps<'a> { /* ports */ }
pub struct Request { /* typed fields */ }
pub struct Report  { /* typed facts, next action */ }
pub enum Error     { /* transparent port errors + domain errors */ }
pub fn use_case(deps: &UseCaseDeps<'_>, request: Request) -> Result<Report, Error>;
```

**Prompt parsing & run journaling** — reuse strict reply DTO parsing with
`deny_unknown_fields`, `strip_one_optional_fence`, stage-specific prompt/reply
files, and `RunJournal`/`PreparedRun` stage/profile checks. Rename stages
`Import`/`Build` → `Extract`/`Enrich`.

**Audio catalog** — reuse the explicit `AudioCatalogBuilder`, duplicate-backend
detection, retryable fallback, and the gTTS/ElevenLabs backends.

**Artifact staging & path safety** — reuse `publish_directory`/`publish_file`
staging/swap, `ArtifactPath` safety, checksum helpers, the APKG stable-ID
helpers, and the manifest-verification pattern. Change package generation to read
from library `PublishMaterial` rather than old `CardBatch` files.

**Workspace infrastructure** — reuse `WorkspaceRoot` discovery, `WorkspaceLayout`
(add `library_db()`), atomic file helpers, config/profile merge + prompt override
resolution, and `FsRunJournal`.

## 2. Code to retire or contain

- **Old `WorkspaceStore` batch contract** (raw/source/card batches) → replaced by
  `LibraryStore`. Keep only for one-time migration if real data must be preserved.
- **`SourceBatch` / `CardBatch` as runtime aggregates** → useful as references for
  extracting logic, but the runtime aggregate is `Sentence` + derived word/audio
  metadata.
- **Studio handlers doing routing + DTO parsing + app calls + output shaping +
  direct config/file writes** → split into `viewer_server.rs` (transport only),
  `studio/handlers.rs` (orchestration), `studio/dto.rs` (wire mapping),
  `studio/error.rs` (HTTP mapping), with typed use cases in `lingo-application`.
- **The prototype `import-package`** → replace with a package-import use case that
  validates and commits through `LibraryStore` (doc 08). The prototype
  `sentences/*.json` layer → becomes `library.db` rows.

## 3. Patterns to introduce

1. **Port + adapter for the library store.** One cohesive `LibraryStore` port;
   `SqliteLibraryStore` for production, `InMemoryLibraryStore` for tests. Passes
   the deletion test: removing the port would push SQL and transaction mechanics
   into application use cases.
2. **Transaction script / unit of work for multi-table writes** — insert drafts;
   claim enrichment; apply enrichment + rebuild word projections; reorder; attach
   audio metadata after file write. No ORM-like repository per table; the
   transaction *is* the business persistence operation.
3. **Typed selectors instead of SQL strings:**

   ```rust
   SentenceQuery { selection: SentenceSelection, sort: SentenceSort, page: PageRequest }
   SentenceSelection::{ All, Collection(..), Section{..}, Status(..), MissingAudio, ReadyToPublish, Ids(..) }
   ```
   `lingo-workspace-fs` converts selectors to SQL; CLI/Studio never pass SQL.
4. **DTO mapper at the app edge:** `JSON → Studio DTO → typed request → use case →
   typed report → Studio DTO → JSON`. Keep CLI output formatting separate from
   application report construction.
5. **Explicit catalogs** for audio backends, built-in profiles, and package
   output formats. No hidden self-registration, inventory/linker magic, plugin
   scanning, or event buses.
6. **Semantic codecs for stable JSON columns and manifests** (`authority`,
   `breakdown`, `tags`, `provenance`, manifest keys) — local codecs with owner
   vocabulary, not raw string keys scattered through behavior.
7. **Named collections** (`DraftSentences`, `EnrichedSentences`, `SentenceTags`,
   `FieldAuthoritySet`, `WordMeanings`, `SentenceOccurrences`, `PublishSelection`)
   exposing iterators and safe operations, not public `Vec`/`BTreeMap`.

## 4. Patterns to avoid

- Event bus for workflow stages (these are simple commands + transaction scripts).
- Generic `serde_json::Value` inside application/domain code.
- Global `utils`/`common` modules for domain vocabulary.
- Repositories for every SQL table before a real reuse boundary exists.
- Keeping source YAML, card JSON, per-sentence JSON, and SQLite all writable as
  equivalent truths.
- A new crate for every responsibility just because the diagram looks symmetric.
- Compatibility shims for prototype data unless real user data must be protected.
