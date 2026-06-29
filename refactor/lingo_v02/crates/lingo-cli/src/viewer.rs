//! Local web viewer.
//!
//! Serves the Astro single-page app under `viewer/dist` (embedded at build time)
//! and implements its `GET /api/view/state` contract by reading the deck's
//! `library.db` + `config.toml`. Audio is served from the deck's `audio/` dir.
//! Mutations are not wired yet: the UI surfaces the equivalent CLI command.

use lingo_application::ports::{
    CreateBatch, DeckContextProvider, LibraryStore, PageRequest, SentenceQuery, SentenceSelection,
    SentenceSort, UpdateSentence,
};
use lingo_application::{
    ApplyEnrich, ApplyExtract, BatchRef, EnrichDeps, ExtractDeps, ImportDeps, ImportPackage,
    ImportedSentence, PackageDeps, PackageFormat, PackageRequest, PrepareEnrich, PrepareExtract,
    apply_enrich, apply_extract, import_package, package, prepare_enrich, prepare_extract,
    update_sentence,
};
use lingo_artifacts::PortablePackagePublisher;
use lingo_domain::{
    Batch, BatchId, BatchName, BreakdownItem, FieldAuthoritySet, Gloss, LiteralGloss, Register,
    Romanisation, RunId, SectionName, SentenceId, SentenceStatus, SentenceTags, TargetText,
    TokenBreakdown,
};
use lingo_prompt::HandlebarsPromptEngine;
use lingo_workspace_fs::{FsWorkspace, read_config, set_config_value};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tiny_http::{Header, Response, Server};

// The built Astro viewer, embedded so the installed binary is self-contained.
// Rebuild with `(cd viewer && npm run build:static)` after changing the UI.
const INDEX_HTML: &str = include_str!("../../../viewer/dist/index.html");
const APP_MJS: &str = include_str!("../../../viewer/dist/viewer/app.mjs");
const COMMANDS_MJS: &str = include_str!("../../../viewer/dist/viewer/commands.mjs");
const STATE_MJS: &str = include_str!("../../../viewer/dist/viewer/state.mjs");
const VIEWER_CSS: &str = include_str!("../../../viewer/dist/viewer/viewer.css");

pub fn serve(workspace: &FsWorkspace, port: u16) -> Result<(), Box<dyn Error>> {
    let address = format!("127.0.0.1:{port}");
    let server = Server::http(&address).map_err(|error| std::io::Error::other(error.to_string()))?;
    let audio_root = workspace.layout().audio_dir();
    println!("Lingo viewer  →  http://{address}\n\nPress Ctrl-C to stop.");

    for mut request in server.incoming_requests() {
        let path = request.url().split('?').next().unwrap_or("/").to_string();
        let result = match path.as_str() {
            "/" | "/index.html" => request.respond(text(INDEX_HTML, "text/html; charset=utf-8")),
            "/viewer/app.mjs" => request.respond(text(APP_MJS, "text/javascript; charset=utf-8")),
            "/viewer/commands.mjs" => request.respond(text(COMMANDS_MJS, "text/javascript; charset=utf-8")),
            "/viewer/state.mjs" => request.respond(text(STATE_MJS, "text/javascript; charset=utf-8")),
            "/viewer/viewer.css" => request.respond(text(VIEWER_CSS, "text/css; charset=utf-8")),
            "/api/view/state" => match view_state(workspace) {
                Ok(body) => request.respond(text(&body, "application/json; charset=utf-8")),
                Err(error) => request.respond(Response::from_string(error.to_string()).with_status_code(500)),
            },
            "/api/view/action" => {
                let mut body = String::new();
                let _ = request.as_reader().read_to_string(&mut body);
                let payload = view_action(workspace, &body);
                request.respond(text(&payload, "application/json; charset=utf-8"))
            }
            _ if path.starts_with("/audio/") => serve_audio(request, &audio_root, &path["/audio/".len()..]),
            _ => request.respond(Response::from_string("not found").with_status_code(404)),
        };
        if let Err(error) = result {
            eprintln!("viewer: {error}");
        }
    }
    Ok(())
}

