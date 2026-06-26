use crate::cli::{
    AudioArgs, AudioBackendArg, AudioCommandArg, AudioVoiceCommandArg, AudioVoiceSelectArgs,
    AudioVoicesArgs,
};
use crate::commands::{CommandResult, command_error, current_dir, parse_batch};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use lingo_application::ports::DeckContextProvider;
use lingo_application::{
    AudioCommand as SynthesizeAudioCommand, AudioDeps, AudioMode, synthesize_audio,
};
use lingo_domain::AudioBackendId;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use toml::Value;

pub fn run(args: AudioArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    match args.command {
        Some(AudioCommandArg::Voices(voice_args)) => list_voices(&composition, voice_args, output),
        Some(AudioCommandArg::Voice(voice_args)) => match voice_args.command {
            AudioVoiceCommandArg::Show => show_voice(&composition, output),
            AudioVoiceCommandArg::Select(select_args) => {
                select_voice(&composition, select_args, output)
            }
            AudioVoiceCommandArg::Set { voice } => set_voice(&composition, &voice, output),
        },
        None => synthesize(args, &composition, output),
    }
}

fn synthesize(args: AudioArgs, composition: &Composition, output: &Output) -> CommandResult {
    let requested_backend = args.backend.map(|value| match value {
        AudioBackendArg::Gtts => AudioBackendId::Gtts,
        AudioBackendArg::Elevenlabs => AudioBackendId::ElevenLabs,
    });
    if args.voice.is_some() && requested_backend == Some(AudioBackendId::Gtts) {
        return Err(command_error(
            "--voice can only be used with the ElevenLabs backend",
        ));
    }
    let backend = if args.voice.is_some() {
        Some(AudioBackendId::ElevenLabs)
    } else {
        requested_backend
    };
    let report = synthesize_audio(
        &AudioDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            synthesizer: &composition.audio,
        },
        SynthesizeAudioCommand {
            batch: parse_batch(args.batch.as_deref())?,
            mode: if args.force {
                AudioMode::ReplaceAll
            } else {
                AudioMode::MissingOnly
            },
            backend,
            elevenlabs_voice: args.voice,
        },
    )?;
    output.audio(&report);
    Ok(ExitStatus::Success)
}

fn show_voice(composition: &Composition, output: &Output) -> CommandResult {
    let context = composition.workspace.resolve()?;
    println!("ElevenLabs voice");
    println!(
        "  voice  {}",
        context
            .audio
            .elevenlabs_voice
            .as_deref()
            .unwrap_or("(not configured)")
    );
    println!(
        "  model  {}",
        context
            .audio
            .elevenlabs_model
            .as_deref()
            .unwrap_or("(not configured)")
    );
    output
        .note("Use `lingo audio voices` to list voices, then `lingo audio voice set <VOICE_ID>`.");
    Ok(ExitStatus::Success)
}

fn list_voices(composition: &Composition, args: AudioVoicesArgs, output: &Output) -> CommandResult {
    let voices = fetch_voices(composition)?;
    println!("ElevenLabs voices  {} available\n", voices.len());
    println!("  {:<34} {:<24} Category", "Name", "Voice ID");
    for voice in voices.iter().take(args.limit) {
        println!(
            "  {:<34} {:<24} {}",
            truncate(&voice.name, 34),
            voice.voice_id,
            voice.category.as_deref().unwrap_or("-")
        );
    }
    if voices.len() > args.limit {
        println!(
            "\n  showing {} of {}; rerun with --limit {}",
            args.limit,
            voices.len(),
            voices.len()
        );
    }
    output.note("\nPick one with `lingo audio voice select`.");
    Ok(ExitStatus::Success)
}

fn select_voice(
    composition: &Composition,
    args: AudioVoiceSelectArgs,
    output: &Output,
) -> CommandResult {
    let voices = fetch_voices(composition)?;
    let selected = pick_voice_with_fzf(&voices, args.limit)?;
    println!("ElevenLabs voice  selected");
    println!("  name   {}", selected.name);
    println!("  voice  {}", selected.voice_id);

    if args.for_run {
        let report = synthesize_audio(
            &AudioDeps {
                workspace: &composition.workspace,
                context: &composition.workspace,
                synthesizer: &composition.audio,
            },
            SynthesizeAudioCommand {
                batch: parse_batch(args.batch.as_deref())?,
                mode: if args.force {
                    AudioMode::ReplaceAll
                } else {
                    AudioMode::MissingOnly
                },
                backend: Some(AudioBackendId::ElevenLabs),
                elevenlabs_voice: Some(selected.voice_id),
            },
        )?;
        output.audio(&report);
    } else {
        set_voice(composition, &selected.voice_id, output)?;
    }
    Ok(ExitStatus::Success)
}

