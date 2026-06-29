# 09 — Prompts and Reply Codecs

## Ownership

Prompt rendering and reply parsing are handoff behavior. They should either live in `lingo-service::handoff` initially or split into `lingo-handoff` once templates/codecs bring real dependency pressure.

The handoff layer owns:

- task packet shape;
- strict output contract text;
- fence extraction;
- YAML/JSON parsing;
- stage reply DTOs;
- schema/version tags for replies.

It does not own:

- database writes;
- human overwrite policy;
- terminal rendering;
- model API calls.

## Task packet

```rust
pub struct PromptTask {
    pub run_id: RunId,
    pub stage: RunStage,
    pub title: PromptTitle,
    pub body: PromptBody,
    pub expected_reply: ReplyContract,
}

pub struct ReplyContract {
    pub file_name: RunReplyName,
    pub fence: ReplyFenceKind,
    pub format_version: ReplyFormatVersion,
}
```

## Fence parsing

```rust
pub enum ReplyFenceKind {
    Json,
    Yaml,
}

pub struct ReplyFenceExtractor;

impl ReplyFenceExtractor {
    pub fn extract(bytes: &ReplyBytes, expected: ReplyFenceKind) -> Result<ReplyFence, ReplyParseError> {
        let text = bytes.as_utf8()?;
        let fences = find_code_fences(text, expected);
        match fences.len() {
            1 => Ok(fences.into_iter().next().unwrap()),
            0 => Err(ReplyParseError::MissingFence { expected }),
            _ => Err(ReplyParseError::MultipleFences { expected }),
        }
    }
}
```

This utility is mechanical. Stage codecs still own expected keys and semantic reply DTOs.

## Extract reply DTO

```rust
pub struct ExtractReply {
    pub format: ExtractReplyFormat,
    pub sentences: Vec<ExtractReplySentence>,
}

pub struct ExtractReplySentence {
    pub target: TargetText,
    pub english: Option<Authored<NaturalEnglish>>,
    pub romanisation: Option<Authored<Romanisation>>,
    pub literal: Option<Authored<LiteralGloss>>,
    pub register: Option<Authored<Register>>,
    pub tags: SentenceTags,
}

pub struct Authored<T> {
    pub value: T,
    pub authority: FieldAuthority,
}
```

Lingo assigns sentence IDs at apply time. The model does not choose canonical identity.

## Enrich reply DTO

```rust
pub struct EnrichReply {
    pub format: EnrichReplyFormat,
    pub sentences: Vec<EnrichReplySentence>,
}

pub struct EnrichReplySentence {
    pub id: SentenceId,
    pub romanisation: Option<Romanisation>,
    pub english: Option<NaturalEnglish>,
    pub literal: Option<LiteralGloss>,
    pub register: Register,
    pub breakdown: Vec<EnrichReplyToken>,
}

pub struct EnrichReplyToken {
    pub surface: TargetText,
    pub roman: Option<Romanisation>,
    pub gloss: TokenGloss,
    pub kind: Option<TokenKind>,
    pub lemma: Option<String>,
}
```

The parser accepts shape. The validator checks IDs, coverage, authority, and word-key derivation.

## QA reply DTO

```rust
pub struct QaReply {
    pub format: QaReplyFormat,
    pub corrections: Vec<QaCorrection>,
    pub clean: Vec<SentenceId>,
}

pub struct QaCorrection {
    pub id: SentenceId,
    pub fields: Vec<QaFieldCorrection>,
}

pub enum QaFieldCorrection {
    Romanisation(Romanisation),
    English(NaturalEnglish),
    Literal(LiteralGloss),
    Register(Register),
    Breakdown(Vec<EnrichReplyToken>),
}
```

A sentence can be clean without corrections. Applying QA stamps all claimed sentences checked.

## Format version owners

Each stage has one owner for format strings:

```rust
pub enum ReplyFormat {
    ExtractV1,
    EnrichV1,
    QaV1,
}

impl ReplyFormat {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::ExtractV1 => "lingo.extract/v1",
            Self::EnrichV1 => "lingo.enrich/v1",
            Self::QaV1 => "lingo.qa/v1",
        }
    }
}
```

Do not repeat format literals in validators, tests, or templates. Templates call owner methods or receive rendered strings from the owner.

## Prompt templates

Use templates for display and instructions, not truth. Reply codecs and validators define the actual accepted contract.

Built-in templates live in:

```text
crates/lingo-handoff/templates/
  extract.md.hbs
  enrich.md.hbs
  qa.md.hbs
```

Workspace overrides live in:

```text
prompts/
  extract.md.hbs
  enrich.md.hbs
  qa.md.hbs
```

`HandoffPort::render_task` checks workspace overrides first and falls back to the
built-in template. The refined templates are product behavior: add golden tests
for rendered `task.md` output so prompt regressions fail before a model ever sees
the task.

Template data should be typed:

```rust
pub struct EnrichTemplateData {
    pub run_id: RunId,
    pub language: LanguageName,
    pub profile_rules: PromptProfileRules,
    pub sentences: Vec<PromptSentenceRow>,
    pub reply_format: ReplyFormat,
}
```

No template should invent stable IDs, field names, or reply format names itself.

## Strictness policy

Reject:

- missing required top-level `format`;
- unknown format version;
- multiple fences;
- non-UTF-8 reply;
- unknown sentence IDs;
- duplicate sentence IDs in reply;
- unknown field names;
- register strings outside the enum;
- attempts to overwrite human fields;
- token coverage failures.

Warnings are for recoverable quality concerns only. Shape and authority violations are errors.
