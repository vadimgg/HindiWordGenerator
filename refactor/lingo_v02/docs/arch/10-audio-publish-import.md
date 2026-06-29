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

| Format | Scope default | Missing audio | QA gate | Round-trip |
|---|---|---|---|---|
| package | deck or library | include as `null` | none | yes |
| db | deck or library | include metadata | none | no/import not needed |
| study | whole library | skip/report | warn | no |
| anki | deck | skip/report | warn | no |

## Publish snapshot

The service builds a snapshot from the repository, then passes it to a publisher. Publishers do not query the authoring DB.

```rust
pub struct PublishSnapshot {
    pub generated_at: UtcTimestamp,
    pub library: LibraryInfo,
    pub decks: Vec<PublishDeck>,
    pub sentences: Vec<PublishSentence>,
    pub tokens: Vec<PublishToken>,
    pub audio: Vec<PublishAudio>,
    pub warnings: Vec<PublishWarning>,
}
```

## Package writer

Package is lossless and round-trippable.

```text
out/ch01/
  manifest.json
  sentences/
    sen-ch01-01.json
  audio/
    sen-ch01-01.mp3
  README.txt
```

Package audio can also be flat by sentence ID. The manifest maps sentence to optional audio path.

```rust
pub struct PackageManifest {
    pub format: PackageFormatVersion,
    pub generated_at: UtcTimestamp,
    pub language: LanguageCode,
    pub decks: Vec<PackageDeckManifest>,
    pub counts: PackageCounts,
    pub integrity: PackageIntegrity,
}
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

Study export can organize audio however the app prefers. That path is export-local, not authoring truth.

## Anki writer

Anki note GUID is derived only from sentence ID.

```rust
pub fn anki_note_guid(sentence: &SentenceId) -> NoteGuid {
    NoteGuid::from_stable_hash(sentence.as_str())
}
```

A deck rename should update deck placement/name on export, not duplicate cards.

## Import

`lingo import` reads package format, not study or Anki.

Import rules:

- package may contain one or many decks;
- preserve deck slugs unless collision requires deduping the deck slug;
- dedupe sentences by profile-normalized target identity within destination deck;
- preserve source package ID as provenance only, not current identity;
- copy audio to `audio/<new-sentence-id>.mp3`;
- compute fresh audio fingerprint for copied audio from imported metadata and current profile;
- `--dry-run` reports additions, duplicates, and conflicts.

## Import dry-run report

```rust
pub struct ImportPreview {
    pub package: PackageName,
    pub decks: Vec<ImportDeckPreview>,
    pub conflicts: Vec<ImportConflict>,
}

pub enum ImportConflict {
    SameTargetDifferentEnglish {
        existing: SentenceId,
        incoming_source_id: Option<String>,
    },
    UnsupportedPackageVersion { found: String },
}
```
