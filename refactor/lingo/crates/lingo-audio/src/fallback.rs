use crate::backend::AudioBackend;
use crate::error::AudioAdapterError;
use crate::model::{BackendRequest, EncodedAudio};
use lingo_domain::{AudioBackendId, AudioFailureClass};

pub(crate) fn synthesize_with_fallback(
    primary: &dyn AudioBackend,
    fallback: Option<&dyn AudioBackend>,
    request: &BackendRequest<'_>,
) -> Result<(AudioBackendId, EncodedAudio), AudioAdapterError> {
    match primary.synthesize(request) {
        Ok(audio) => Ok((primary.id(), audio)),
        Err(error) if error.class() == AudioFailureClass::Retryable => {
            let Some(fallback) = fallback else {
                return Err(error);
            };
            if fallback.id() == primary.id() {
                return Err(AudioAdapterError::DuplicateBackend(primary.id()));
            }
            fallback
                .synthesize(request)
                .map(|audio| (fallback.id(), audio))
        }
        Err(error) => Err(error),
    }
}
