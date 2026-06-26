use crate::layout::WorkspaceLayout;
use crate::profiles::FsProfileCatalog;
use lingo_application::ports::{
    AudioSettings, ConfigOrigin, ContextFailure, DeckContext, LearnerContext, ProfileCatalog,
};
use lingo_domain::{AudioBackendId, DisplayLead, DisplayPolicy, ProfileId};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
struct ConfigDto {
    target: Option<TargetDto>,
    learner: Option<LearnerDto>,
    display: Option<DisplayDto>,
    audio: Option<AudioDto>,
    package: Option<PackageDto>,
    export: Option<ExportDto>,
}

#[derive(Deserialize)]
struct TargetDto {
    profile: String,
}

#[derive(Default, Deserialize, Clone)]
struct LearnerDto {
    #[serde(default)]
    native_languages: Vec<String>,
    location: Option<String>,
    goal: Option<String>,
    notes: Option<String>,
}

#[derive(Default, Deserialize, Clone)]
struct DisplayDto {
    lead: Option<String>,
    show_secondary: Option<bool>,
}

#[derive(Default, Deserialize, Clone)]
struct AudioDto {
    backend: Option<String>,
    fallback: Option<String>,
    #[serde(default)]
    gtts: GttsDto,
    #[serde(default)]
    elevenlabs: ElevenLabsDto,
}

#[derive(Default, Deserialize, Clone)]
struct GttsDto {
    lang: Option<String>,
}

#[derive(Default, Deserialize, Clone)]
struct ElevenLabsDto {
    voice: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
}

#[derive(Default, Deserialize, Clone)]
struct PackageDto {
    destination: Option<String>,
}

#[derive(Default, Deserialize, Clone)]
struct ExportDto {
    deck: Option<String>,
}

pub fn resolve_context(
    layout: &WorkspaceLayout,
    global_config_dir: &Path,
) -> Result<DeckContext, ContextFailure> {
    let global_path = global_config_dir.join("config.toml");
    let global = load_optional(&global_path)?;
    let deck = load_required(&layout.config_file())?;
    let target = deck
        .target
        .as_ref()
        .ok_or_else(|| ContextFailure::Invalid("config.toml is missing [target]".into()))?;
    let profile_id = ProfileId::parse(&target.profile)
        .map_err(|error| ContextFailure::Invalid(error.to_string()))?;
    let catalog =
        FsProfileCatalog::for_workspace(global_config_dir.to_path_buf(), layout.root().clone());
    let definition = catalog
        .require(&profile_id)
        .map_err(|error| ContextFailure::Invalid(error.to_string()))?;

    let mut origins = BTreeMap::new();
    let learner = merge_learner(global.learner, deck.learner, &mut origins);
    let display = merge_display(
        definition.profile.romanisation().is_required(),
        global.display,
        deck.display,
        &mut origins,
    )?;
    let audio = merge_audio(&definition, global.audio, deck.audio, &mut origins)?;
    let package_destination = merge_package(layout, global.package, deck.package, &mut origins);
    let export_deck = merge_export(global.export, deck.export, &mut origins);

    Ok(DeckContext {
        profile: definition.profile,
        learner,
        display,
        audio,
        package_destination,
        exports_directory: layout.exports_dir(),
        export_deck,
        import_prompt: definition.import_prompt,
        build_prompt: definition.build_prompt,
        field_origins: origins,
    })
}

fn load_optional(path: &Path) -> Result<ConfigDto, ContextFailure> {
    if !path.is_file() {
        return Ok(ConfigDto::default());
    }
    load_required(path)
}

fn load_required(path: &Path) -> Result<ConfigDto, ContextFailure> {
    let content =
        fs::read_to_string(path).map_err(|error| ContextFailure::Io(error.to_string()))?;
    toml::from_str(&content)
        .map_err(|error| ContextFailure::Invalid(format!("{}: {error}", path.display())))
}