/// Build the `GET /api/view/state` payload from the deck's library + config.
fn view_state(workspace: &FsWorkspace) -> Result<String, Box<dyn Error>> {
    let context = workspace.resolve()?;
    let config = read_config(&workspace.layout().config_file())?;

    let page = workspace.list_sentences(&SentenceQuery {
        selection: SentenceSelection::All,
        sort: SentenceSort::LibraryOrder,
        page: PageRequest { limit: 1_000_000, offset: 0 },
    })?;

    let sentences = page.sentences.iter().map(|s| {
        let audio = match s.audio() {
            Some(a) => json!({
                "path": format!("/{}", a.path().as_str()),
                "backend": a.backend().wire_name(),
                "voice": a.voice(),
                "hash": a.hash().as_str(),
            }),
            None => Value::Null,
        };
        let breakdown = s.breakdown()
            .map(serde_json::to_value)
            .transpose().unwrap_or(None)
            .unwrap_or_else(|| json!([]));
        json!({
            "id": s.id().as_str(),
            "collection": config.collection.title,
            "batchId": s.batch_id().map(|v| v.as_str()),
            "title": s.title().map(|v| v.as_str()),
            "section": s.section().map(|v| v.as_str()),
            "order": s.order().get(),
            "target": s.target().as_str(),
            "romanisation": s.romanisation().map(|v| v.as_str()),
            "english": s.english().map(|v| v.as_str()),
            "literal": s.literal().map(|v| v.as_str()),
            "register": s.register().map(|r| r.wire_name()),
            "status": s.status().wire_name(),
            "tags": s.tags().values(),
            "authority": serde_json::to_value(s.authority()).unwrap_or_else(|_| json!({})),
            "breakdown": breakdown,
            "audio": audio,
            "provenance": serde_json::to_value(s.provenance()).unwrap_or(Value::Null),
        })
    }).collect::<Vec<_>>();

    let state = json!({
        "workspace": {
            "name": config.collection.title,
            "language": context.profile.language().as_str(),
            "languageCode": context.profile.code().as_str(),
            "libraryPath": "library.db",
        },
        "config": {
            "display": { "lead": config.display.lead, "showSecondary": config.display.show_secondary },
            "audio": { "backend": config.audio.backend, "voice": "", "model": "" },
            "anki": { "deck": config.export.deck, "replace": false },
            "package": { "destination": config.package.destination, "format": "json" },
        },
        "sentences": sentences,
        "batches": batches_value(workspace)?,
        "words": [],
    });
    Ok(serde_json::to_string(&state)?)
}

/// Handle `POST /api/view/action`. The producer packet loop (extract/enrich) is
/// wired live; other mutations return a message pointing at the CLI. Always
/// returns a JSON object the UI understands: `{ prompt?, runId?, state?, message? }`.
fn view_action(workspace: &FsWorkspace, body: &str) -> String {
    let request: Value = serde_json::from_str(body).unwrap_or(Value::Null);
    let kind = request.get("kind").and_then(Value::as_str).unwrap_or("");
    let payload = request.get("payload").cloned().unwrap_or(Value::Null);
    let get = |key: &str| payload.get(key).and_then(Value::as_str).unwrap_or("").to_string();

    match run_action(workspace, kind, &payload, &get) {
        Ok(value) => value.to_string(),
        Err(error) => json!({ "message": format!("Action failed: {error}"), "state": state_value(workspace) }).to_string(),
    }
}

