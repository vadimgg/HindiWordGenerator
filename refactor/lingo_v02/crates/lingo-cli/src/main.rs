#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

mod viewer;

use clap::{Args, Parser, Subcommand, ValueEnum};
use lingo_application::ports::{
    LibraryStore, PageRequest, ReorderSentences, SentenceQuery, SentenceSelection, SentenceSort,
    UpdateSentence, WordQuery,
};
use lingo_application::{
    ApplyEnrich, ApplyExtract, AudioCommand, AudioDeps, AudioMode, EnrichDeps, ExportDeps,
    ExportRequest, ExtractDeps, ImportDeps, ImportPackage, ImportedSentence, PackageDeps,
    PackageFormat, PackageRequest, PrepareEnrich, PrepareExtract, StatusDeps, apply_enrich,
    apply_extract, export_anki, import_package, list_library, list_words, package, prepare_enrich,
    prepare_extract, reorder_sentences, status, synthesize_audio, update_sentence,
};
use lingo_artifacts::{ApkgExporter, PortablePackagePublisher};
use lingo_audio::AudioCatalog;
use lingo_domain::{
    AudioBackendId, CollectionTitle, FieldAuthoritySet, Gloss, LiteralGloss, Register, Romanisation,
    RunId, SectionName, SentenceId, SentenceStatus, SentenceTags, TargetText, TokenBreakdown,
};
use lingo_prompt::HandlebarsPromptEngine;
use lingo_workspace_fs::{FsWorkspace, get_config_value, set_config_value};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Lingo — a sentence-centric language-learning library.
///
/// Lingo turns raw foreign-language text into studyable sentences and stores
/// them in a local SQLite library (`library.db`). The canonical workflow:
///
///   1. extract  raw text            → draft sentences      (via an LLM packet)
///   2. enrich   drafts              → translation + breakdown (via an LLM packet)
///   3. audio    sentences           → spoken MP3s (gTTS)
///   4. organize reorder / re-title / edit fields
///   5. package  → portable JSON or SQLite (for Grasp); export → Anki .apkg
///
/// You can also `import` a package produced by `package`. Run with no command to
/// open the local web viewer.
///
/// The LLM steps are manual by default: copy the printed packet into ChatGPT or
/// Claude, then apply the reply. Use `--out <file> --watch` to hand the packet to
/// an agent (Codex / Claude Code) that writes `reply.md` in place.
#[derive(Debug, Parser)]
#[command(name = "lingo", version, propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create or repair a deck (workspace) in DIR.
    Init(InitArgs),
    /// Turn raw text into draft sentences through an LLM prompt packet.
    Extract(ExtractArgs),
    /// Fill translation, romanisation, literal gloss, and word breakdown.
    Enrich(EnrichArgs),
    /// Merge a package (the output of `package`) into this deck's library.
    Import(ImportArgs),
    /// List sentences (optionally as JSON, or filtered by status / missing audio).
    Ls(ListArgs),
    /// Show one sentence by id.
    Show(ShowArgs),
    /// Show the words lexicon — each word, how many sentences it is in, meanings.
    Words(WordsArgs),
    /// Synthesize spoken audio (gTTS) for sentences that need it.
    Audio(AudioArgs),
    /// Reorder, re-section, and edit sentences (edits become human-authored).
    Organize(OrganizeArgs),
    /// Read or write deck settings in config.toml.
    Config(ConfigArgs),
    /// Publish a portable package: one-file-per-sentence JSON (default) or a db copy.
    Package(PackageArgs),
    /// Export selected sentences to an Anki .apkg deck.
    Export(ExportArgs),
    /// Show library counts and the next useful command.
    Status,
    /// Serve the local web viewer (also the default when no command is given).
    Viewer(ViewerArgs),
}

#[derive(Debug, Args)]
struct ViewerArgs {
    /// Port for the local web server.
    #[arg(long, default_value_t = 4321)]
    port: u16,
}

#[derive(Debug, Args)]
struct OrganizeArgs {
    #[command(subcommand)]
    command: OrganizeCommand,
}

#[derive(Debug, Subcommand)]
enum OrganizeCommand {
    /// Edit a sentence's fields (marks edited fields human-authored).
    Set {
        id: String,
        #[arg(long)]
        section: Option<String>,
        #[arg(long)]
        target: Option<String>,
        #[arg(long)]
        romanisation: Option<String>,
        #[arg(long)]
        english: Option<String>,
        #[arg(long)]
        literal: Option<String>,
        #[arg(long)]
        register: Option<String>,
        #[arg(long = "tag")]
        tags: Vec<String>,
    },
    /// Move a sentence to a 1-based position within its collection.
    Move {
        id: String,
        #[arg(long = "to")]
        to: usize,
    },
}

