use crate::cli::BuildArgs;
use crate::commands::{CommandResult, command_error, current_dir, parse_batch, run_id};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::interaction::{copy_to_clipboard, edit_reply};
use crate::output::Output;
use lingo_application::ports::WorkspaceStore;
use lingo_application::{ApplyBuild, BuildDeps, PrepareBuild, apply_build, prepare_build};
use lingo_domain::BatchId;
use std::collections::BTreeSet;
use std::fs;

pub fn run(args: BuildArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let batch = match parse_batch(args.batch.as_deref())? {
        Some(batch) => batch,
        None => select_pending_batch(&composition)?,
    };
    let run_id = run_id("build", &batch)?;
    let deps = BuildDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let preparation = prepare_build(
        &deps,
        PrepareBuild {
            run_id: run_id.clone(),
            batch,
            journal: !args.print,
        },
    )?;
    if args.print {
        output.packet(&preparation.packet);
        return Ok(ExitStatus::Success);
    }
    let run = preparation
        .run
        .ok_or_else(|| command_error("prepared run was not journaled"))?;
    let prompt_path = composition.workspace.root().join(&run.prompt_path);
    let reply_path = composition.workspace.root().join(&run.reply_path);
    output.prepared("Build", &prompt_path, &reply_path);
    if let Err(error) = copy_to_clipboard(&preparation.packet) {
        output.note(&format!("clipboard unavailable: {error}"));
    }
    let reply = match args.apply {
        Some(path) => fs::read_to_string(path)?,
        None => edit_reply(&reply_path)?,
    };
    if reply.trim().is_empty() {
        output.note("No reply was applied; canonical card data was not changed.");
        return Ok(ExitStatus::ActionRequired);
    }
    let report = apply_build(&deps, ApplyBuild { run_id, reply })?;
    output.built(&report);
    Ok(ExitStatus::Success)
}

fn select_pending_batch(composition: &Composition) -> Result<BatchId, Box<dyn std::error::Error>> {
    let completed = composition
        .workspace
        .list_card_batches()?
        .into_iter()
        .collect::<BTreeSet<_>>();
    composition
        .workspace
        .list_source_batches()?
        .into_iter()
        .find(|batch| !completed.contains(batch))
        .ok_or_else(|| command_error("no source batch needs a build"))
}
