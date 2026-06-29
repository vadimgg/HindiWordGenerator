# 03 — Domain Model

## Aggregate relationships

```text
Library
  └── Deck
        └── Sentence
              ├── FieldAuthoritySet
              ├── SentenceTokenBreakdown
              ├── QaState
              └── AudioAsset metadata

Run
  ├── RunStage
  ├── RunStatus
  └── RunSentenceClaims / participation rows
```

The domain model expresses invariants. It does not know SQL rows, files, terminal output, or JSON reply formats.

## Opaque IDs

IDs validate shape but expose no semantic parsing.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SentenceId(String);

impl SentenceId {
    pub fn parse(raw: impl Into<String>) -> Result<Self, SentenceIdError> {
        let raw = raw.into();
        if !is_valid_public_id(&raw, "sen-") {
            return Err(SentenceIdError::Invalid { raw });
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Do not add methods that infer deck slug, stage, sequence, or date from an ID.

## Slug and profile values

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeckSlug(String);

impl DeckSlug {
    pub fn parse(raw: impl Into<String>) -> Result<Self, DeckSlugError> {
        let raw = raw.into();
        let valid = !raw.is_empty()
            && raw.len() <= 80
            && raw.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !valid {
            return Err(DeckSlugError::Invalid { raw });
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProfileId(String);
```

## Closed sets own wire names

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SentenceLifecycle {
    Draft,
    Enriched,
}

struct SentenceLifecycleMeta {
    wire: &'static str,
    display: &'static str,
}

impl SentenceLifecycle {
    pub const fn wire_name(self) -> &'static str {
        self.meta().wire
    }

    pub const fn display_label(self) -> &'static str {
        self.meta().display
    }

    const fn meta(self) -> SentenceLifecycleMeta {
        match self {
            Self::Draft => SentenceLifecycleMeta { wire: "draft", display: "draft" },
            Self::Enriched => SentenceLifecycleMeta { wire: "enriched", display: "enriched" },
        }
    }
}
```

`enriching` is not in this enum. It is a visible status computed from pending run claims.

```rust
pub enum VisibleSentenceStatus {
    Draft,
    Enriching { run_id: RunId },
    Enriched,
}
```

## Sentence aggregate

```rust
pub struct Sentence {
    id: SentenceId,
    deck_id: DeckId,
    position: SentencePosition,
    lifecycle: SentenceLifecycle,
    active: ActiveFlag,
    qa: QaState,
    text: SentenceText,
    authority: FieldAuthoritySet,
    tags: SentenceTags,
    tokens: SentenceTokenBreakdown,
    audio: Option<AudioAsset>,
    created_at: UtcTimestamp,
    updated_at: UtcTimestamp,
}

impl Sentence {
    pub fn lifecycle(&self) -> SentenceLifecycle { self.lifecycle }
    pub fn active(&self) -> ActiveFlag { self.active }
    pub fn qa_state(&self) -> &QaState { &self.qa }
    pub fn target(&self) -> &TargetText { self.text.target() }
}
```

Fields are private. Mutations happen through named operations.

## Sentence text

```rust
pub struct SentenceText {
    target: TargetText,
    romanisation: Option<Romanisation>,
    english: Option<NaturalEnglish>,
    literal: Option<LiteralGloss>,
    register: Option<Register>,
}
```

`Register` is a closed set:

```rust
pub enum Register {
    Informal,
    Standard,
    Formal,
}
```

## Field authority

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SentenceField {
    Target,
    Romanisation,
    English,
    Literal,
    Register,
    Breakdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldAuthority {
    Human,
    Ai,
}

pub struct FieldAuthoritySet {
    by_field: BTreeMap<SentenceField, FieldAuthority>,
}

impl FieldAuthoritySet {
    pub fn authority(&self, field: SentenceField) -> FieldAuthority {
        self.by_field.get(&field).copied().unwrap_or(FieldAuthority::Ai)
    }

    pub fn mark_human(&mut self, field: SentenceField) {
        self.by_field.insert(field, FieldAuthority::Human);
    }

    pub fn may_replace(&self, field: SentenceField) -> bool {
        self.authority(field) != FieldAuthority::Human
    }
}
```

`apply` uses this set to reject model attempts to overwrite human fields.

## Token breakdown

Tokens are canonical rows, not a JSON blob.

```rust
pub struct SentenceToken {
    position: TokenPosition,
    surface: TargetText,
    roman: Option<Romanisation>,
    gloss: TokenGloss,
    kind: Option<TokenKind>,
    word_key: WordKey,
}

pub struct SentenceTokenBreakdown {
    tokens: Vec<SentenceToken>,
}

impl SentenceTokenBreakdown {
    pub fn try_new(tokens: Vec<SentenceToken>) -> Result<Self, TokenBreakdownError> {
        if tokens.is_empty() {
            return Err(TokenBreakdownError::Empty);
        }
        reject_duplicate_token_positions(&tokens)?;
        Ok(Self { tokens })
    }

    pub fn empty() -> Self {
        Self { tokens: Vec::new() }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SentenceToken> {
        self.tokens.iter()
    }
}
```

## Word key derivation

Word keys are profile-owned normalization, not string cleanup scattered through code.

```rust
pub struct WordKey(String);

pub struct WordKeyInput<'a> {
    pub surface: &'a TargetText,
    pub lemma: Option<&'a str>,
    pub romanisation: Option<&'a Romanisation>,
    pub token_kind: Option<&'a TokenKind>,
    pub profile: &'a dyn LanguageProfile,
}

impl WordKey {
    pub fn derive(input: WordKeyInput<'_>) -> Result<Self, WordKeyError> {
        input.profile.derive_word_key(input)
    }
}
```

## Language profile

```rust
pub trait LanguageProfile {
    fn id(&self) -> &ProfileId;
    fn language_code(&self) -> &LanguageCode;
    fn audio_language_code(&self) -> &LanguageCode;
    fn romanisation_convention(&self) -> &RomanisationConvention;

    fn target_identity_key(&self, target: &TargetText) -> TargetIdentityKey;
    fn classify_target_edit(&self, before: &TargetText, after: &TargetText) -> TargetEditImpact;
    fn derive_word_key(&self, input: WordKeyInput<'_>) -> Result<WordKey, WordKeyError>;
    fn normalize_for_audio(&self, target: &TargetText) -> AudioInputText;
}
```

Profiles are explicit built-ins. Do not add plugin loading.

## Target edit impact

```rust
pub enum TargetEditImpact {
    NoContentChange,
    AudioOnlyChange,
    SemanticChange,
}

pub enum DerivedFieldPolicy {
    InvalidateWhenStale,
    PreserveWithWarning,
}

pub struct TargetEditReport {
    pub impact: TargetEditImpact,
    pub invalidated_fields: Vec<SentenceField>,
    pub preserved_human_fields: Vec<SentenceField>,
    pub audio_marked_stale: bool,
    pub lifecycle_changed: bool,
}
```

## Editing sentence text

```rust
impl Sentence {
    pub fn edit_target(
        &mut self,
        new_target: TargetText,
        profile: &dyn LanguageProfile,
        policy: DerivedFieldPolicy,
        now: UtcTimestamp,
    ) -> TargetEditReport {
        let impact = profile.classify_target_edit(self.text.target(), &new_target);
        self.text.set_target(new_target);
        self.authority.mark_human(SentenceField::Target);
        self.updated_at = now;

        match (impact, policy) {
            (TargetEditImpact::NoContentChange, _) => TargetEditReport::no_content_change(),
            (TargetEditImpact::AudioOnlyChange, _) => {
                self.mark_audio_stale();
                TargetEditReport::audio_only()
            }
            (TargetEditImpact::SemanticChange, DerivedFieldPolicy::PreserveWithWarning) => {
                self.mark_audio_stale();
                self.qa = QaState::Unchecked;
                TargetEditReport::preserved_with_warning()
            }
            (TargetEditImpact::SemanticChange, DerivedFieldPolicy::InvalidateWhenStale) => {
                self.invalidate_ai_derived_fields();
                self.tokens = SentenceTokenBreakdown::empty();
                self.qa = QaState::Unchecked;
                self.lifecycle = SentenceLifecycle::Draft;
                self.mark_audio_stale();
                TargetEditReport::semantic_invalidation()
            }
        }
    }
}
```

`active` is intentionally untouched.

## Run aggregate

```rust
pub struct Run {
    id: RunId,
    stage: RunStage,
    status: RunStatus,
    deck_id: Option<DeckId>,
    task_path: RunRelativePath,
    reply_path: RunRelativePath,
    reply_sha256: Option<ContentHash>,
    created_at: UtcTimestamp,
    applied_at: Option<UtcTimestamp>,
    reset_at: Option<UtcTimestamp>,
    abandoned_at: Option<UtcTimestamp>,
    last_validation_error: Option<ValidationErrorText>,
}

pub enum RunStage { Extract, Enrich, Qa }
pub enum RunStatus { Pending, Applied, Reset, Abandoned }
```

A pending run with `last_validation_error` can be displayed as `failed` in CLI, but the durable status remains `pending` so it is retryable.

## Run sentence participation

```rust
pub struct RunSentenceClaim {
    run_id: RunId,
    sentence_id: SentenceId,
    position: RunSentencePosition,
}
```

`run_sentences` is used both for pending claims and post-apply provenance.

## Audio asset

```rust
pub struct AudioAsset {
    file_sha256: ContentHash,
    input_fingerprint: AudioInputFingerprint,
    backend: AudioBackendId,
    profile_id: ProfileId,
    language: LanguageCode,
    voice: Option<AudioVoice>,
    model: Option<AudioModel>,
    generated_at: UtcTimestamp,
}

pub struct AudioInputFingerprint(ContentHash);

pub struct AudioFingerprintInput<'a> {
    pub target: &'a TargetText,
    pub profile: &'a dyn LanguageProfile,
    pub backend: AudioBackendId,
    pub voice: Option<&'a AudioVoice>,
    pub model: Option<&'a AudioModel>,
}

impl AudioInputFingerprint {
    pub fn derive(input: AudioFingerprintInput<'_>) -> Self {
        let audio_text = input.profile.normalize_for_audio(input.target);
        let canonical = CanonicalFingerprint::new()
            .field("profile", input.profile.id().as_str())
            .field("lang", input.profile.audio_language_code().as_str())
            .field("backend", input.backend.wire_name())
            .optional_field("voice", input.voice.map(AudioVoice::as_str))
            .optional_field("model", input.model.map(AudioModel::as_str))
            .field("text", audio_text.as_str())
            .finish();
        Self(ContentHash::sha256(canonical.as_bytes()))
    }
}
```

## Deck aggregate

```rust
pub struct Deck {
    id: DeckId,
    slug: DeckSlug,
    title: Option<DeckTitle>,
    subtitle: Option<DeckSubtitle>,
    source_path: Option<WorkspaceRelativePath>,
    position: DeckPosition,
    created_at: UtcTimestamp,
    updated_at: UtcTimestamp,
}
```

Changing `slug` changes display and future commands only. It does not change sentence IDs, run IDs, audio filenames, or study/Anki identity.
