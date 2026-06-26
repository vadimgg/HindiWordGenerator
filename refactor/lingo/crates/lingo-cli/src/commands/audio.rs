use crate::cli::{AudioArgs, AudioBackendArg};
use crate::commands::{CommandResult, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::{AudioCommand, AudioDeps, AudioMode, synthesize_audio};
use lingo_domain::AudioBackendId;

pub fn run(args: AudioArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let backend = args.backend.map(|value| match value {
        AudioBackendArg::Gtts => AudioBackendId::Gtts,
        AudioBackendArg::Elevenlabs => AudioBackendId::ElevenLabs,
    });
    let report = synthesize_audio(
        &AudioDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            synthesizer: &composition.audio,
        },
        AudioCommand {
            batch: parse_batch(args.batch.as_deref())?,
            mode: if args.force {
                AudioMode::ReplaceAll
            } else {
                AudioMode::MissingOnly
            },
            backend,
        },
    )?;
    output.audio(&report);
    Ok(ExitStatus::Success)
}
