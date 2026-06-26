use crate::atomic_file::create_atomic;
use crate::root::WorkspaceRoot;
use lingo_application::ports::{
    ConfigOrigin, OverrideScope, ProfileCatalog as ProfileCatalogPort, ProfileDefinition,
    ProfileFailure, ProfileOverrideStore, ProfileSummary, PromptOverrideTarget, PromptStage,
    ResolvedPrompt,
};
use lingo_domain::{
    LanguageCode, LanguageName, LanguageProfile, ProfileId, RomanisationConvention, ScriptName,
    TextDirection,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const HINDI_PROFILE: &str = include_str!("../assets/profiles/hindi/profile.toml");
const HINDI_IMPORT: &str = include_str!("../assets/profiles/hindi/prompts/import.md.hbs");
const HINDI_BUILD: &str = include_str!("../assets/profiles/hindi/prompts/build.md.hbs");

#[derive(Clone, Copy)]
struct BuiltInProfile {
    id: &'static str,
    profile: &'static str,
    import_prompt: &'static str,
    build_prompt: &'static str,
}

const BUILT_INS: &[BuiltInProfile] = &[BuiltInProfile {
    id: "hindi",
    profile: HINDI_PROFILE,
    import_prompt: HINDI_IMPORT,
    build_prompt: HINDI_BUILD,
}];

#[derive(Clone, Debug)]
pub struct FsProfileCatalog {
    global_config_dir: PathBuf,
    deck_root: Option<WorkspaceRoot>,
}

impl FsProfileCatalog {
    pub fn global(global_config_dir: PathBuf) -> Self {
        Self {
            global_config_dir,
            deck_root: None,
        }
    }

    pub fn for_workspace(global_config_dir: PathBuf, deck_root: WorkspaceRoot) -> Self {
        Self {
            global_config_dir,
            deck_root: Some(deck_root),
        }
    }

    pub fn global_config_dir(&self) -> &Path {
        &self.global_config_dir
    }

    fn built_in(&self, id: &ProfileId) -> Option<BuiltInProfile> {
        BUILT_INS
            .iter()
            .copied()
            .find(|profile| profile.id == id.as_str())
    }

    fn global_profile_dir(&self, id: &ProfileId) -> PathBuf {
        self.global_config_dir.join("profiles").join(id.as_str())
    }

    fn deck_prompt_path(&self, stage: PromptStage) -> Option<PathBuf> {
        self.deck_root.as_ref().map(|root| {
            root.join("prompts")
                .join(format!("{}.md.hbs", stage.wire_name()))
        })
    }
}

impl ProfileCatalogPort for FsProfileCatalog {
    fn list(&self) -> Result<Vec<ProfileSummary>, ProfileFailure> {
        validate_builtin_ids()?;
        let mut ids = BUILT_INS
            .iter()
            .map(|profile| profile.id.to_string())
            .collect::<BTreeSet<_>>();
        let profiles_dir = self.global_config_dir.join("profiles");
        if profiles_dir.is_dir() {
            for entry in fs::read_dir(&profiles_dir).map_err(profile_io)? {
                let entry = entry.map_err(profile_io)?;
                if entry.path().join("profile.toml").is_file() {
                    if let Some(name) = entry.file_name().to_str() {
                        ids.insert(name.to_string());
                    }
                }
            }
        }
        ids.into_iter()
            .map(|raw| {
                let id = ProfileId::parse(raw).map_err(profile_invalid)?;
                let definition = self.require(&id)?;
                Ok(summary(&definition))
            })
            .collect()
    }

    fn require(&self, profile: &ProfileId) -> Result<ProfileDefinition, ProfileFailure> {
        validate_builtin_ids()?;
        let built_in = self.built_in(profile);
        let global_dir = self.global_profile_dir(profile);
        let global_profile_path = global_dir.join("profile.toml");
        let profile_text = if global_profile_path.is_file() {
            fs::read_to_string(&global_profile_path).map_err(profile_io)?
        } else if let Some(built_in) = built_in {
            built_in.profile.to_string()
        } else {
            return Err(ProfileFailure::Unknown(profile.clone()));
        };
        let deck_profile_path = self
            .deck_root
            .as_ref()
            .map(|root| root.join("profile.toml"));
        let effective_profile_text = match deck_profile_path.as_ref().filter(|path| path.is_file())
        {
            Some(path) => fs::read_to_string(path).map_err(profile_io)?,
            None => profile_text,
        };
        let parsed = parse_profile(profile, &effective_profile_text)?;

        let import_prompt = resolve_prompt(
            built_in.map(|profile| profile.import_prompt),
            global_dir.join("prompts/import.md.hbs"),
            self.deck_prompt_path(PromptStage::Import),
        )?;
        let build_prompt = resolve_prompt(
            built_in.map(|profile| profile.build_prompt),
            global_dir.join("prompts/build.md.hbs"),
            self.deck_prompt_path(PromptStage::Build),
        )?;
        let field_origins = BTreeMap::from([
            ("target.profile".to_string(), ConfigOrigin::Profile),
            ("prompts.import".to_string(), import_prompt.origin),
            ("prompts.build".to_string(), build_prompt.origin),
        ]);
        Ok(ProfileDefinition {
            profile: parsed.profile,
            import_prompt,
            build_prompt,
            default_elevenlabs_voice: parsed.elevenlabs_voice,
            default_elevenlabs_model: parsed.elevenlabs_model,
            field_origins,
        })
    }
}

impl ProfileOverrideStore for FsProfileCatalog {
    fn create_prompt_override(
        &self,
        profile: &ProfileId,
        stage: PromptStage,
        scope: OverrideScope,
    ) -> Result<PromptOverrideTarget, ProfileFailure> {
        let resolved = self.require(profile)?;
        let source = match stage {
            PromptStage::Import => resolved.import_prompt.content,
            PromptStage::Build => resolved.build_prompt.content,
        };
        let target = match scope {
            OverrideScope::Global => self
                .global_profile_dir(profile)
                .join("prompts")
                .join(format!("{}.md.hbs", stage.wire_name())),
            OverrideScope::Deck => self
                .deck_prompt_path(stage)
                .ok_or_else(|| ProfileFailure::Invalid("deck scope requires a workspace".into()))?,
        };
        if target.exists() {
            return Ok(PromptOverrideTarget {
                path: target,
                created: false,
            });
        }
        let Some(parent) = target.parent() else {
            return Err(ProfileFailure::Invalid(
                "prompt target has no parent".into(),
            ));
        };
        fs::create_dir_all(parent).map_err(profile_io)?;
        create_atomic(&target, source.as_bytes())
            .map_err(|error| ProfileFailure::Io(error.to_string()))?;
        Ok(PromptOverrideTarget {
            path: target,
            created: true,
        })
    }
}

#[derive(Deserialize)]
struct ProfileFileDto {
    language: LanguageDto,
    romanisation: RomanisationDto,
    #[serde(default)]
    audio: AudioDefaultsDto,
}

#[derive(Deserialize)]
struct LanguageDto {
    name: String,
    code: String,
    script: String,
    direction: String,
}

#[derive(Deserialize)]
struct RomanisationDto {
    convention: String,
}

#[derive(Default, Deserialize)]
struct AudioDefaultsDto {
    #[serde(default)]
    elevenlabs: ElevenLabsDefaultsDto,
}

#[derive(Default, Deserialize)]
struct ElevenLabsDefaultsDto {
    voice: Option<String>,
    model: Option<String>,
}

struct ParsedProfile {
    profile: LanguageProfile,
    elevenlabs_voice: Option<String>,
    elevenlabs_model: Option<String>,
}

fn parse_profile(id: &ProfileId, text: &str) -> Result<ParsedProfile, ProfileFailure> {
    let dto: ProfileFileDto = toml::from_str(text).map_err(profile_invalid)?;
    let direction = match dto.language.direction.as_str() {
        "ltr" => TextDirection::Ltr,
        "rtl" => TextDirection::Rtl,
        other => {
            return Err(ProfileFailure::Invalid(format!(
                "unknown text direction {other:?}"
            )));
        }
    };
    Ok(ParsedProfile {
        profile: LanguageProfile::new(
            id.clone(),
            LanguageName::parse(dto.language.name).map_err(profile_invalid)?,
            LanguageCode::parse(dto.language.code).map_err(profile_invalid)?,
            ScriptName::parse(dto.language.script).map_err(profile_invalid)?,
            direction,
            RomanisationConvention::parse(&dto.romanisation.convention).map_err(profile_invalid)?,
        ),
        elevenlabs_voice: dto.audio.elevenlabs.voice,
        elevenlabs_model: dto.audio.elevenlabs.model,
    })
}

fn resolve_prompt(
    built_in: Option<&str>,
    global: PathBuf,
    deck: Option<PathBuf>,
) -> Result<ResolvedPrompt, ProfileFailure> {
    if let Some(deck) = deck.filter(|path| path.is_file()) {
        return Ok(ResolvedPrompt {
            content: fs::read_to_string(&deck).map_err(profile_io)?,
            origin: ConfigOrigin::Deck,
            origin_path: Some(deck),
        });
    }
    if global.is_file() {
        return Ok(ResolvedPrompt {
            content: fs::read_to_string(&global).map_err(profile_io)?,
            origin: ConfigOrigin::Global,
            origin_path: Some(global),
        });
    }
    let Some(built_in) = built_in else {
        return Err(ProfileFailure::Invalid(
            "profile has no import/build prompt".to_string(),
        ));
    };
    Ok(ResolvedPrompt {
        content: built_in.to_string(),
        origin: ConfigOrigin::BuiltIn,
        origin_path: None,
    })
}

fn validate_builtin_ids() -> Result<(), ProfileFailure> {
    let mut ids = BTreeMap::new();
    for profile in BUILT_INS {
        if ids.insert(profile.id, ()).is_some() {
            return Err(ProfileFailure::Invalid(format!(
                "duplicate built-in profile id {:?}",
                profile.id
            )));
        }
        ProfileId::parse(profile.id).map_err(profile_invalid)?;
    }
    Ok(())
}

fn summary(definition: &ProfileDefinition) -> ProfileSummary {
    ProfileSummary {
        id: definition.profile.id().clone(),
        language: definition.profile.language().as_str().to_string(),
        code: definition.profile.code().as_str().to_string(),
        romanisation: definition.profile.romanisation().wire_name().to_string(),
    }
}

fn profile_io(error: std::io::Error) -> ProfileFailure {
    ProfileFailure::Io(error.to_string())
}

fn profile_invalid(error: impl std::fmt::Display) -> ProfileFailure {
    ProfileFailure::Invalid(error.to_string())
}
