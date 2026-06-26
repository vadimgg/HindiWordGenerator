use crate::ports::{
    ContextFailure, DeckContextProvider, OverrideScope, ProfileCatalog, ProfileFailure,
    ProfileOverrideStore, ProfileSummary, PromptOverrideTarget, PromptStage,
};
use lingo_domain::ProfileId;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageListReport {
    pub profiles: Vec<ProfileSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageShowReport {
    pub profile: ProfileSummary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptOriginReport {
    pub profile: ProfileId,
    pub import_origin: String,
    pub build_origin: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditPromptRequest {
    pub profile: ProfileId,
    pub stage: PromptStage,
    pub scope: OverrideScope,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditPromptReport {
    pub target: PromptOverrideTarget,
}

#[derive(Debug, Error)]
pub enum LangError {
    #[error(transparent)]
    Profile(#[from] ProfileFailure),
    #[error(transparent)]
    Context(#[from] ContextFailure),
}

pub fn list_languages(catalog: &dyn ProfileCatalog) -> Result<LanguageListReport, LangError> {
    Ok(LanguageListReport {
        profiles: catalog.list()?,
    })
}

pub fn show_language(
    catalog: &dyn ProfileCatalog,
    profile: &ProfileId,
) -> Result<LanguageShowReport, LangError> {
    let definition = catalog.require(profile)?;
    Ok(LanguageShowReport {
        profile: ProfileSummary {
            id: definition.profile.id().clone(),
            language: definition.profile.language().as_str().to_string(),
            code: definition.profile.code().as_str().to_string(),
            romanisation: definition.profile.romanisation().wire_name().to_string(),
        },
    })
}

pub fn which_prompts(context: &dyn DeckContextProvider) -> Result<PromptOriginReport, LangError> {
    let context = context.resolve()?;
    Ok(PromptOriginReport {
        profile: context.profile.id().clone(),
        import_origin: prompt_origin(&context.import_prompt),
        build_origin: prompt_origin(&context.build_prompt),
    })
}

pub fn create_prompt_override(
    store: &dyn ProfileOverrideStore,
    request: EditPromptRequest,
) -> Result<EditPromptReport, LangError> {
    Ok(EditPromptReport {
        target: store.create_prompt_override(&request.profile, request.stage, request.scope)?,
    })
}

fn prompt_origin(prompt: &crate::ports::ResolvedPrompt) -> String {
    match &prompt.origin_path {
        Some(path) => format!("{} ({})", prompt.origin.wire_name(), path.display()),
        None => prompt.origin.wire_name().to_string(),
    }
}