fn set_voice(composition: &Composition, voice: &str, output: &Output) -> CommandResult {
    let config = composition.workspace.layout().config_file();
    set_elevenlabs_voice(&config, voice)?;
    println!("ElevenLabs voice  set");
    println!("  voice  {voice}");
    println!("  config {}", config.display());
    output.note("Run `lingo audio --backend elevenlabs --force` to regenerate existing audio.");
    Ok(ExitStatus::Success)
}

fn elevenlabs_key(composition: &Composition) -> Result<String, Box<dyn std::error::Error>> {
    let context = composition.workspace.resolve()?;
    let Some(name) = context.audio.elevenlabs_key_env.as_deref() else {
        return Err(command_error(
            "ElevenLabs API key is not configured; set [audio.elevenlabs].api_key = \"env:ELEVENLABS_API_KEY\"",
        ));
    };
    std::env::var(name)
        .map_err(|_| command_error(format!("environment variable ${name} is not set")))
}

fn fetch_voices(composition: &Composition) -> Result<Vec<Voice>, Box<dyn std::error::Error>> {
    let key = elevenlabs_key(composition)?;
    let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
    let response = client
        .get("https://api.elevenlabs.io/v1/voices")
        .header("xi-api-key", key)
        .send()?;
    let status = response.status();
    let body = response.text()?;
    if !status.is_success() {
        return Err(command_error(format!(
            "ElevenLabs returned HTTP {}: {}",
            status.as_u16(),
            body.trim()
        )));
    }
    let response: VoicesResponse = serde_json::from_str(&body)?;
    Ok(response.voices)
}

fn pick_voice_with_fzf(
    voices: &[Voice],
    limit: usize,
) -> Result<Voice, Box<dyn std::error::Error>> {
    let fzf = which::which("fzf").map_err(|_| {
        command_error("fzf is not installed; install fzf or use `lingo audio voices`")
    })?;
    let mut child = Command::new(fzf)
        .args([
            "--delimiter",
            "\t",
            "--with-nth",
            "1,3",
            "--prompt",
            "ElevenLabs voice> ",
            "--header",
            "Enter selects a voice; Esc cancels",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| command_error("could not open fzf stdin"))?;
        for voice in voices.iter().take(limit) {
            writeln!(
                stdin,
                "{}\t{}\t{}",
                voice.name,
                voice.voice_id,
                voice.category.as_deref().unwrap_or("-")
            )?;
        }
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(command_error("voice selection cancelled"));
    }
    let selected = String::from_utf8(output.stdout)?;
    let voice_id = selected
        .split('\t')
        .nth(1)
        .ok_or_else(|| command_error("fzf selection did not include a voice ID"))?
        .trim();
    voices
        .iter()
        .find(|voice| voice.voice_id == voice_id)
        .cloned()
        .ok_or_else(|| command_error(format!("selected voice {voice_id} was not found")))
}

fn set_elevenlabs_voice(config: &Path, voice: &str) -> Result<(), Box<dyn std::error::Error>> {
    let text = fs::read_to_string(config)?;
    let mut value: Value = toml::from_str(&text)?;
    let root = value
        .as_table_mut()
        .ok_or_else(|| command_error("config.toml root is not a table"))?;
    let audio = root
        .entry("audio")
        .or_insert_with(|| Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| command_error("[audio] is not a table"))?;
    let elevenlabs = audio
        .entry("elevenlabs")
        .or_insert_with(|| Value::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| command_error("[audio.elevenlabs] is not a table"))?;
    elevenlabs.insert("voice".to_string(), Value::String(voice.to_string()));
    fs::write(config, toml::to_string_pretty(&value)?)?;
    Ok(())
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_string();
    }
    let mut truncated: String = value.chars().take(width.saturating_sub(3)).collect();
    truncated.push_str("...");
    truncated
}

#[derive(Debug, Deserialize)]
struct VoicesResponse {
    voices: Vec<Voice>,
}

#[derive(Clone, Debug, Deserialize)]
struct Voice {
    voice_id: String,
    name: String,
    category: Option<String>,
}