fn run_action(
    workspace: &FsWorkspace,
    kind: &str,
    payload: &Value,
    get: &dyn Fn(&str) -> String,
) -> Result<Value, Box<dyn Error>> {
    let prompts = HandlebarsPromptEngine::with_overrides(workspace.layout().prompts_dir());
    let section = |raw: &str| -> Option<SectionName> {
        let raw = raw.trim();
        if raw.is_empty() { None } else { SectionName::parse(raw.to_string()).ok() }
    };
    match kind {
        "extract.prepare" => {
            let deps = ExtractDeps { library: workspace, context: workspace, prompts: &prompts };
            let run_id = gen_run("extract")?;
            let prep = prepare_extract(&deps, PrepareExtract {
                run_id: run_id.clone(),
                raw: get("rawText"),
                collection: None,
                section: section(&get("section")),
                batch: batch_ref(payload)?,
            })?;
            Ok(json!({ "runId": prep.run_id.as_str(), "prompt": prep.packet, "batchId": prep.batch_id.as_ref().map(BatchId::as_str) }))
        }
        "extract.apply" => {
            let deps = ExtractDeps { library: workspace, context: workspace, prompts: &prompts };
            let batch_id = match get("batchId") { id if id.is_empty() => None, id => Some(BatchId::parse(id)?) };
            let report = apply_extract(&deps, ApplyExtract {
                run_id: RunId::parse(get("runId"))?,
                reply: get("reply"),
                collection: None,
                section: section(&get("section")),
                batch_id,
            })?;
            Ok(json!({ "message": format!("Created {} draft sentence(s)", report.created), "state": state_value(workspace) }))
        }
        "enrich.prepare" => {
            let deps = EnrichDeps { library: workspace, context: workspace, prompts: &prompts };
            let run_id = gen_run("enrich")?;
            // A selection of ids enriches exactly those (one bounded batch the
            // learner picks to fit the model's context); otherwise take the next
            // `limit` drafts in library order.
            let ids = payload_ids(payload);
            let (selection, limit) = if ids.is_empty() {
                (SentenceSelection::All, payload.get("limit").and_then(Value::as_u64).unwrap_or(20) as usize)
            } else {
                let count = ids.len();
                (SentenceSelection::Ids(ids), count)
            };
            let prep = prepare_enrich(&deps, PrepareEnrich { run_id: run_id.clone(), selection, limit, force: false })?;
            if prep.claimed == 0 {
                return Ok(json!({ "message": "No draft sentences to enrich in that selection.", "state": state_value(workspace) }));
            }
            Ok(json!({ "runId": run_id.as_str(), "prompt": prep.packet, "claimed": prep.claimed }))
        }
        "enrich.apply" => {
            let deps = EnrichDeps { library: workspace, context: workspace, prompts: &prompts };
            let report = apply_enrich(&deps, ApplyEnrich {
                run_id: RunId::parse(get("runId"))?,
                reply: get("reply"),
            })?;
            Ok(json!({ "message": format!("Enriched {} sentence(s)", report.updated), "state": state_value(workspace) }))
        }
        "import.preview" => import_preview(workspace, &PathBuf::from(get("path"))),
        "import.commit" => {
            let ids: HashSet<String> = payload
                .get("ids")
                .and_then(Value::as_array)
                .map(|array| array.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                .unwrap_or_default();
            import_commit(workspace, &PathBuf::from(get("path")), &ids)
        }
        "package.export" => package_export(workspace, payload),
        "batch.create" => batch_create(workspace, payload),
        "batch.list" => Ok(json!({ "batches": batches_value(workspace)? })),
        "batch.get" => {
            let batch = workspace.get_batch(&BatchId::parse(get("batchId"))?)?;
            Ok(json!({ "batch": batch_detail_json(&batch) }))
        }
        "organize.set-section" => set_section(workspace, payload),
        "organize.set-status" => set_status(workspace, payload),
        "config.save" => config_save(workspace, payload),
        _ => Ok(json!({
            "message": "Run the CLI command shown under ⌘ for this action.",
            "state": state_value(workspace),
        })),
    }
}

// ── package import (lingo.package-cards/v1 preview + selective commit) ──────

#[derive(serde::Deserialize)]
struct CardFileDto {
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    cards: Vec<CardDto>,
}

#[derive(serde::Deserialize)]
struct CardDto {
    id: Option<String>,
    target: String,
    romanisation: Option<String>,
    english: Option<String>,
    literal: Option<String>,
    register: Option<String>,
    #[serde(default)]
    tokens: Vec<CardTokenDto>,
    #[serde(default)]
    words: Vec<CardWordDto>,
    #[serde(default)]
    tags: Vec<String>,
    audio: Option<String>,
}

#[derive(serde::Deserialize)]
struct CardTokenDto {
    target: String,
    romanisation: Option<String>,
    word_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct CardWordDto {
    id: String,
    meaning: Option<String>,
    kind: Option<String>,
}

/// One parsed card plus the section it belongs to and a stable selection id.
struct CardEntry {
    select_id: String,
    section: Option<String>,
    card: CardDto,
}

/// Normalise a target for duplicate detection: trim + collapse internal
/// whitespace + lowercase. Devanagari matras live in U+09xx so they survive.
fn normalize_target(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Normalised targets already in the library, for duplicate detection.
fn library_targets(workspace: &FsWorkspace) -> HashSet<String> {
    workspace
        .list_sentences(&SentenceQuery {
            selection: SentenceSelection::All,
            sort: SentenceSort::LibraryOrder,
            page: PageRequest { limit: 1_000_000, offset: 0 },
        })
        .map(|page| page.sentences.iter().map(|s| normalize_target(s.target().as_str())).collect())
        .unwrap_or_default()
}

/// Read a `lingo.package-cards/v1` directory (`cards/*.json`) into ordered entries.
fn read_card_entries(dir: &Path) -> Result<Vec<CardEntry>, Box<dyn Error>> {
    let cards_dir = dir.join("cards");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&cards_dir)
        .map_err(|e| std::io::Error::other(format!("{}: {e}", cards_dir.display())))?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|x| x.to_str()) == Some("json"))
        .collect();
    files.sort();

    let mut entries = Vec::new();
    for (file_index, path) in files.iter().enumerate() {
        let file: CardFileDto = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        let section = file.subtitle;
        for (card_index, card) in file.cards.into_iter().enumerate() {
            let select_id =
                card.id.clone().unwrap_or_else(|| format!("{file_index}:{card_index}"));
            entries.push(CardEntry { select_id, section: section.clone(), card });
        }
    }
    Ok(entries)
}

