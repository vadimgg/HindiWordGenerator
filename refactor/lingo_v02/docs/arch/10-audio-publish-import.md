# 10 — Audio, Publish, and Import

## Audio backend catalog

Use an explicit catalog. No hidden self-registration.

```rust
pub fn builtin_audio_backends() -> AudioBackendCatalog {
    AudioBackendCatalog::builder()
        .add(AudioBackendMetadata::gtts())
        .build()
        .expect("built-in audio backend ids are unique")
}
```

Backend metadata lives with the backend owner:

```rust
pub struct AudioBackendMetadata {
    pub id: AudioBackendId,
    pub label: &'static str,
    pub networked: bool,
    pub requires_credentials: bool,
    pub implemented: bool,
}

impl AudioBackendMetadata {
    pub fn gtts() -> Self {
        Self {
            id: AudioBackendId::Gtts,
            label: "gTTS via Google Translate TTS",
            networked: true,
            requires_credentials: false,
            implemented: true,
        }
    }
}
```

`gtts` is networked, not local. Do not expose future backends in `--help` until implemented.

## Audio request

```rust
pub enum AudioGenerateMode {
    MissingOrStale,
    MissingOnly,
    Force,
}

pub struct AudioGenerateRequest {
    pub deck: Option<DeckSlug>,
    pub mode: AudioGenerateMode,
    pub backend: Option<AudioBackendId>,
    pub voice: Option<AudioVoice>,
    pub model: Option<AudioModel>,
}
```

Default mode is `MissingOrStale`.

## Audio fingerprint

Fingerprint includes exact TTS input and all choices that could change the audio.

```rust
pub struct AudioFingerprintInput<'a> {
    pub target: &'a TargetText,
    pub profile: &'a dyn LanguageProfile,
    pub backend: AudioBackendId,
    pub voice: Option<&'a AudioVoice>,
    pub model: Option<&'a AudioModel>,
}
```

If any component changes, audio is stale.

## Audio file write

```rust
pub fn write_sentence_audio(
    layout: &WorkspaceLayout,
    sentence: &SentenceId,
    audio: &SynthesizedAudio,
) -> Result<AudioFileWriteReport, WorkspaceError> {
    let path = layout.audio_path(sentence);
    atomic_write_verified(path.resolved(), audio.bytes())?;
    Ok(AudioFileWriteReport {
        sentence: sentence.clone(),
        file_sha256: ContentHash::sha256(audio.bytes()),
        relative_path: AudioPathPolicy::relative_path(sentence),
    })
}
```

The DB stores file hash and fingerprint. It does not store the deterministic path.

## Audio state report

```rust
pub enum AudioState {
    Missing,
    PresentFresh { path: AudioRelativePath },
    PresentStale { path: AudioRelativePath },
    Broken { path: AudioRelativePath, reason: AudioBrokenReason },
}
```

`doctor` should distinguish missing, stale, and broken rather than collapsing everything into one error.

## Publish formats

```rust
pub enum PublishFormat {
    Package,
    Study,
    Anki,
    Db,
}
```

| Format | Scope default | Selection default | Missing audio | QA gate | Round-trip |
|---|---|---|---|---|---|
| package | deck or library | all rows | include as `null` | none | yes |
| db | deck or library | all rows unless caller filters | include metadata | none | raw copy |
| study | whole library | approved enriched rows | skip/report | warn | no |
| anki | deck | approved enriched rows | skip/report | warn | no |

`--include-unapproved` may include enriched inactive rows for `study`/`anki`; draft rows remain excluded because they are not studyable.

## Publish snapshot

The service builds a snapshot from the repository, then passes it to a publisher. Publishers do not query the authoring DB.

```rust
pub struct PublishSnapshot {
    pub generated_at: UtcTimestamp,
    pub library: LibraryInfo,
    pub source_library_id: LibraryId,
    pub decks: Vec<PublishDeck>,
    pub sentences: Vec<PublishSentence>,
    pub tokens: Vec<PublishToken>,
    pub audio: Vec<PublishAudio>,
    pub warnings: Vec<PublishWarning>,
}

pub struct PublishSentence {
    pub id: SentenceId,
    pub deck: DeckId,
    pub lifecycle: SentenceLifecycle,
    pub approval: ApprovalState,
    pub qa: QaState,
    pub origin: SentenceOrigin,
    pub text: SentenceText,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
}
```

Study/Anki snapshots should already be filtered by the service. Package/db snapshots are lossless by default.

## Package writer

Package is lossless and round-trippable.

```text
out/ch01/
  manifest.json
  sentences/
    sen-01jx9m7q8v6f2x4k9d3p1r0t5w.json
  audio/
    sen-01jx9m7q8v6f2x4k9d3p1r0t5w.mp3
  README.txt
```

Package audio can be flat by sentence ID. The manifest maps sentence to optional audio path.

