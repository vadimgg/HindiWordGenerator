use crate::ports::{
    BootstrapChange, ProfileCatalog, ProfileFailure, WorkspaceBootstrap, WorkspaceFailure,
};
use crate::report::NextAction;
use lingo_domain::ProfileId;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitRequest {
    pub target: PathBuf,
    pub profile: ProfileId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitReport {
    pub root: PathBuf,
    pub profile: ProfileId,
    pub changes: Vec<BootstrapChange>,
    pub next: NextAction,
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error(transparent)]
    Profile(#[from] ProfileFailure),
    #[error(transparent)]
    Workspace(#[from] WorkspaceFailure),
}

pub fn init(
    workspace: &dyn WorkspaceBootstrap,
    profiles: &dyn ProfileCatalog,
    request: InitRequest,
) -> Result<InitReport, InitError> {
    let profile = profiles.require(&request.profile)?;
    let changes = workspace.create_missing(&request.target, &profile)?;
    Ok(InitReport {
        root: changes.root,
        profile: profile.profile.id().clone(),
        changes: changes.entries,
        next: NextAction::Import { raw: None },
    })
}
