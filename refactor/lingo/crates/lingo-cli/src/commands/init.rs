use crate::cli::InitArgs;
use crate::commands::CommandResult;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{InitRequest, init};
use lingo_domain::ProfileId;
use lingo_workspace_fs::{FsProfileCatalog, FsWorkspaceBootstrap, default_global_config_dir};

pub fn run(args: InitArgs, output: &Output) -> CommandResult {
    let global = default_global_config_dir();
    let profiles = FsProfileCatalog::global(global.clone());
    let bootstrap = FsWorkspaceBootstrap::new(global);
    let report = init(
        &bootstrap,
        &profiles,
        InitRequest {
            target: args.directory,
            profile: ProfileId::parse(args.profile)?,
        },
    )?;
    output.init(&report);
    Ok(ExitStatus::Success)
}
