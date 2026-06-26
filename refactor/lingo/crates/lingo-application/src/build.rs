use crate::ports::{
    BuildPromptRequest, CardBatchDraft, CardDraft, ContextFailure, DeckContextProvider,
    PreparedRun, PromptEngine, PromptFailure, PromptStage, RunFailure, RunJournal, RunRecord,
    StoredFile, TokenDraft, WordDraft, WorkspaceFailure, WorkspaceStore,
};
use crate::report::NextAction;
use lingo_domain::{
    BatchId, Card, CardBatch, CardId, CardTags, CardToken, Gloss, GrammarTag, ProfileId, Register,
    Romanisation, RunId, SourceBatch, SourceItemId, SourceRef, TargetText, ValidationReport, Word,
    WordId, WordKind, check_card_batch,
};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub struct BuildDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
    pub prompts: &'a dyn PromptEngine,
    pub runs: &'a dyn RunJournal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrepareBuild {
    pub run_id: RunId,
    pub batch: BatchId,
    pub journal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildPreparation {
    pub run: Option<RunRecord>,
    pub packet: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplyBuild {
    pub run_id: RunId,
    pub reply: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BuildReport {
    pub batch: BatchId,
    pub cards: usize,
    pub stored: StoredFile,
    pub validation: ValidationReport,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum BuildError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Prompt(#[from] PromptFailure),
    #[error(transparent)]
    Run(#[from] RunFailure),
    #[error("prepared run is for {actual:?}, not build")]
    WrongStage { actual: PromptStage },
    #[error("prepared run profile {prepared} does not match current profile {current}")]
    ProfileChanged {
        prepared: ProfileId,
        current: ProfileId,
    },
    #[error("build reply is invalid: {0}")]
    InvalidDraft(String),
    #[error("build reply failed deterministic validation with {0} error(s)")]
    ValidationFailed(usize),
}

pub fn prepare_build(
    deps: &BuildDeps<'_>,
    request: PrepareBuild,
) -> Result<BuildPreparation, BuildError> {
    let source = deps.workspace.load_source(&request.batch)?;
    let context = deps.context.resolve()?;
    let packet = deps.prompts.render_build(&BuildPromptRequest {
        source: source.clone(),
        context: context.clone(),
    })?;
    let prepared = PreparedRun {
        id: request.run_id,
        stage: PromptStage::Build,
        batch: request.batch,
        profile: context.profile.id().clone(),
        title: source.title().clone(),
        subtitle: source.subtitle().cloned(),
        packet: packet.clone(),
    };
    let run = request
        .journal
        .then(|| deps.runs.record_prepared(&prepared))
        .transpose()?;
    Ok(BuildPreparation {
        run,
        packet: packet.content,
    })
}

pub fn apply_build(deps: &BuildDeps<'_>, request: ApplyBuild) -> Result<BuildReport, BuildError> {
    let prepared = deps.runs.require_prepared(&request.run_id)?;
    if prepared.stage != PromptStage::Build {
        return Err(BuildError::WrongStage {
            actual: prepared.stage,
        });
    }
    let context = deps.context.resolve()?;
    if prepared.profile != context.profile.id().clone() {
        return Err(BuildError::ProfileChanged {
            prepared: prepared.profile,
            current: context.profile.id().clone(),
        });
    }
    let source = deps.workspace.load_source(&prepared.batch)?;
    let draft = deps.prompts.parse_build_reply(&request.reply)?;
    let cards = accept_draft(draft, &source)?;
    let validation = check_card_batch(&cards, &source, &context.profile);
    if !validation.is_clean() {
        return Err(BuildError::ValidationFailed(validation.error_count()));
    }
    let stored = deps.workspace.create_cards(&cards)?;
    deps.runs
        .record_applied(&request.run_id, &request.reply, &stored)?;
    Ok(BuildReport {
        batch: cards.batch_id().clone(),
        cards: cards.cards().len(),
        stored,
        validation,
        next: NextAction::Check {
            batch: Some(cards.batch_id().clone()),
        },
    })
}

fn accept_draft(draft: CardBatchDraft, source: &SourceBatch) -> Result<CardBatch, BuildError> {
    if draft.cards.is_empty() {
        return Err(BuildError::InvalidDraft(
            "reply contains no cards".to_string(),
        ));
    }
    let source_by_id = source
        .items()
        .iter()
        .map(|item| (item.id().clone(), item))
        .collect::<BTreeMap<_, _>>();
    let mut seen = BTreeSet::new();
    let mut cards = Vec::with_capacity(draft.cards.len());
    for draft_card in draft.cards {
        let item_id = SourceItemId::parse(&draft_card.source_item).map_err(invalid)?;
        if !seen.insert(item_id.clone()) {
            return Err(BuildError::InvalidDraft(format!(
                "duplicate card for source item {item_id}"
            )));
        }
        let Some(source_item) = source_by_id.get(&item_id) else {
            return Err(BuildError::InvalidDraft(format!(
                "unknown source item {item_id}"
            )));
        };
        cards.push(accept_card(draft_card, source, source_item)?);
    }
    for item in source.items() {
        if !seen.contains(item.id()) {
            return Err(BuildError::InvalidDraft(format!(
                "reply omitted source item {}",
                item.id()
            )));
        }
    }
    CardBatch::try_new(
        source.batch_id().clone(),
        source.title().clone(),
        source.subtitle().cloned(),
        cards,
    )
    .map_err(invalid)
}

fn accept_card(
    draft: CardDraft,
    source: &SourceBatch,
    source_item: &lingo_domain::SourceItem,
) -> Result<Card, BuildError> {
    let literal = Gloss::parse(draft.literal).map_err(invalid)?;
    let register = Register::parse(&draft.register).map_err(invalid)?;
    let words = draft
        .words
        .into_iter()
        .map(accept_word)
        .collect::<Result<Vec<_>, _>>()?;
    let tokens = draft
        .tokens
        .into_iter()
        .map(accept_token)
        .collect::<Result<Vec<_>, _>>()?;
    let tags = CardTags::try_from_values(draft.tags).map_err(invalid)?;
    let card_id = CardId::new(source.batch_id().clone(), source_item.id().clone());
    Card::try_new(
        card_id,
        source_item.target().clone(),
        source_item.romanisation().cloned(),
        source_item.english().clone(),
        literal,
        register,
        tokens,
        words,
        tags,
        SourceRef::new(
            source.batch_id().clone(),
            source_item.id().clone(),
            source_item.fingerprint().clone(),
        ),
    )
    .map_err(invalid)
}

fn accept_word(draft: WordDraft) -> Result<Word, BuildError> {
    let grammar = draft
        .grammar
        .iter()
        .map(|tag| GrammarTag::parse(tag))
        .collect::<Result<Vec<_>, _>>()
        .map_err(invalid)?;
    Ok(Word::new(
        WordId::parse(draft.id).map_err(invalid)?,
        TargetText::parse(draft.target).map_err(invalid)?,
        draft
            .romanisation
            .map(Romanisation::parse)
            .transpose()
            .map_err(invalid)?,
        Gloss::parse(draft.meaning).map_err(invalid)?,
        WordKind::parse(&draft.kind).map_err(invalid)?,
        grammar,
    ))
}

fn accept_token(draft: TokenDraft) -> Result<CardToken, BuildError> {
    Ok(CardToken::new(
        TargetText::parse(draft.target).map_err(invalid)?,
        draft
            .romanisation
            .map(Romanisation::parse)
            .transpose()
            .map_err(invalid)?,
        WordId::parse(draft.word_id).map_err(invalid)?,
    ))
}

fn invalid(error: impl std::fmt::Display) -> BuildError {
    BuildError::InvalidDraft(error.to_string())
}
