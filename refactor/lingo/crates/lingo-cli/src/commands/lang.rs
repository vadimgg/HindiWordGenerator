use crate::cli::{LangArgs, LangCommand, PromptStageArg};
use crate::commands::{CommandResult, current_dir};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::interaction::open_editor;
use crate::output::Output;
use lingo_application::ports::{DeckContextProvider, OverrideScope, PromptStage};
use lingo_application::{
    EditPromptRequest, create_prompt_override, list_languages, show_language, which_prompts,
};
use lingo_domain::ProfileId;
use lingo_workspace_fs::{FsProfileCatalog, default_global_config_dir};

pub fn run(args: LangArgs, output: &Output) -> CommandResult {
    match args.command {
        LangCommand::List => {
            let catalog = FsProfileCatalog::global(default_global_config_dir());
            output.languages(&list_languages(&catalog)?);
        }
        LangCommand::Show { profile } => {
            let catalog = FsProfileCatalog::global(default_global_config_dir());
            output.language(&show_language(&catalog, &ProfileId::parse(profile)?)?);
        }
        LangCommand::Which => {
            let composition = Composition::discover(&current_dir()?)?;
            output.prompt_origins(&which_prompts(&composition.workspace)?);
        }
        LangCommand::Edit {
            stage,
            profile,
            global,
            deck: _,
        } => {
            let composition = Composition::discover(&current_dir()?)?;
            let profile = match profile {
                Some(profile) => ProfileId::parse(profile)?,
                None => composition.workspace.resolve()?.profile.id().clone(),
            };
            let report = create_prompt_override(
                &composition.profiles,
                EditPromptRequest {
                    profile,
                    stage: match stage {
                        PromptStageArg::Import => PromptStage::Import,
                        PromptStageArg::Build => PromptStage::Build,
                    },
                    scope: if global {
                        OverrideScope::Global
                    } else {
                        OverrideScope::Deck
                    },
                },
            )?;
            println!(
                "Prompt override  {}  {}",
                if report.target.created {
                    "created"
                } else {
                    "kept"
                },
                report.target.path.display()
            );
            open_editor(&report.target.path)?;
        }
    }
    Ok(ExitStatus::Success)
}