/// `enriched` when the card carries a full translation + breakdown, else `draft`.
fn card_status(card: &CardDto) -> SentenceStatus {
    let full = !card.tokens.is_empty()
        && card.literal.is_some()
        && card.romanisation.is_some()
        && card.english.is_some();
    if full { SentenceStatus::Enriched } else { SentenceStatus::Draft }
}

/// Build a word-by-word breakdown from the card's tokens, glossed via its words.
/// Returns `None` (rather than failing the import) if any token can't be parsed.
fn card_breakdown(card: &CardDto) -> Option<TokenBreakdown> {
    if card.tokens.is_empty() {
        return None;
    }
    let glosses: HashMap<&str, &CardWordDto> =
        card.words.iter().map(|w| (w.id.as_str(), w)).collect();
    let mut items = Vec::with_capacity(card.tokens.len());
    for token in &card.tokens {
        let word = token.word_id.as_deref().and_then(|id| glosses.get(id).copied());
        let gloss_text = word
            .and_then(|w| w.meaning.clone())
            .filter(|m| !m.trim().is_empty())
            .or_else(|| token.romanisation.clone())
            .unwrap_or_else(|| token.target.clone());
        let surface = TargetText::parse(token.target.clone()).ok()?;
        let roman = token.romanisation.clone().and_then(|r| Romanisation::parse(r).ok());
        let gloss = Gloss::parse(gloss_text).ok()?;
        let kind = word.and_then(|w| w.kind.clone());
        items.push(BreakdownItem::new(surface, roman, gloss, kind));
    }
    TokenBreakdown::try_new(items).ok()
}

/// Convert one card into the importer's DTO, loading its audio clip if present.
fn card_to_import(dir: &Path, entry: &CardEntry) -> Result<ImportedSentence, Box<dyn Error>> {
    let card = &entry.card;
    let audio = match &card.audio {
        Some(rel) => {
            let asset = dir.join(rel);
            if asset.is_file() { Some(std::fs::read(asset)?) } else { None }
        }
        None => None,
    };
    Ok(ImportedSentence {
        section: entry.section.clone().map(SectionName::parse).transpose()?,
        target: TargetText::parse(card.target.clone())?,
        romanisation: card.romanisation.clone().map(Romanisation::parse).transpose()?,
        english: card.english.clone().map(Gloss::parse).transpose()?,
        literal: card.literal.clone().map(LiteralGloss::parse).transpose()?,
        register: card.register.as_deref().map(Register::parse).transpose()?,
        authority: FieldAuthoritySet::default(),
        tags: SentenceTags::try_from_values(card.tags.clone())?,
        breakdown: card_breakdown(card),
        status: card_status(card),
        source_item: card.id.clone(),
        audio,
    })
}

