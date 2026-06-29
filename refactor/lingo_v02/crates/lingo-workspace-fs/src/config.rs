use lingo_application::ports::{ContextFailure, DeckContext};
use lingo_domain::{
    AudioBackendId, CollectionId, CollectionTitle, DisplayLead, DisplayPolicy, LanguageCode,
    LanguageName, LanguageProfile, ProfileId, RomanisationConvention, ScriptName, TextDirection,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeckConfig {
    pub target: TargetConfig,
    pub collection: CollectionConfig,
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub package: PackageConfig,
    #[serde(default)]
    pub export: ExportConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TargetConfig { pub profile: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CollectionConfig { pub id: String, pub title: String, pub language: String }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplayConfig { pub lead: String, pub show_secondary: bool }

impl Default for DisplayConfig {
    fn default() -> Self { Self { lead: "romanisation".to_string(), show_secondary: true } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig { pub backend: String }

impl Default for AudioConfig {
    fn default() -> Self { Self { backend: "gtts".to_string() } }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageConfig { pub destination: String }
impl Default for PackageConfig { fn default() -> Self { Self { destination: "packages/sentences".to_string() } } }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportConfig { pub destination: String, pub deck: String }
impl Default for ExportConfig { fn default() -> Self { Self { destination: "exports/lingo.apkg".to_string(), deck: "Lingo::Sentences".to_string() } } }

impl DeckConfig {
    pub fn hindi_default() -> Self {
        Self {
            target: TargetConfig { profile: "hindi".to_string() },
            collection: CollectionConfig { id: "default".to_string(), title: "Lingo Sentences".to_string(), language: "hi".to_string() },
            display: DisplayConfig::default(),
            audio: AudioConfig::default(),
            package: PackageConfig::default(),
            export: ExportConfig::default(),
        }
    }
}

pub fn read_config(path: &Path) -> Result<DeckConfig, ContextFailure> {
    let text = std::fs::read_to_string(path).map_err(|error| ContextFailure::Io(error.to_string()))?;
    toml::from_str(&text).map_err(|error| ContextFailure::Invalid(error.to_string()))
}

/// Read a dotted config key (e.g. `display.lead`).
pub fn get_config_value(path: &Path, key: &str) -> Result<String, ContextFailure> {
    let config = read_config(path)?;
    let value = match key {
        "display.lead" => config.display.lead,
        "display.show_secondary" => config.display.show_secondary.to_string(),
        "audio.backend" => config.audio.backend,
        "collection.title" => config.collection.title,
        "collection.id" => config.collection.id,
        "collection.language" => config.collection.language,
        "target.profile" => config.target.profile,
        "package.destination" => config.package.destination,
        "export.deck" => config.export.deck,
        "export.destination" => config.export.destination,
        other => return Err(ContextFailure::Invalid(format!("unknown config key {other:?}"))),
    };
    Ok(value)
}

/// Set a dotted config key and write `config.toml` back.
pub fn set_config_value(path: &Path, key: &str, value: &str) -> Result<(), ContextFailure> {
    let mut config = read_config(path)?;
    match key {
        "display.lead" => config.display.lead = value.to_string(),
        "display.show_secondary" => config.display.show_secondary = parse_bool(value)?,
        "audio.backend" => config.audio.backend = value.to_string(),
        "collection.title" => config.collection.title = value.to_string(),
        "collection.id" => config.collection.id = value.to_string(),
        "collection.language" => config.collection.language = value.to_string(),
        "target.profile" => config.target.profile = value.to_string(),
        "package.destination" => config.package.destination = value.to_string(),
        "export.deck" => config.export.deck = value.to_string(),
        "export.destination" => config.export.destination = value.to_string(),
        other => return Err(ContextFailure::Invalid(format!("unknown config key {other:?}"))),
    }
    let rendered = toml::to_string_pretty(&config).map_err(|error| ContextFailure::Invalid(error.to_string()))?;
    std::fs::write(path, rendered).map_err(|error| ContextFailure::Io(error.to_string()))
}

fn parse_bool(value: &str) -> Result<bool, ContextFailure> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        other => Err(ContextFailure::Invalid(format!("expected true/false, got {other:?}"))),
    }
}

pub fn write_default(path: &Path, profile: &str) -> Result<(), ContextFailure> {
    let mut config = DeckConfig::hindi_default();
    config.target.profile = profile.to_string();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| ContextFailure::Io(error.to_string()))?;
    }
    let rendered = toml::to_string_pretty(&config).map_err(|error| ContextFailure::Invalid(error.to_string()))?;
    std::fs::write(path, rendered).map_err(|error| ContextFailure::Io(error.to_string()))
}

pub fn context_from_config(root: &Path, config: DeckConfig) -> Result<DeckContext, ContextFailure> {
    let profile = match config.target.profile.as_str() {
        "hindi" => LanguageProfile::new(
            ProfileId::parse("hindi").map_err(map)? ,
            LanguageName::parse("Hindi").map_err(map)?,
            LanguageCode::parse("hi").map_err(map)?,
            ScriptName::parse("Devanagari").map_err(map)?,
            TextDirection::Ltr,
            RomanisationConvention::IastTilde,
        ),
        other => return Err(ContextFailure::Invalid(format!("unsupported profile {other:?}"))),
    };
    let lead = match config.display.lead.as_str() {
        "romanisation" => DisplayLead::Romanisation,
        "target" => DisplayLead::Target,
        other => return Err(ContextFailure::Invalid(format!("unknown display lead {other:?}"))),
    };
    let backend = AudioBackendId::parse(&config.audio.backend).map_err(map)?;
    Ok(DeckContext {
        profile,
        display: DisplayPolicy::new(lead, config.display.show_secondary),
        default_collection: CollectionId::parse(config.collection.id).map_err(map)?,
        default_title: CollectionTitle::parse(config.collection.title).map_err(map)?,
        package_destination: absolutize(root, &config.package.destination),
        exports_directory: absolutize(root, &config.export.destination),
        audio_backend: backend,
        root: root.to_path_buf(),
    })
}

fn absolutize(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() { path } else { root.join(path) }
}

fn map(error: impl std::fmt::Display) -> ContextFailure { ContextFailure::Invalid(error.to_string()) }
