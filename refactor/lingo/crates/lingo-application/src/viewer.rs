use crate::ports::{ContextFailure, DeckContextProvider, WorkspaceFailure, WorkspaceStore};
use lingo_domain::{BatchId, CardId, DisplayLead};
use thiserror::Error;

pub struct ViewerDeps<'a> {
    pub workspace: &'a dyn WorkspaceStore,
    pub context: &'a dyn DeckContextProvider,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ViewerRequest {
    pub batch: Option<BatchId>,
    pub lead: Option<DisplayLead>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewerCard {
    pub id: CardId,
    pub primary: String,
    pub secondary: Option<String>,
    pub english: String,
    pub literal: String,
    pub register: String,
    pub audio_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ViewerPlan {
    pub cards: Vec<ViewerCard>,
    pub lead: DisplayLead,
}

#[derive(Debug, Error)]
pub enum ViewerError {
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
}

pub fn prepare_viewer(
    deps: &ViewerDeps<'_>,
    request: ViewerRequest,
) -> Result<ViewerPlan, ViewerError> {
    let context = deps.context.resolve()?;
    let lead = request.lead.unwrap_or(context.display.lead());
    let batches = match request.batch {
        Some(batch) => vec![batch],
        None => deps.workspace.list_card_batches()?,
    };
    let mut viewer_cards = Vec::new();
    for batch in batches {
        let cards = deps.workspace.load_cards(&batch)?;
        for card in cards.cards() {
            let target = card.target().as_str().to_string();
            let romanisation = card.romanisation().map(|value| value.as_str().to_string());
            let (primary, secondary) = match lead {
                DisplayLead::Romanisation => (
                    romanisation.clone().unwrap_or_else(|| target.clone()),
                    context.display.show_secondary().then_some(target),
                ),
                DisplayLead::Target => (
                    target,
                    context
                        .display
                        .show_secondary()
                        .then_some(romanisation)
                        .flatten(),
                ),
            };
            viewer_cards.push(ViewerCard {
                id: card.id().clone(),
                primary,
                secondary,
                english: card.english().as_str().to_string(),
                literal: card.literal().as_str().to_string(),
                register: card.register().wire_name().to_string(),
                audio_url: card
                    .audio()
                    .map(|audio| format!("/{}", audio.relative_path().as_str())),
            });
        }
    }
    Ok(ViewerPlan {
        cards: viewer_cards,
        lead,
    })
}
