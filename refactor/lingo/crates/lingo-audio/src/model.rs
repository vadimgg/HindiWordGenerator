use lingo_application::ports::AudioRequest;
use lingo_domain::{AudioBackendId, AudioFormat, LanguageCode};

pub(crate) struct BackendRequest<'a> {
    pub text: &'a str,
    pub language: &'a LanguageCode,
    pub voice: Option<&'a str>,
    pub model: Option<&'a str>,
}

impl<'a> BackendRequest<'a> {
    pub fn for_backend(request: &'a AudioRequest, backend: AudioBackendId) -> Self {
        let (voice, model) = if backend == AudioBackendId::ElevenLabs {
            (
                request.elevenlabs_voice.as_deref(),
                request.elevenlabs_model.as_deref(),
            )
        } else {
            (None, None)
        };
        Self {
            text: &request.text,
            language: &request.language,
            voice,
            model,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EncodedAudio {
    pub bytes: Vec<u8>,
    pub format: AudioFormat,
}

impl EncodedAudio {
    pub fn mp3(bytes: Vec<u8>) -> Result<Self, &'static str> {
        if bytes.is_empty() {
            return Err("provider returned empty audio");
        }
        Ok(Self {
            bytes,
            format: AudioFormat::Mp3,
        })
    }
}
