use crate::project::ProjectRoot;
use std::fs;
use std::io;
use std::path::PathBuf;

const CONFIG_PATH: &str = "hindi.toml";
const DEFAULT_SENTENCE_MODEL: &str = "ollama:translategemma:12b";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub sentence_generation_model: ModelSpec,
    pub sentence_package_destination: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSpec {
    pub original: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug)]
pub enum ConfigError {
    Io { path: PathBuf, source: io::Error },
    InvalidModel(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io { path, source } => {
                write!(formatter, "Could not read {}\n\n{source}", path.display())
            }
            ConfigError::InvalidModel(value) => write!(
                formatter,
                "Invalid sentence generation model {value:?}\n\nUse provider:model, for example ollama:translategemma:12b."
            ),
        }
    }
}

pub fn load_config(root: &ProjectRoot) -> Result<AppConfig, ConfigError> {
    let path = root.join(CONFIG_PATH);
    if !path.exists() {
        return Ok(default_config());
    }
    let content = fs::read_to_string(&path).map_err(|source| ConfigError::Io { path, source })?;
    let model =
        parse_sentence_model(&content).unwrap_or_else(|| DEFAULT_SENTENCE_MODEL.to_string());
    Ok(AppConfig {
        sentence_generation_model: ModelSpec::parse(&model)?,
        sentence_package_destination: parse_package_destination(&content).map(PathBuf::from),
    })
}

fn default_config() -> AppConfig {
    AppConfig {
        sentence_generation_model: ModelSpec::parse(DEFAULT_SENTENCE_MODEL)
            .expect("default sentence generation model is valid"),
        sentence_package_destination: None,
    }
}

fn parse_sentence_model(content: &str) -> Option<String> {
    parse_string_key(content, "models", "sentence_generation")
}

fn parse_package_destination(content: &str) -> Option<String> {
    parse_string_key(content, "package", "sentences_destination")
}

fn parse_string_key(content: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_section = line == format!("[{section}]");
            continue;
        }
        if !in_section {
            continue;
        }
        let Some(value) = line.strip_prefix(key) else {
            continue;
        };
        let Some((_, value)) = value.split_once('=') else {
            continue;
        };
        return Some(unquote(value.trim()).to_string());
    }
    None
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}

impl ModelSpec {
    pub fn parse(value: &str) -> Result<Self, ConfigError> {
        let Some((provider, model)) = value.split_once(':') else {
            return Err(ConfigError::InvalidModel(value.to_string()));
        };
        if provider.trim().is_empty() || model.trim().is_empty() {
            return Err(ConfigError::InvalidModel(value.to_string()));
        }
        Ok(Self {
            original: value.to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
        })
    }

    pub fn ollama_run_command(&self) -> String {
        format!("ollama run {}", self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::{default_config, parse_package_destination, parse_sentence_model, ModelSpec};

    #[test]
    fn parses_sentence_generation_model_from_models_section() {
        let model = parse_sentence_model(
            r#"
            [models]
            sentence_generation = "ollama:gemma4:latest"
            "#,
        );

        assert_eq!(model.as_deref(), Some("ollama:gemma4:latest"));
    }

    #[test]
    fn parses_package_destination_from_package_section() {
        let destination = parse_package_destination(
            r#"
            [models]
            sentence_generation = "ollama:translategemma:12b"

            [package]
            sentences_destination = "/tmp/hindi-package"
            "#,
        );

        assert_eq!(destination.as_deref(), Some("/tmp/hindi-package"));
    }

    #[test]
    fn default_config_has_no_package_destination() {
        let config = default_config();

        assert!(config.sentence_package_destination.is_none());
    }

    #[test]
    fn parses_provider_model_spec() {
        let spec = ModelSpec::parse("ollama:translategemma:12b").unwrap();

        assert_eq!(spec.provider, "ollama");
        assert_eq!(spec.model, "translategemma:12b");
    }
}