/// `import.preview`: scan the package, group by section, flag duplicates.
fn import_preview(workspace: &FsWorkspace, dir: &Path) -> Result<Value, Box<dyn Error>> {
    let entries = read_card_entries(dir)?;
    if entries.is_empty() {
        return Err("no importable cards found (expected cards/*.json)".into());
    }
    let existing = library_targets(workspace);
    let package_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("package").to_string();

    let mut order: Vec<String> = Vec::new();
    let mut groups: HashMap<String, Vec<Value>> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    let (mut total, mut duplicates) = (0usize, 0usize);

    for entry in &entries {
        let card = &entry.card;
        let norm = normalize_target(&card.target);
        let duplicate = existing.contains(&norm) || !seen.insert(norm);
        total += 1;
        if duplicate {
            duplicates += 1;
        }
        let section = entry.section.clone().unwrap_or_else(|| "Unsectioned".into());
        let row = json!({
            "id": entry.select_id,
            "target": card.target,
            "romanisation": card.romanisation,
            "english": card.english,
            "status": card_status(card).wire_name(),
            "hasAudio": card.audio.is_some(),
            "duplicate": duplicate,
        });
        if !groups.contains_key(&section) {
            order.push(section.clone());
        }
        groups.entry(section).or_default().push(row);
    }

    let group_values: Vec<Value> = order
        .into_iter()
        .map(|section| {
            let rows = groups.remove(&section).unwrap_or_default();
            json!({ "section": section, "count": rows.len(), "rows": rows })
        })
        .collect();

    Ok(json!({
        "preview": {
            "packageName": package_name,
            "path": dir.to_string_lossy(),
            "format": "package-cards/v1",
            "counts": { "total": total, "new": total - duplicates, "duplicate": duplicates },
            "groups": group_values,
        }
    }))
}

/// `import.commit`: import the selected cards, skipping any that already exist.
fn import_commit(
    workspace: &FsWorkspace,
    dir: &Path,
    ids: &HashSet<String>,
) -> Result<Value, Box<dyn Error>> {
    let entries = read_card_entries(dir)?;
    let existing = library_targets(workspace);
    let package_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("package").to_string();

    let mut sentences = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut skipped = 0usize;
    for entry in &entries {
        if !ids.contains(&entry.select_id) {
            continue;
        }
        let norm = normalize_target(&entry.card.target);
        if existing.contains(&norm) || !seen.insert(norm) {
            skipped += 1;
            continue;
        }
        sentences.push(card_to_import(dir, entry)?);
    }

    if sentences.is_empty() {
        return Ok(json!({
            "message": format!("Nothing imported ({skipped} duplicate(s) skipped)."),
            "state": state_value(workspace),
        }));
    }

    let files = workspace.audio_files();
    let report = import_package(
        &ImportDeps { library: workspace, context: workspace, audio_files: &files },
        ImportPackage { package_name, sentences },
    )?;
    let suffix = if skipped > 0 { format!(" ({skipped} duplicate(s) skipped)") } else { String::new() };
    Ok(json!({
        "message": format!("Imported {} sentence(s){}, {} audio.", report.imported, suffix, report.audio),
        "state": state_value(workspace),
    }))
}

fn gen_run(prefix: &str) -> Result<RunId, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    Ok(RunId::parse(format!("{prefix}-{millis:x}"))?)
}

/// Parse a payload `ids` array into typed sentence ids (bad ids are dropped).
fn payload_ids(payload: &Value) -> Vec<SentenceId> {
    payload
        .get("ids")
        .and_then(Value::as_array)
        .map(|array| array.iter().filter_map(Value::as_str).filter_map(|s| SentenceId::parse(s).ok()).collect())
        .unwrap_or_default()
}

fn trimmed(payload: &Value, key: &str) -> String {
    payload.get(key).and_then(Value::as_str).unwrap_or("").trim().to_string()
}

