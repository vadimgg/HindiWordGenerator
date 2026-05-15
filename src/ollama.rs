use crate::config::ModelSpec;
use serde_json::json;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const OLLAMA_HOST: &str = "127.0.0.1:11434";

pub trait SentenceModelClient {
    fn check_model(&self, model: &ModelSpec) -> ModelReadiness;
    fn generate(&self, model: &ModelSpec, prompt: &str) -> Result<ModelOutput, ModelClientError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelReadiness {
    pub ready: bool,
    pub model_digest: Option<String>,
    pub message: String,
    pub recovery: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelOutput {
    pub text: String,
}

#[derive(Debug)]
pub enum ModelClientError {
    UnsupportedProvider(String),
    Http(io::Error),
    Status { status: u16, body: String },
    Json(serde_json::Error),
    MissingField(&'static str),
}

impl std::fmt::Display for ModelClientError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelClientError::UnsupportedProvider(provider) => {
                write!(formatter, "Unsupported model provider: {provider}")
            }
            ModelClientError::Http(error) => write!(formatter, "Ollama request failed: {error}"),
            ModelClientError::Status { status, body } => {
                write!(formatter, "Ollama returned HTTP {status}\n\n{body}")
            }
            ModelClientError::Json(error) => {
                write!(formatter, "Could not parse Ollama response: {error}")
            }
            ModelClientError::MissingField(field) => {
                write!(formatter, "Ollama response did not include {field:?}.")
            }
        }
    }
}

pub struct HttpOllamaClient;

impl SentenceModelClient for HttpOllamaClient {
    fn check_model(&self, model: &ModelSpec) -> ModelReadiness {
        if model.provider != "ollama" {
            return ModelReadiness {
                ready: false,
                model_digest: None,
                message: format!("Unsupported model provider: {}", model.provider),
                recovery: None,
            };
        }

        if let Err(error) = http_request("GET", "/api/version", None) {
            return ModelReadiness {
                ready: false,
                model_digest: None,
                message: format!("Ollama is not reachable at http://{OLLAMA_HOST}: {error}"),
                recovery: Some(model.ollama_run_command()),
            };
        }

        match show_model(model) {
            Ok(digest) => ModelReadiness {
                ready: true,
                model_digest: digest,
                message: format!("Ollama model is ready: {}", model.model),
                recovery: None,
            },
            Err(error) => ModelReadiness {
                ready: false,
                model_digest: None,
                message: format!("Configured Ollama model is not installed or reachable: {error}"),
                recovery: Some(model.ollama_run_command()),
            },
        }
    }

    fn generate(&self, model: &ModelSpec, prompt: &str) -> Result<ModelOutput, ModelClientError> {
        if model.provider != "ollama" {
            return Err(ModelClientError::UnsupportedProvider(
                model.provider.clone(),
            ));
        }
        let body = json!({
            "model": model.model,
            "prompt": prompt,
            "stream": false,
        })
        .to_string();
        let response = http_request("POST", "/api/generate", Some(&body))?;
        if response.status != 200 {
            return Err(ModelClientError::Status {
                status: response.status,
                body: response.body,
            });
        }
        let value: serde_json::Value =
            serde_json::from_str(&response.body).map_err(ModelClientError::Json)?;
        let text = value
            .get("response")
            .and_then(|value| value.as_str())
            .ok_or(ModelClientError::MissingField("response"))?
            .to_string();
        Ok(ModelOutput { text })
    }
}

fn show_model(model: &ModelSpec) -> Result<Option<String>, ModelClientError> {
    let body = json!({ "model": model.model }).to_string();
    let response = http_request("POST", "/api/show", Some(&body))?;
    if response.status != 200 {
        return Err(ModelClientError::Status {
            status: response.status,
            body: response.body,
        });
    }
    let value: serde_json::Value =
        serde_json::from_str(&response.body).map_err(ModelClientError::Json)?;
    Ok(value
        .get("digest")
        .and_then(|value| value.as_str())
        .map(ToString::to_string))
}

struct HttpResponse {
    status: u16,
    body: String,
}

fn http_request(
    method: &str,
    path: &str,
    body: Option<&str>,
) -> Result<HttpResponse, ModelClientError> {
    let mut stream = TcpStream::connect(OLLAMA_HOST).map_err(ModelClientError::Http)?;
    stream
        .set_read_timeout(Some(Duration::from_secs(360)))
        .map_err(ModelClientError::Http)?;
    stream
        .set_write_timeout(Some(Duration::from_secs(30)))
        .map_err(ModelClientError::Http)?;

    let body = body.unwrap_or_default();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {OLLAMA_HOST}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .map_err(ModelClientError::Http)?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(ModelClientError::Http)?;
    parse_http_response(&response)
}

fn parse_http_response(response: &str) -> Result<HttpResponse, ModelClientError> {
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| ModelClientError::Http(io::Error::other("invalid HTTP response")))?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or_else(|| ModelClientError::Http(io::Error::other("invalid HTTP status")))?;
    Ok(HttpResponse {
        status,
        body: body.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::parse_http_response;

    #[test]
    fn parses_http_response_status_and_body() {
        let response =
            parse_http_response("HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}").unwrap();

        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{}");
    }
}
