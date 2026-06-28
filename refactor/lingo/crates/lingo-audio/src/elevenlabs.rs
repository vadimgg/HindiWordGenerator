use crate::backend::AudioBackend;
use crate::error::AudioAdapterError;
use crate::model::{BackendRequest, EncodedAudio};
use lingo_domain::{AudioBackendId, AudioFailureClass};
use reqwest::StatusCode;
use reqwest::blocking::Client;
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

const MAX_AUDIO_BYTES: usize = 24 * 1024 * 1024;

pub(crate) struct ElevenLabsBackend {
    client: Client,
    api_key: SecretString,
    endpoint: String,
}

impl std::fmt::Debug for ElevenLabsBackend {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ElevenLabsBackend")
            .field("api_key", &"[REDACTED]")
            .field("endpoint", &self.endpoint)
            .finish_non_exhaustive()
    }
}

impl ElevenLabsBackend {
    pub fn new(client: Client, api_key: SecretString) -> Self {
        Self {
            client,
            api_key,
            endpoint: "https://api.elevenlabs.io/v1/text-to-speech".to_string(),
        }
    }
}

impl AudioBackend for ElevenLabsBackend {
    fn id(&self) -> AudioBackendId {
        AudioBackendId::ElevenLabs
    }

    fn synthesize(&self, request: &BackendRequest<'_>) -> Result<EncodedAudio, AudioAdapterError> {
        let voice = request.voice.ok_or_else(|| {
            AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::Configuration,
                "ElevenLabs voice is not configured",
            )
        })?;
        let model = request.model.ok_or_else(|| {
            AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::Configuration,
                "ElevenLabs model is not configured",
            )
        })?;
        let endpoint = format!("{}/{voice}?output_format=mp3_44100_128", self.endpoint);
        let response = self
            .client
            .post(endpoint)
            .header("xi-api-key", self.api_key.expose_secret())
            .header("accept", "audio/mpeg")
            .json(&ElevenLabsRequest {
                text: request.text,
                model_id: model,
            })
            .send()
            .map_err(|error| {
                AudioAdapterError::backend(
                    self.id(),
                    AudioFailureClass::Retryable,
                    error.to_string(),
                )
            })?;
        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .unwrap_or_else(|error| format!("failed to read error response: {error}"));
            return Err(AudioAdapterError::backend(
                self.id(),
                classify_status(status),
                format!(
                    "ElevenLabs returned HTTP {}: {}",
                    status.as_u16(),
                    humanize_error_body(&body)
                ),
            ));
        }
        if response
            .content_length()
            .is_some_and(|length| length > MAX_AUDIO_BYTES as u64)
        {
            return Err(AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::InvalidRequest,
                "ElevenLabs response exceeds the audio size limit",
            ));
        }
        let bytes = response.bytes().map_err(|error| {
            AudioAdapterError::backend(self.id(), AudioFailureClass::Retryable, error.to_string())
        })?;
        if bytes.len() > MAX_AUDIO_BYTES {
            return Err(AudioAdapterError::backend(
                self.id(),
                AudioFailureClass::InvalidRequest,
                "ElevenLabs response exceeds the audio size limit",
            ));
        }
        EncodedAudio::mp3(bytes.to_vec()).map_err(|message| {
            AudioAdapterError::backend(self.id(), AudioFailureClass::Retryable, message)
        })
    }
}

#[derive(Serialize)]
struct ElevenLabsRequest<'a> {
    text: &'a str,
    model_id: &'a str,
}

fn classify_status(status: StatusCode) -> AudioFailureClass {
    match status.as_u16() {
        401 | 403 => AudioFailureClass::Configuration,
        429 | 500..=599 => AudioFailureClass::Retryable,
        _ => AudioFailureClass::InvalidRequest,
    }
}

fn truncate_error_body(body: &str) -> String {
    const LIMIT: usize = 500;
    let body = body.trim();
    if body.len() <= LIMIT {
        return body.to_string();
    }
    format!("{}...", &body[..LIMIT])
}

/// Pull the human-readable message out of an ElevenLabs error body. Their API
/// nests it under `detail.message` (e.g. "Free users cannot use library voices
/// via the API"); `detail` may also be a plain string or an array of field
/// errors. Falls back to the raw, truncated body when nothing matches.
fn humanize_error_body(body: &str) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(detail) = value.get("detail") {
            if let Some(message) = detail.get("message").and_then(|m| m.as_str()) {
                return message.trim().to_string();
            }
            if let Some(text) = detail.as_str() {
                return text.trim().to_string();
            }
            if let Some(items) = detail.as_array() {
                let joined = items
                    .iter()
                    .filter_map(|item| item.get("msg").and_then(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join("; ");
                if !joined.is_empty() {
                    return joined;
                }
            }
        }
        if let Some(message) = value.get("message").and_then(|m| m.as_str()) {
            return message.trim().to_string();
        }
    }
    truncate_error_body(body)
}

#[cfg(test)]
mod tests {
    use super::{classify_status, humanize_error_body, truncate_error_body};
    use lingo_domain::AudioFailureClass;
    use reqwest::StatusCode;

    #[test]
    fn classifies_provider_statuses() {
        assert_eq!(
            classify_status(StatusCode::UNAUTHORIZED),
            AudioFailureClass::Configuration
        );
        assert_eq!(
            classify_status(StatusCode::TOO_MANY_REQUESTS),
            AudioFailureClass::Retryable
        );
        assert_eq!(
            classify_status(StatusCode::BAD_REQUEST),
            AudioFailureClass::InvalidRequest
        );
    }

    #[test]
    fn truncates_long_provider_errors() {
        let body = "x".repeat(600);
        let truncated = truncate_error_body(&body);
        assert!(truncated.len() < body.len());
        assert!(truncated.ends_with("..."));
    }

    #[test]
    fn extracts_human_message_from_provider_error() {
        let body = r#"{"detail":{"type":"payment_required","code":"paid_plan_required","message":"Free users cannot use library voices via the API. Please upgrade your subscription to use this voice.","status":"payment_required"}}"#;
        assert_eq!(
            humanize_error_body(body),
            "Free users cannot use library voices via the API. Please upgrade your subscription to use this voice."
        );
    }

    #[test]
    fn falls_back_to_raw_body_when_unstructured() {
        assert_eq!(humanize_error_body("upstream timeout"), "upstream timeout");
    }
}