fn opt_section(raw: String) -> Result<Option<SectionName>, Box<dyn Error>> {
    if raw.is_empty() { Ok(None) } else { Ok(Some(SectionName::parse(raw)?)) }
}

/// Resolve the extract payload into a batch reference: `batchId` → append to an
/// existing batch; `batchName` → create a new one; neither → no batch (legacy).
fn batch_ref(payload: &Value) -> Result<Option<BatchRef>, Box<dyn Error>> {
    let existing = trimmed(payload, "batchId");
    if !existing.is_empty() {
        return Ok(Some(BatchRef::Existing(BatchId::parse(existing)?)));
    }
    let name = trimmed(payload, "batchName");
    if name.is_empty() {
        return Ok(None);
    }
    Ok(Some(BatchRef::New {
        name: BatchName::parse(name)?,
        default_title: opt_section(trimmed(payload, "defaultTitle"))?,
        default_subtitle: opt_section(trimmed(payload, "defaultSubtitle"))?,
    }))
}

/// `batch.create`: create an empty batch shell (raw text optional).
fn batch_create(workspace: &FsWorkspace, payload: &Value) -> Result<Value, Box<dyn Error>> {
    let context = workspace.resolve()?;
    let report = workspace.create_batch(CreateBatch {
        collection: context.default_collection.clone(),
        title: context.default_title.clone(),
        language: context.profile.code().clone(),
        name: BatchName::parse(trimmed(payload, "name"))?,
        default_title: opt_section(trimmed(payload, "defaultTitle"))?,
        default_subtitle: opt_section(trimmed(payload, "defaultSubtitle"))?,
        raw_text: trimmed(payload, "rawText"),
    })?;
    Ok(json!({ "batchId": report.batch.id().as_str(), "state": state_value(workspace) }))
}

/// The `batches` array for view_state / `batch.list` (no raw text — that's large).
fn batches_value(workspace: &FsWorkspace) -> Result<Value, Box<dyn Error>> {
    let page = workspace.list_batches(None)?;
    Ok(Value::Array(page.batches.iter().map(|b| json!({
        "id": b.batch.id().as_str(),
        "name": b.batch.name().as_str(),
        "defaultTitle": b.batch.default_title().map(SectionName::as_str),
        "defaultSubtitle": b.batch.default_subtitle().map(SectionName::as_str),
        "counts": { "draft": b.draft, "enriching": b.enriching, "enriched": b.enriched, "active": b.active, "total": b.total },
    })).collect()))
}

/// Full batch detail (incl. raw text) for `batch.get`.
fn batch_detail_json(batch: &Batch) -> Value {
    json!({
        "id": batch.id().as_str(),
        "name": batch.name().as_str(),
        "defaultTitle": batch.default_title().map(SectionName::as_str),
        "defaultSubtitle": batch.default_subtitle().map(SectionName::as_str),
        "rawText": batch.raw_text(),
    })
}

/// `organize.set-section`: assign the selected sentences to a section (the
/// "subtitle"). An empty name clears the section. Renaming a chapter is just
/// "select all in it, set the new name".
fn set_section(workspace: &FsWorkspace, payload: &Value) -> Result<Value, Box<dyn Error>> {
    let ids = payload_ids(payload);
    if ids.is_empty() {
        return Ok(json!({ "message": "No sentences selected.", "state": state_value(workspace) }));
    }
    let raw = payload.get("section").and_then(Value::as_str).unwrap_or("").trim().to_string();
    let section = if raw.is_empty() { None } else { Some(SectionName::parse(raw.clone())?) };
    let mut updated = 0usize;
    for id in ids {
        update_sentence(workspace, UpdateSentence {
            id,
            section: Some(section.clone()),
            target: None,
            romanisation: None,
            english: None,
            literal: None,
            register: None,
            tags: None,
            status: None,
            batch_id: None,
            title: None,
        })?;
        updated += 1;
    }
    let where_to = if raw.is_empty() { "no section".to_string() } else { format!("\"{raw}\"") };
    Ok(json!({ "message": format!("Moved {updated} sentence(s) to {where_to}."), "state": state_value(workspace) }))
}

