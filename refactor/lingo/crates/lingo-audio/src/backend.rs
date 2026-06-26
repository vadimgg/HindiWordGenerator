use crate::error::AudioAdapterError;
use crate::model::{BackendRequest, EncodedAudio};
use lingo_domain::AudioBackendId;

pub(crate) trait AudioBackend: Send + Sync {
    fn id(&self) -> AudioBackendId;
    fn synthesize(&self, request: &BackendRequest<'_>) -> Result<EncodedAudio, AudioAdapterError>;
}