#[derive(Debug, Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Print the whole config, or a single dotted key.
    Get { key: Option<String> },
    /// Set a dotted key (e.g. display.lead target) and write config.toml.
    Set { key: String, value: String },
}

#[derive(Debug, Args)]
struct InitArgs {
    /// Language profile to scaffold (e.g. hindi).
    #[arg(long = "lang", default_value = "hindi")]
    profile: String,
    /// Directory to create the deck in.
    #[arg(value_name = "DIR", default_value = ".")]
    directory: PathBuf,
}

/// Extract sentences from raw text.
///
/// Prints a prompt packet by default. Paste it into ChatGPT/Claude and apply the
/// reply with `--apply`, or hand it to an agent with `--out <file> --watch`.
#[derive(Debug, Args)]
struct ExtractArgs {
    /// Raw text file to segment into sentences.
    raw: PathBuf,
    /// Collection title to file these sentences under.
    #[arg(long)]
    collection: Option<String>,
    /// Section (e.g. a chapter) within the collection.
    #[arg(long)]
    section: Option<String>,
    /// Print the prompt packet and exit.
    #[arg(long)]
    print: bool,
    /// Apply a model reply file (creates draft sentences).
    #[arg(long, value_name = "REPLY")]
    apply: Option<PathBuf>,
    /// Write the prompt packet to FILE (file-handoff mode for agents).
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    /// After --out, wait for a sibling reply.md and apply it automatically.
    #[arg(long)]
    watch: bool,
}

/// Enrich draft sentences with translation, romanisation, literal gloss, and a
/// word-by-word breakdown. Human-authored fields are never overwritten.
#[derive(Debug, Args)]
struct EnrichArgs {
    /// Maximum sentences to claim for this prompt (context-window control).
    #[arg(long, default_value_t = 20)]
    limit: usize,
    /// Re-enrich already-enriched sentences (still preserves human fields).
    #[arg(long)]
    force: bool,
    /// Print the prompt packet and exit.
    #[arg(long)]
    print: bool,
    /// Apply a model reply file for a claimed run (needs --run).
    #[arg(long, value_name = "REPLY")]
    apply: Option<PathBuf>,
    /// Run id to apply against, or to reset.
    #[arg(long)]
    run: Option<String>,
    /// Return abandoned 'enriching' sentences to 'draft' (optionally for --run).
    #[arg(long)]
    reset: bool,
    /// Write the prompt packet to FILE (file-handoff mode for agents).
    #[arg(long, value_name = "FILE")]
    out: Option<PathBuf>,
    /// After --out, wait for a sibling reply.md and apply it automatically.
    #[arg(long)]
    watch: bool,
}

#[derive(Debug, Args)]
struct ImportArgs {
    /// Package directory to import (the output of `lingo package`).
    #[arg(long = "from", value_name = "DIR")]
    from: PathBuf,
}

#[derive(Debug, Args)]
struct ListArgs {
    /// Output JSON instead of a table.
    #[arg(long)]
    json: bool,
    /// Filter by status: draft | enriching | enriched.
    #[arg(long)]
    status: Option<String>,
    /// Only sentences that have no audio yet.
    #[arg(long = "missing-audio")]
    missing_audio: bool,
}

#[derive(Debug, Args)]
struct ShowArgs {
    /// Sentence id.
    id: String,
}