fn merge_learner(
    global: Option<LearnerDto>,
    deck: Option<LearnerDto>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> LearnerContext {
    let global = global.unwrap_or_default();
    let deck = deck.unwrap_or_default();
    let native_languages = if deck.native_languages.is_empty() {
        origins.insert("learner.native_languages".into(), ConfigOrigin::Global);
        global.native_languages
    } else {
        origins.insert("learner.native_languages".into(), ConfigOrigin::Deck);
        deck.native_languages
    };
    LearnerContext {
        native_languages,
        location: choose("learner.location", global.location, deck.location, origins),
        goal: choose("learner.goal", global.goal, deck.goal, origins),
        notes: choose("learner.notes", global.notes, deck.notes, origins),
    }
}

fn merge_display(
    romanisation_required: bool,
    global: Option<DisplayDto>,
    deck: Option<DisplayDto>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> Result<DisplayPolicy, ContextFailure> {
    let global = global.unwrap_or_default();
    let deck = deck.unwrap_or_default();
    let lead_raw = choose("display.lead", global.lead, deck.lead, origins).unwrap_or_else(|| {
        origins.insert("display.lead".into(), ConfigOrigin::BuiltIn);
        if romanisation_required {
            "romanisation".into()
        } else {
            "target".into()
        }
    });
    let lead = match lead_raw.as_str() {
        "romanisation" => DisplayLead::Romanisation,
        "target" => DisplayLead::Target,
        other => {
            return Err(ContextFailure::Invalid(format!(
                "unknown display lead {other:?}"
            )));
        }
    };
    let show_secondary = deck
        .show_secondary
        .or(global.show_secondary)
        .unwrap_or(true);
    origins.insert(
        "display.show_secondary".into(),
        if deck.show_secondary.is_some() {
            ConfigOrigin::Deck
        } else if global.show_secondary.is_some() {
            ConfigOrigin::Global
        } else {
            ConfigOrigin::BuiltIn
        },
    );
    Ok(DisplayPolicy::new(lead, show_secondary))
}

fn merge_audio(
    definition: &lingo_application::ports::ProfileDefinition,
    global: Option<AudioDto>,
    deck: Option<AudioDto>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> Result<AudioSettings, ContextFailure> {
    let global = global.unwrap_or_default();
    let deck = deck.unwrap_or_default();
    let backend_raw = choose("audio.backend", global.backend, deck.backend, origins)
        .unwrap_or_else(|| {
            origins.insert("audio.backend".into(), ConfigOrigin::BuiltIn);
            "gtts".into()
        });
    let primary = AudioBackendId::parse(&backend_raw)
        .map_err(|error| ContextFailure::Invalid(error.to_string()))?;
    let fallback_raw = choose("audio.fallback", global.fallback, deck.fallback, origins);
    let fallback = fallback_raw
        .as_deref()
        .map(AudioBackendId::parse)
        .transpose()
        .map_err(|error| ContextFailure::Invalid(error.to_string()))?
        .filter(|backend| *backend != primary);
    let language_raw = deck
        .gtts
        .lang
        .or(global.gtts.lang)
        .unwrap_or_else(|| definition.profile.code().as_str().to_string());
    let language = lingo_domain::LanguageCode::parse(language_raw)
        .map_err(|error| ContextFailure::Invalid(error.to_string()))?;
    let voice = deck
        .elevenlabs
        .voice
        .or(global.elevenlabs.voice)
        .or_else(|| definition.default_elevenlabs_voice.clone());
    let model = deck
        .elevenlabs
        .model
        .or(global.elevenlabs.model)
        .or_else(|| definition.default_elevenlabs_model.clone());
    let key_env = deck
        .elevenlabs
        .api_key
        .or(global.elevenlabs.api_key)
        .map(parse_env_reference)
        .transpose()?;
    Ok(AudioSettings {
        primary,
        fallback,
        language,
        elevenlabs_voice: voice,
        elevenlabs_model: model,
        elevenlabs_key_env: key_env,
    })
}

fn merge_package(
    layout: &WorkspaceLayout,
    global: Option<PackageDto>,
    deck: Option<PackageDto>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> PathBuf {
    let value = choose(
        "package.destination",
        global.and_then(|value| value.destination),
        deck.and_then(|value| value.destination),
        origins,
    );
    match value {
        Some(value) => resolve_deck_path(layout, &value),
        None => {
            origins.insert("package.destination".into(), ConfigOrigin::BuiltIn);
            layout.packages_dir().join("sentences")
        }
    }
}

fn merge_export(
    global: Option<ExportDto>,
    deck: Option<ExportDto>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> Option<String> {
    choose(
        "export.deck",
        global.and_then(|value| value.deck),
        deck.and_then(|value| value.deck),
        origins,
    )
}

fn choose<T>(
    key: &str,
    global: Option<T>,
    deck: Option<T>,
    origins: &mut BTreeMap<String, ConfigOrigin>,
) -> Option<T> {
    if let Some(value) = deck {
        origins.insert(key.into(), ConfigOrigin::Deck);
        Some(value)
    } else if let Some(value) = global {
        origins.insert(key.into(), ConfigOrigin::Global);
        Some(value)
    } else {
        None
    }
}

fn parse_env_reference(value: String) -> Result<String, ContextFailure> {
    let Some(name) = value.strip_prefix("env:") else {
        return Err(ContextFailure::Invalid(
            "audio API keys must use env:VARIABLE, never a literal secret".into(),
        ));
    };
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ContextFailure::Invalid(format!(
            "invalid environment variable name {name:?}"
        )));
    }
    Ok(name.to_string())
}

fn resolve_deck_path(layout: &WorkspaceLayout, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        layout.root().join(path)
    }
}
