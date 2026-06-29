# 03 — Domain Model

## Aggregate relationships

```text
Library
  └── Deck
        └── Sentence
              ├── SentenceOrigin
              ├── ApprovalState
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

Generated sentence IDs use the real format:

```text
sen-<ulid>
```

Example:

```text
sen-01jx9m7q8v6f2x4k9d3p1r0t5w
```

The prefix is a kind marker only. Code must not parse deck slug, position, creation time, or any other business fact out of the suffix.

```rust
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SentenceId(String);

impl SentenceId {
    pub fn parse(raw: impl Into<String>) -> Result<Self, SentenceIdError> {
        let raw = raw.into();
        if !is_valid_sentence_id(&raw) {
            return Err(SentenceIdError::Invalid { raw });
        }
        Ok(Self(raw))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

Forbidden API:

```rust
sentence_id.deck_slug();
sentence_id.position();
sentence_id.created_at_from_ulid();
```

CLI docs may use friendlier sample IDs such as `sen-ch01-01` in examples. Those are illustrative only and must not define the generated ID contract.

Other durable IDs are also value objects:

```rust
pub struct LibraryId(String);   // lib-<random/ulid-ish opaque value>
pub struct DeckId(String);      // internal opaque id
pub struct RunId(String);       // opaque, even if human-readable
pub struct PackageId(String);   // package manifest identity
```

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

## Sentence origin

Origin is durable sentence provenance. It survives run cleanup and package round-trips.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SentenceOrigin {
    Generated(GeneratedOrigin),
    Imported(ImportedOrigin),
    Manual(ManualOrigin),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedOrigin {
    source_extract_run_id: Option<RunIdText>,
    source_label: Option<SourceLabel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedOrigin {
    source_library_id: LibraryId,
    source_package_id: PackageId,
    source_sentence_id: SourceSentenceId,
    source_label: Option<SourceLabel>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManualOrigin {
    source_label: Option<SourceLabel>,
}
```

`RunIdText` is intentionally a stored string wrapper, not necessarily a live foreign-key reference. Old run rows may be cleaned up.

Creation rules:

```text
apply extract reply -> Generated { source_extract_run_id: run id, source_label: raw path/deck source }
manual add/edit UI  -> Manual { source_label: optional }
import package      -> Imported { source_library_id, source_package_id, source_sentence_id }
```

No separate origin timestamp exists on the origin value. `Sentence.created_at` is the local entry timestamp. Package manifests carry their own generation timestamp.

## Approval state

Approval is a real gate for study-facing export.

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalState {
    Unapproved,
    Approved,
}

impl ApprovalState {
    pub fn is_approved(self) -> bool {
        matches!(self, Self::Approved)
    }
}
```

Invariant:

```text
Approved implies SentenceLifecycle::Enriched.
```

The domain should make invalid states unrepresentable through operations:

```rust
impl Sentence {
    pub fn approve(&mut self) -> Result<(), ApprovalError> {
        if self.lifecycle != SentenceLifecycle::Enriched {
            return Err(ApprovalError::CannotApproveDraft { id: self.id.clone() });
        }
        self.approval = ApprovalState::Approved;
        Ok(())
    }

    pub fn unapprove(&mut self) {
        self.approval = ApprovalState::Unapproved;
    }

    fn downgrade_to_draft(&mut self) {
        self.lifecycle = SentenceLifecycle::Draft;
        self.approval = ApprovalState::Unapproved;
    }
}
```

QA is not part of this invariant.

## Sentence aggregate

```rust
pub struct Sentence {
    id: SentenceId,
    deck_id: DeckId,
    position: SentencePosition,
    lifecycle: SentenceLifecycle,
    approval: ApprovalState,
    qa: QaState,
    origin: SentenceOrigin,
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
    pub fn approval(&self) -> ApprovalState { self.approval }
    pub fn qa_state(&self) -> &QaState { &self.qa }
    pub fn origin(&self) -> &SentenceOrigin { &self.origin }
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
    fn audio_input_key(&self, target: &TargetText) -> AudioInputText;
    fn classify_target_edit(&self, before: &TargetText, after: &TargetText) -> TargetEditImpact;
    fn derive_word_key(&self, input: WordKeyInput<'_>) -> Result<WordKey, WordKeyError>;
    fn normalize_for_audio(&self, target: &TargetText) -> AudioInputText;
}
```

The classifier rule is:

```text
SemanticChange  if target_identity_key changes
AudioOnlyChange if target identity is same but audio input key changes
NoContentChange if both are same
```

Audio fingerprints then combine profile/audio input with backend/voice/model/language.
