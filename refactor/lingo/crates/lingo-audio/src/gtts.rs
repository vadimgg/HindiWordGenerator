use crate::backend::AudioBackend;
use crate::error::AudioAdapterError;
use crate::model::{BackendRequest, EncodedAudio};
use lingo_domain::{AudioBackendId, AudioFailureClass};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub(crate) struct GttsBackend {
    uv: PathBuf,
}

impl GttsBackend {
    pub fn new(uv: PathBuf) -> Self {
        Self { uv }
    }
}

impl AudioBackend for GttsBackend {
    fn id(&self) -> AudioBackendId {
        AudioBackendId::Gtts
    }

    fn synthesize(&self, request: &BackendRequest<'_>) -> Result<EncodedAudio, AudioAdapterError> {
        if request.text.trim().is_empty() {
            return Err(AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::InvalidRequest,
                "target text is empty",
            ));
        }
        let directory = tempfile::tempdir().map_err(|error| {
            AudioAdapterError::backend(self.id(), AudioFailureClass::Retryable, error.to_string())
        })?;
        let target = directory.path().join("speech.mp3");
        let output = Command::new(&self.uv)
            .args(["run", "--with", "gtts", "gtts-cli"])
            .arg(request.text)
            .arg("--lang")
            .arg(request.language.as_str())
            .arg("--output")
            .arg(&target)
            .output()
            .map_err(|error| {
                AudioAdapterError::backend(
                    self.id(),
                    if error.kind() == std::io::ErrorKind::NotFound {
                        AudioFailureClass::Configuration
                    } else {
                        AudioFailureClass::Retryable
                    },
                    error.to_string(),
                )
            })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::Retryable,
                safe_process_message(&stderr, output.status.code()),
            ));
        }
        let bytes = fs::read(&target).map_err(|error| {
            AudioAdapterError::backend(self.id(), AudioFailureClass::Retryable, error.to_string())
        })?;
        EncodedAudio::mp3(bytes).map_err(|message| {
            AudioAdapterError::backend(self.id(), AudioFailureClass::Retryable, message)
        })
    }
}

fn safe_process_message(stderr: &str, code: Option<i32>) -> String {
    let compact = stderr
        .lines()
        .take(4)
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(500)
        .collect::<String>();
    if compact.is_empty() {
        format!("gTTS process exited with status {code:?}")
    } else {
        format!("gTTS process exited with status {code:?}: {compact}")
    }
}
