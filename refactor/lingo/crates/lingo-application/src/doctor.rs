use crate::ports::{
    CapabilityCheck, CapabilityKind, ContextFailure, DeckContextProvider, EnvironmentFailure,
    EnvironmentProbe, RequiredCapabilities,
};
use lingo_domain::AudioBackendId;
use thiserror::Error;

pub struct DoctorDeps<'a> {
    pub context: &'a dyn DeckContextProvider,
    pub environment: &'a dyn EnvironmentProbe,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorReport {
    pub checks: Vec<CapabilityCheck>,
}

impl DoctorReport {
    pub fn required_checks_passed(&self) -> bool {
        self.checks
            .iter()
            .filter(|check| check.required)
            .all(|check| check.available)
    }
}

#[derive(Debug, Error)]
pub enum DoctorError {
    #[error(transparent)]
    Context(#[from] ContextFailure),
    #[error(transparent)]
    Environment(#[from] EnvironmentFailure),
}

pub fn doctor(deps: &DoctorDeps<'_>) -> Result<DoctorReport, DoctorError> {
    let context = deps.context.resolve()?;
    let mut kinds = vec![
        CapabilityKind::Workspace,
        CapabilityKind::Editor,
        CapabilityKind::Clipboard,
        CapabilityKind::Node,
    ];
    if context.audio.primary == AudioBackendId::Gtts
        || context.audio.fallback == Some(AudioBackendId::Gtts)
    {
        kinds.push(CapabilityKind::Gtts);
    }
    if context.audio.primary == AudioBackendId::ElevenLabs
        || context.audio.fallback == Some(AudioBackendId::ElevenLabs)
    {
        kinds.push(CapabilityKind::ElevenLabsKey);
    }
    kinds.sort_by_key(|kind| kind.wire_name());
    kinds.dedup();
    Ok(DoctorReport {
        checks: deps.environment.probe(&RequiredCapabilities {
            kinds,
            elevenlabs_key_env: context.audio.elevenlabs_key_env,
        })?,
    })
}