```rust
pub struct PackageManifest {
    pub format: PackageFormatVersion,
    pub package_id: PackageId,
    pub source_library_id: LibraryId,
    pub generated_at: UtcTimestamp,
    pub language: LanguageCode,
    pub profile_id: ProfileId,
    pub decks: Vec<PackageDeckManifest>,
    pub counts: PackageCounts,
    pub integrity: PackageIntegrity,
}
```

New-format package sentence JSON must preserve every field needed for
backup/restore:

```rust
pub struct PackageSentence {
    pub format: PackageSentenceFormat,
    pub id: SentenceId,
    pub deck_slug: DeckSlug,
    pub position: SentencePosition,
    pub lifecycle: SentenceLifecycle,
    pub approval: ApprovalState,
    pub qa_checked_at: Option<UtcTimestamp>,
    pub origin: SentenceOrigin,
    pub text: SentenceText,
    pub authority: FieldAuthoritySet,
    pub tags: SentenceTags,
    pub tokens: SentenceTokenBreakdown,
    pub audio: Option<PackageAudio>,
    pub created_at: UtcTimestamp,
    pub updated_at: UtcTimestamp,
}
```

Required round-trip fields:

```text
id
status/lifecycle
active/approval
qa_checked_at
origin and source fields
field authority
tokens/breakdown
tags
audio metadata and optional audio file
created_at/updated_at
```

Every artifact the user trusts should be read back and verified.

## Study writer

Study export is app-facing and decoupled from authoring schema.

```sql
CREATE TABLE study_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
) STRICT;

CREATE TABLE decks (
  slug TEXT PRIMARY KEY,
  title TEXT,
  subtitle TEXT,
  position INTEGER NOT NULL
) STRICT;

CREATE TABLE sentences (
  id TEXT PRIMARY KEY,
  deck_slug TEXT NOT NULL REFERENCES decks(slug),
  position INTEGER NOT NULL,
  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register TEXT,
  audio TEXT NOT NULL
) STRICT;

CREATE TABLE words (
  key TEXT PRIMARY KEY,
  form TEXT NOT NULL,
  roman TEXT
) STRICT;

CREATE TABLE word_sentences (
  word_key TEXT NOT NULL REFERENCES words(key),
  sentence_id TEXT NOT NULL REFERENCES sentences(id),
  position INTEGER NOT NULL,
  surface TEXT NOT NULL,
  roman TEXT,
  gloss TEXT NOT NULL,
  PRIMARY KEY(word_key, sentence_id, position)
) STRICT;
```

Default selection for study export is active approved rows only. The export may include a manifest warning count for unQA'd selected rows, but QA does not block.

Study export can organize audio however the app prefers. That path is export-local, not authoring truth.

## Anki writer

Anki note GUID is derived only from sentence ID.

```rust
pub fn anki_note_guid(sentence: &SentenceId) -> NoteGuid {
    NoteGuid::from_stable_hash(sentence.as_str())
}
```

Default selection for Anki export is active approved rows only. A deck rename should update deck placement/name on export, not duplicate cards.

## Import

`lingo import` reads package format, not study or Anki.

Import rules:

- package may contain one or many decks;
- preserve deck slugs unless collision requires deduping the deck slug;
- dedupe sentences by profile-normalized target identity within destination deck;
- write durable imported origin fields on every new imported sentence;
- copy audio to `audio/<new-sentence-id>.mp3`;
- compute fresh audio fingerprint for copied audio from imported metadata and current profile;
- `--dry-run` reports additions, duplicates, conflicts, and approval/QA policy effects.

## Import approval policy

Default policy is safe re-approval for external content:

| Source | Sentence IDs | `active` | `qa_checked_at` | Origin |
|---|---|---|---|---|
| same `source_library_id` as destination | preserve | preserve | preserve | preserve/new-format restore |
| different or missing `source_library_id` | allocate local IDs | `false` | `NULL` | imported source fields |
| external with explicit trust | allocate local IDs | preserve if enriched | preserve | imported source fields |

A true disaster restore from a new-format package into an empty workspace should
be a restore workflow that seeds the destination `meta.library_id` from the
package before importing. Plain cross-library import should not silently bless
imported approvals.

## Import dry-run report

```rust
pub struct ImportPreview {
    pub package: PackageName,
    pub source_library_id: Option<LibraryId>,
    pub approval_policy: ImportApprovalPolicy,
    pub decks: Vec<ImportDeckPreview>,
    pub conflicts: Vec<ImportConflict>,
    pub approval_effects: ImportApprovalEffects,
}

pub enum ImportConflict {
    SameTargetDifferentEnglish {
        existing: SentenceId,
        incoming_source_id: SourceSentenceId,
    },
    UnsupportedPackageVersion { found: String },
    ActiveDraftInPackage { source_sentence_id: SourceSentenceId },
}

pub struct ImportApprovalEffects {
    pub approvals_preserved: usize,
    pub approvals_reset: usize,
    pub qa_preserved: usize,
    pub qa_reset: usize,
}
```

Package validation rejects invalid package states such as `active=true` on a draft sentence.
