use clap::builder::styling::{AnsiColor, Styles};
use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "lingo",
    version,
    about = "Build review-ready language-learning cards from raw text",
    long_about = "A local, language-agnostic pipeline: raw text -> reviewed source -> enriched cards -> checks -> audio -> package/export. Import and build emit prompt packets for ChatGPT or Claude; Lingo never calls a model server."
)]
pub struct Cli {
    #[arg(
        long,
        global = true,
        conflicts_with = "color",
        help = "Disable ANSI color"
    )]
    pub no_color: bool,
    #[arg(
        long,
        global = true,
        value_enum,
        default_value_t = ColorArg::Auto,
        help = "Control ANSI color output"
    )]
    pub color: ColorArg,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum ColorArg {
    Auto,
    Always,
    Never,
}

pub fn help_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Cyan.on_default().bold())
        .usage(AnsiColor::Cyan.on_default().bold())
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Yellow.on_default())
        .valid(AnsiColor::Green.on_default())
        .invalid(AnsiColor::Red.on_default())
        .error(AnsiColor::Red.on_default().bold())
        .context(AnsiColor::Yellow.on_default())
        .context_value(AnsiColor::Yellow.on_default().bold())
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create or repair a workspace.
    Init(InitArgs),
    /// Convert raw text into reviewed source YAML through a prompt packet.
    Import(ImportArgs),
    /// Enrich reviewed source into canonical card JSON through a prompt packet.
    Build(BuildArgs),
    /// Run deterministic card checks.
    Check(CheckArgs),
    /// Synthesize missing or replacement audio.
    Audio(AudioArgs),
    /// Publish a self-contained portable package.
    Package(PackageArgs),
    /// Export an Anki APKG.
    Export(ExportArgs),
    /// Show pipeline state and the next useful command.
    Status(StatusArgs),
    /// Inspect or override language profiles and prompts.
    Lang(LangArgs),
    /// Check required local tooling and configuration.
    Doctor,
    /// Serve the local card viewer.
    Viewer(ViewerArgs),
}

#[derive(Debug, Args)]
pub struct InitArgs {
    #[arg(long = "lang", value_name = "PROFILE")]
    pub profile: String,
    #[arg(value_name = "DIR", default_value = ".")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    #[arg(value_name = "RAW_FILE")]
    pub raw: Option<PathBuf>,
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub subtitle: Option<String>,
    #[arg(long, value_name = "REPLY_FILE", conflicts_with = "print")]
    pub apply: Option<PathBuf>,
    #[arg(long, conflicts_with = "apply")]
    pub print: bool,
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long, value_name = "REPLY_FILE", conflicts_with = "print")]
    pub apply: Option<PathBuf>,
    #[arg(long, conflicts_with = "apply")]
    pub print: bool,
}

#[derive(Debug, Args)]
pub struct CheckArgs {
    #[arg(long)]
    pub batch: Option<String>,
}

#[derive(Debug, Args)]
pub struct AudioArgs {
    #[command(subcommand)]
    pub command: Option<AudioCommandArg>,
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long, value_enum)]
    pub backend: Option<AudioBackendArg>,
    #[arg(
        long,
        value_name = "VOICE_ID",
        help = "Use an ElevenLabs voice ID for this run only"
    )]
    pub voice: Option<String>,
    #[arg(long, help = "Replace audio even when a card already has it")]
    pub force: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum AudioBackendArg {
    Gtts,
    Elevenlabs,
}

#[derive(Debug, Subcommand)]
pub enum AudioCommandArg {
    /// List ElevenLabs voices available to the configured API key.
    Voices(AudioVoicesArgs),
    /// Inspect or change audio voice settings.
    Voice(AudioVoiceArgs),
}

#[derive(Debug, Args)]
pub struct AudioVoicesArgs {
    #[arg(long, default_value_t = 25)]
    pub limit: usize,
}

#[derive(Debug, Args)]
pub struct AudioVoiceArgs {
    #[command(subcommand)]
    pub command: AudioVoiceCommandArg,
}

#[derive(Debug, Subcommand)]
pub enum AudioVoiceCommandArg {
    /// Show the configured ElevenLabs voice.
    Show,
    /// Pick an ElevenLabs voice with fzf.
    Select(AudioVoiceSelectArgs),
    /// Set the deck's ElevenLabs voice ID.
    Set {
        #[arg(value_name = "VOICE_ID")]
        voice: String,
    },
}

#[derive(Debug, Args)]
pub struct AudioVoiceSelectArgs {
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
    #[arg(long, help = "Use the selected voice for this synthesis run only")]
    pub for_run: bool,
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long, help = "Replace audio even when a card already has it")]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct PackageArgs {
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long = "dest")]
    pub destination: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct ExportArgs {
    #[arg(long, action = clap::ArgAction::Append)]
    pub batch: Vec<String>,
    #[arg(long)]
    pub all: bool,
    #[arg(long)]
    pub deck: Option<String>,
    #[arg(long = "dest")]
    pub destination: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub batch: Option<String>,
    #[arg(long)]
    pub problems: bool,
}

#[derive(Debug, Args)]
pub struct LangArgs {
    #[command(subcommand)]
    pub command: LangCommand,
}

#[derive(Debug, Subcommand)]
pub enum LangCommand {
    List,
    Show {
        profile: String,
    },
    Which,
    Edit {
        stage: PromptStageArg,
        #[arg(long)]
        profile: Option<String>,
        #[arg(long, conflicts_with = "deck")]
        global: bool,
        #[arg(long, conflicts_with = "global")]
        deck: bool,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum PromptStageArg {
    Import,
    Build,
}

#[derive(Debug, Args)]
pub struct ViewerArgs {
    #[arg(long, default_value_t = 4321)]
    pub port: u16,
    #[arg(long)]
    pub no_open: bool,
    #[arg(long, value_enum)]
    pub lead: Option<DisplayLeadArg>,
    #[arg(long)]
    pub batch: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DisplayLeadArg {
    Romanisation,
    Target,
}
