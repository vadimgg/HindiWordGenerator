use crate::cli::ImportArgs;
use crate::commands::{CommandResult, command_error, current_dir, run_id};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::interaction::{copy_to_clipboard, edit_reply};
use crate::output::Output;
use lingo_application::ports::{RawDocumentSummary, WorkspaceStore};
use lingo_application::{ApplyImport, ImportDeps, PrepareImport, apply_import, prepare_import};
use lingo_domain::{BatchId, RawDocumentId, SourceSubtitle, SourceTitle};
use std::fs;
use std::path::Path;

pub fn run(args: ImportArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let raw = select_raw(&composition, args.raw.as_deref())?;
    let batch = match args.batch {
        Some(value) => BatchId::parse(value)?,
        None => match raw.batch_hint.clone() {
            Some(batch) => batch,
            None => BatchId::parse(raw.id.as_str())?,
        },
    };
    let title = SourceTitle::parse(args.title.unwrap_or_else(|| human_title(raw.id.as_str())))?;
    let subtitle = args.subtitle.map(SourceSubtitle::parse).transpose()?;
    let run_id = run_id("import", &batch)?;
    let deps = ImportDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let preparation = prepare_import(
        &deps,
        PrepareImport {
            run_id: run_id.clone(),
            raw: raw.id,
            batch,
            title,
            subtitle,
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
    output.prepared("Import", &prompt_path, &reply_path);
    if let Err(error) = copy_to_clipboard(&preparation.packet) {
        output.note(&format!("clipboard unavailable: {error}"));
    }
    let reply = match args.apply {
        Some(path) => fs::read_to_string(path)?,
        None => edit_reply(&reply_path)?,
    };
    if reply.trim().is_empty() {
        output.note("No reply was applied; canonical source data was not changed.");
        return Ok(ExitStatus::ActionRequired);
    }
    let report = apply_import(&deps, ApplyImport { run_id, reply })?;
    output.imported(&report);
    Ok(ExitStatus::Success)
}

fn select_raw(
    composition: &Composition,
    requested: Option<&Path>,
) -> Result<RawDocumentSummary, Box<dyn std::error::Error>> {
    let raw = composition.workspace.list_raw()?;
    match requested {
        None => raw
            .into_iter()
            .next()
            .ok_or_else(|| command_error("no raw files found under raw/")),
        Some(path) => {
            let stem = path
                .file_stem()
                .and_then(|value| value.to_str())
                .ok_or_else(|| command_error("raw filename must be UTF-8"))?;
            let normalized = slug(stem);
            let id = RawDocumentId::parse(normalized)?;
            raw.into_iter()
                .find(|summary| {
                    summary.id == id
                        || summary.relative_path == path
                        || summary.relative_path.file_name() == path.file_name()
                })
                .ok_or_else(|| {
                    command_error(format!(
                        "raw file {} is not present under this workspace's raw/ directory",
                        path.display()
                    ))
                })
        }
    }
}

fn human_title(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_uppercase().collect::<String>() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            result.push(character.to_ascii_lowercase());
            separator = false;
        } else if !separator && !result.is_empty() {
            result.push('-');
            separator = true;
        }
    }
    result.trim_end_matches('-').to_string()
}