/// `organize.set-status`: transition the selected sentences' lifecycle status
/// (e.g. confirm `enriched` → `active`, or send one back to `enriched`).
fn set_status(workspace: &FsWorkspace, payload: &Value) -> Result<Value, Box<dyn Error>> {
    let ids = payload_ids(payload);
    if ids.is_empty() {
        return Ok(json!({ "message": "No sentences selected.", "state": state_value(workspace) }));
    }
    let status = SentenceStatus::parse(payload.get("status").and_then(Value::as_str).unwrap_or("active"))?;
    let mut updated = 0usize;
    for id in ids {
        update_sentence(workspace, UpdateSentence {
            id,
            section: None,
            target: None,
            romanisation: None,
            english: None,
            literal: None,
            register: None,
            tags: None,
            status: Some(status),
            batch_id: None,
            title: None,
        })?;
        updated += 1;
    }
    Ok(json!({ "message": format!("Set {updated} sentence(s) to {}.", status.wire_name()), "state": state_value(workspace) }))
}

/// `config.save`: persist editable deck settings (title, display, audio, deck,
/// package destination) to `config.toml`.
fn config_save(workspace: &FsWorkspace, payload: &Value) -> Result<Value, Box<dyn Error>> {
    let config_file = workspace.layout().config_file();
    let apply = |key: &str, value: Option<String>| {
        if let Some(value) = value {
            let _ = set_config_value(&config_file, key, &value);
        }
    };
    let workspace_obj = payload.get("workspace").cloned().unwrap_or(Value::Null);
    let config_obj = payload.get("config").cloned().unwrap_or(Value::Null);
    apply("collection.title", workspace_obj.get("name").and_then(Value::as_str).map(str::to_string));
    apply("display.lead", config_obj.pointer("/display/lead").and_then(Value::as_str).map(str::to_string));
    apply("display.show_secondary", config_obj.pointer("/display/showSecondary").and_then(Value::as_bool).map(|b| b.to_string()));
    apply("audio.backend", config_obj.pointer("/audio/backend").and_then(Value::as_str).map(str::to_string));
    apply("export.deck", config_obj.pointer("/anki/deck").and_then(Value::as_str).map(str::to_string));
    apply("package.destination", config_obj.pointer("/package/destination").and_then(Value::as_str).map(str::to_string));
    Ok(json!({ "message": "Settings saved to config.toml.", "state": state_value(workspace) }))
}

/// `package.export`: write the library (selected ids, or all) to a portable
/// JSON package or a SQLite db copy on disk. The live `library.db` is always
/// the source of truth; this is the explicit "save out to package files" step.
fn package_export(workspace: &FsWorkspace, payload: &Value) -> Result<Value, Box<dyn Error>> {
    let ids = payload_ids(payload);
    let selection = if ids.is_empty() { SentenceSelection::All } else { SentenceSelection::Ids(ids) };
    let format = match payload.get("format").and_then(Value::as_str) {
        Some("db") => PackageFormat::Db,
        _ => PackageFormat::Json,
    };
    let destination = payload
        .get("destination")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    let publisher = PortablePackagePublisher;
    let report = package(
        &PackageDeps { library: workspace, context: workspace, publisher: &publisher },
        PackageRequest { selection, destination, format },
    )?;
    Ok(json!({
        "message": format!("Saved {} sentence(s) → {}", report.sentences, report.artifact.path.display()),
        "state": state_value(workspace),
    }))
}

/// The state payload as a JSON value (or null if it cannot be built).
fn state_value(workspace: &FsWorkspace) -> Value {
    view_state(workspace).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null)
}

fn serve_audio(request: tiny_http::Request, audio_root: &Path, relative: &str) -> std::io::Result<()> {
    if relative.contains("..") || relative.starts_with('/') {
        return request.respond(Response::from_string("bad path").with_status_code(400));
    }
    match std::fs::read(audio_root.join(relative)) {
        Ok(bytes) => request.respond(Response::from_data(bytes).with_header(header("Content-Type", "audio/mpeg"))),
        Err(_) => request.respond(Response::from_string("not found").with_status_code(404)),
    }
}

fn text(body: &str, content: &'static str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body).with_header(header("Content-Type", content))
}

fn header(name: &'static str, value: &'static str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}