#[derive(Debug, Args)]
struct WordsArgs {
    /// Only words appearing in at least this many sentences.
    #[arg(long = "min-count")]
    min_count: Option<usize>,
    /// Output JSON instead of a table.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct AudioArgs {
    /// TTS backend (gtts is free and local; elevenlabs needs an API key).
    #[arg(long, value_enum, default_value = "gtts")]
    backend: BackendArg,
    /// Re-synthesize audio even for sentences that already have it.
    #[arg(long)]
    force: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BackendArg { Gtts, Elevenlabs }

#[derive(Debug, Args)]
struct PackageArgs {
    /// Destination folder (defaults to the deck's configured package path).
    #[arg(long = "dest")]
    destination: Option<PathBuf>,
    /// Output format: json (one file per sentence) or db (a SQLite copy).
    #[arg(long = "as", value_enum, default_value = "json")]
    format: PackageFormatArg,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum PackageFormatArg { Json, Db }

#[derive(Debug, Args)]
struct ExportArgs {
    /// Anki deck name (e.g. Hindi::Sentences).
    #[arg(long)]
    deck: String,
    /// Destination .apkg file.
    #[arg(long = "dest")]
    destination: PathBuf,
}

fn main() -> std::process::ExitCode {
    match run(Cli::parse()) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    // No subcommand → open the local viewer.
    let command = cli.command.unwrap_or(Command::Viewer(ViewerArgs { port: 4321 }));
    match command {
        Command::Init(args) => {
            let workspace = FsWorkspace::init(args.directory, &args.profile)?;
            println!("initialized {}", workspace.layout().root().display());
        }
        Command::Extract(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let prompts = HandlebarsPromptEngine::with_overrides(workspace.layout().prompts_dir());
            let run_id = run_id("extract")?;
            let raw = std::fs::read_to_string(&args.raw)?;
            let deps = ExtractDeps { library: &workspace, context: &workspace, prompts: &prompts };
            let section = args.section.map(SectionName::parse).transpose()?;
            let collection = args.collection.map(CollectionTitle::parse).transpose()?;
            if let Some(out) = &args.out {
                let prep = prepare_extract(&deps, PrepareExtract { run_id: run_id.clone(), raw: raw.clone(), collection: collection.clone(), section: section.clone(), batch: None })?;
                write_packet(out, &prep.packet)?;
                println!("run: {}\nprompt -> {}", prep.run_id, out.display());
                if args.watch {
                    let reply = watch_for_reply(&reply_path(out))?;
                    let report = apply_extract(&deps, ApplyExtract { run_id, reply, collection, section, batch_id: None })?;
                    println!("created {} draft sentence(s)", report.created);
                }
                return Ok(());
            }
            if args.print || args.apply.is_none() {
                let prep = prepare_extract(&deps, PrepareExtract { run_id: run_id.clone(), raw: raw.clone(), collection: collection.clone(), section: section.clone(), batch: None })?;
                println!("run: {}\n{}", prep.run_id, prep.packet);
                if args.apply.is_none() { return Ok(()); }
            }
            let Some(reply) = args.apply else { return Ok(()); };
            let report = apply_extract(&deps, ApplyExtract { run_id, reply: std::fs::read_to_string(reply)?, collection, section, batch_id: None })?;
            println!("created {} draft sentence(s)", report.created);
        }
        Command::Enrich(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let prompts = HandlebarsPromptEngine::with_overrides(workspace.layout().prompts_dir());
            let deps = EnrichDeps { library: &workspace, context: &workspace, prompts: &prompts };
            if args.reset {
                let run = args.run.map(RunId::parse).transpose()?;
                let report = lingo_application::reset_enrichment_claim(&deps, lingo_application::ResetEnrich { run_id: run })?;
                println!("reset {} sentence(s)", report.reset);
                return Ok(());
            }
            if let Some(reply) = args.apply {
                let run = args.run.ok_or_else(|| std::io::Error::other("--run is required with --apply"))?;
                let report = apply_enrich(&deps, ApplyEnrich { run_id: RunId::parse(run)?, reply: std::fs::read_to_string(reply)? })?;
                println!("enriched {} sentence(s)", report.updated);
                return Ok(());
            }
            let run_id = run_id("enrich")?;
            let prep = prepare_enrich(&deps, PrepareEnrich { run_id: run_id.clone(), selection: SentenceSelection::All, limit: args.limit, force: args.force })?;
            if let Some(out) = &args.out {
                write_packet(out, &prep.packet)?;
                println!("run: {}\nclaimed: {}\nprompt -> {}", run_id, prep.claimed, out.display());
                if args.watch {
                    let reply = watch_for_reply(&reply_path(out))?;
                    let report = apply_enrich(&deps, ApplyEnrich { run_id, reply })?;
                    println!("enriched {} sentence(s)", report.updated);
                }
                return Ok(());
            }
            println!("run: {}\nclaimed: {}\n{}", run_id, prep.claimed, prep.packet);
        }
        Command::Import(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let pkg = read_package(&args.from)?;
            let files = workspace.audio_files();
            let report = import_package(&ImportDeps { library: &workspace, context: &workspace, audio_files: &files }, pkg)?;
            println!("imported {} sentence(s), {} audio", report.imported, report.audio);
        }
        Command::Ls(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let selection = if args.missing_audio {
                SentenceSelection::MissingAudio
            } else if let Some(status) = args.status {
                SentenceSelection::Status(SentenceStatus::parse(&status)?)
            } else {
                SentenceSelection::All
            };
            let page = list_library(&workspace, lingo_application::default_query(selection))?;
            if args.json {
                let rows = page.sentences.iter().map(|s| serde_json::json!({
                    "id": s.id().as_str(),
                    "section": s.section().map(|v| v.as_str()),
                    "order": s.order().get(),
                    "target": s.target().as_str(),
                    "romanisation": s.romanisation().map(|v| v.as_str()),
                    "english": s.english().map(|v| v.as_str()),
                    "status": s.status().wire_name(),
                    "audio": s.audio().is_some(),
                })).collect::<Vec<_>>();
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                for sentence in page.sentences {
                    println!("{}\t{}\t{}", sentence.id(), sentence.status().wire_name(), sentence.target().as_str());
                }
            }
        }
        Command::Show(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let sentence = workspace.get_sentence(&SentenceId::parse(args.id)?)?;
            println!("{}\n{}\n{}", sentence.id(), sentence.target(), sentence.english().map(|v| v.as_str()).unwrap_or(""));
        }
        Command::Words(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let page = list_words(&workspace, WordQuery { min_count: args.min_count, ..WordQuery::default() })?;
            if args.json {
                let rows = page.words.iter().map(|w| serde_json::json!({
                    "form": w.word.form().as_str(),
                    "key": w.word.key().as_str(),
                    "count": w.sentence_count,
                    "meanings": w.meanings.iter().map(|m| m.as_str()).collect::<Vec<_>>()
                })).collect::<Vec<_>>();
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else {
                for word in page.words {
                    println!("{}\t{}\t{}", word.word.form().as_str(), word.sentence_count, word.meanings.iter().map(|m| m.as_str()).collect::<Vec<_>>().join(", "));
                }
            }
        }
        Command::Audio(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let audio = AudioCatalog::production();
            let files = workspace.audio_files();
            let report = synthesize_audio(&AudioDeps { library: &workspace, context: &workspace, synthesizer: &audio, files: &files }, AudioCommand {
                selection: SentenceSelection::All,
                mode: if args.force { AudioMode::ReplaceAll } else { AudioMode::MissingOnly },
                backend: Some(match args.backend { BackendArg::Gtts => AudioBackendId::Gtts, BackendArg::Elevenlabs => AudioBackendId::ElevenLabs }),
                voice: None,
                model: None,
            })?;
            println!("updated {} skipped {}", report.updated, report.skipped);
        }
        Command::Organize(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            match args.command {
                OrganizeCommand::Set { id, section, target, romanisation, english, literal, register, tags } => {
                    let request = UpdateSentence {
                        id: SentenceId::parse(id)?,
                        section: section.map(|s| SectionName::parse(s)).transpose()?.map(Some),
                        target: target.map(TargetText::parse).transpose()?,
                        romanisation: romanisation.map(Romanisation::parse).transpose()?.map(Some),
                        english: english.map(Gloss::parse).transpose()?.map(Some),
                        literal: literal.map(LiteralGloss::parse).transpose()?.map(Some),
                        register: register.as_deref().map(Register::parse).transpose()?.map(Some),
                        tags: if tags.is_empty() { None } else { Some(SentenceTags::try_from_values(tags)?) },
                        status: None,
                        batch_id: None,
                        title: None,
                    };
                    let report = update_sentence(&workspace, request)?;
                    println!("updated {}", report.sentence.id());
                }
                OrganizeCommand::Move { id, to } => {
                    let id = SentenceId::parse(id)?;
                    let collection = workspace.get_sentence(&id)?.collection().clone();
                    let page = workspace.list_sentences(&SentenceQuery {
                        selection: SentenceSelection::Collection(collection.clone()),
                        sort: SentenceSort::LibraryOrder,
                        page: PageRequest { limit: 1_000_000, offset: 0 },
                    })?;
                    let mut ordered: Vec<SentenceId> = page.sentences.iter().map(|s| s.id().clone()).filter(|x| x != &id).collect();
                    let index = to.saturating_sub(1).min(ordered.len());
                    ordered.insert(index, id);
                    let report = reorder_sentences(&workspace, ReorderSentences { collection, ordered_ids: ordered })?;
                    println!("reordered {} sentence(s)", report.moved);
                }
            }
        }
        Command::Config(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let config_file = workspace.layout().config_file();
            match args.command {
                ConfigCommand::Get { key: Some(key) } => println!("{}", get_config_value(&config_file, &key)?),
                ConfigCommand::Get { key: None } => print!("{}", std::fs::read_to_string(&config_file)?),
                ConfigCommand::Set { key, value } => {
                    set_config_value(&config_file, &key, &value)?;
                    println!("{key} = {value}   (config.toml updated)");
                }
            }
        }
        Command::Package(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let publisher = PortablePackagePublisher;
            let report = package(&PackageDeps { library: &workspace, context: &workspace, publisher: &publisher }, PackageRequest {
                selection: SentenceSelection::All,
                destination: args.destination,
                format: match args.format { PackageFormatArg::Json => PackageFormat::Json, PackageFormatArg::Db => PackageFormat::Db },
            })?;
            println!("packaged {} sentence(s) -> {}", report.sentences, report.artifact.path.display());
        }
        Command::Export(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let exporter = ApkgExporter;
            let report = export_anki(&ExportDeps { library: &workspace, context: &workspace, exporter: &exporter }, ExportRequest {
                selection: SentenceSelection::ReadyToPublish,
                deck: args.deck,
                destination: args.destination,
            })?;
            println!("exported {} sentence(s) -> {}", report.sentences, report.artifact.path.display());
        }
        Command::Status => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            let report = status(&StatusDeps { library: &workspace, context: &workspace })?;
            println!("sentences {} (draft {} enriching {} enriched {})", report.summary.sentences, report.summary.draft, report.summary.enriching, report.summary.enriched);
            if let Some(hint) = report.next.command_hint() { println!("next: {}", hint.as_str()); }
        }
        Command::Viewer(args) => {
            let workspace = FsWorkspace::discover(std::env::current_dir()?)?;
            viewer::serve(&workspace, args.port)?;
        }
    }
    Ok(())
}

/// Read a JSON package (the layout `lingo package` emits) into typed import DTOs.
#[derive(serde::Deserialize)]
struct SentenceFileDto {
    id: Option<String>,
    target: String,
    romanisation: Option<String>,
    english: Option<String>,
    literal: Option<String>,
    register: Option<String>,
    section: Option<String>,
    #[serde(default)]
    authority: FieldAuthoritySet,
    breakdown: Option<TokenBreakdown>,
    #[serde(default)]
    tags: Vec<String>,
    audio: Option<String>,
}

fn read_package(dir: &Path) -> Result<ImportPackage, Box<dyn Error>> {
    let sentences_dir = dir.join("sentences");
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&sentences_dir)
        .map_err(|error| std::io::Error::other(format!("{}: {error}", sentences_dir.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .collect();
    paths.sort();

    let mut sentences = Vec::with_capacity(paths.len());
    for path in paths {
        let file: SentenceFileDto = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
        let audio = match &file.audio {
            Some(relative) => {
                let asset = dir.join(relative);
                if asset.is_file() { Some(std::fs::read(asset)?) } else { None }
            }
            None => None,
        };
        let status = if file.breakdown.is_some() && file.literal.is_some()
            && file.romanisation.is_some() && file.english.is_some()
        {
            SentenceStatus::Enriched
        } else {
            SentenceStatus::Draft
        };
        sentences.push(ImportedSentence {
            section: file.section.map(SectionName::parse).transpose()?,
            target: TargetText::parse(file.target)?,
            romanisation: file.romanisation.map(Romanisation::parse).transpose()?,
            english: file.english.map(Gloss::parse).transpose()?,
            literal: file.literal.map(LiteralGloss::parse).transpose()?,
            register: file.register.as_deref().map(Register::parse).transpose()?,
            authority: file.authority,
            tags: SentenceTags::try_from_values(file.tags)?,
            breakdown: file.breakdown,
            status,
            source_item: file.id,
            audio,
        });
    }
    let package_name = dir.file_name().and_then(|name| name.to_str()).unwrap_or("package").to_string();
    Ok(ImportPackage { package_name, sentences })
}

/// Write a prompt packet to `path`, creating parent directories.
fn write_packet(path: &Path, packet: &str) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() { std::fs::create_dir_all(parent)?; }
    }
    std::fs::write(path, packet)?;
    Ok(())
}

/// The reply file an agent should write next to the prompt (`<dir>/reply.md`).
fn reply_path(prompt: &Path) -> PathBuf {
    prompt.parent().map(|dir| dir.join("reply.md")).unwrap_or_else(|| PathBuf::from("reply.md"))
}

/// Poll for a non-empty reply file (file-handoff mode), up to 15 minutes.
fn watch_for_reply(path: &Path) -> Result<String, Box<dyn Error>> {
    println!("watching {} …", path.display());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(900);
    loop {
        if path.is_file() {
            let content = std::fs::read_to_string(path)?;
            if !content.trim().is_empty() { return Ok(content); }
        }
        if std::time::Instant::now() > deadline {
            return Err(Box::new(std::io::Error::other(format!("timed out waiting for {}", path.display()))));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn run_id(prefix: &str) -> Result<RunId, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(RunId::parse(format!("{prefix}-{millis:x}"))?)
}
