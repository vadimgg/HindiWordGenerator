//! Studio API orchestration.
//!
//! Each function here behaves like a single CLI invocation: it discovers a fresh
//! `Composition` for the workspace, calls one or more `lingo-application` use
//! cases, and returns a JSON-serialisable value (or a `StudioError` carrying an
//! HTTP status). The viewer server (`viewer_server.rs`) is a thin transport that
//! reads the request body, calls one of these functions, and serialises the
//! result. No model is ever called; generation stays the manual packet loop.

use crate::commands::run_id;
use crate::composition::Composition;
use lingo_application::ports::{DeckContextProvider, WorkspaceStore};
use lingo_application::{
    ApplyBuild, ApplyImport, AudioCommand, AudioDeps, AudioError, AudioMode, BuildDeps, BuildError,
    CheckDeps, CheckRequest, ImportDeps, ImportError, PackageDeps, PackageError, PackageRequest,
    PrepareBuild, PrepareImport, StatusDeps, StatusRequest, apply_build, apply_import,
    check as run_check, package as run_package, prepare_build, prepare_import, status as run_status,
    synthesize_audio,
};
use lingo_domain::{
    AudioBackendId, BatchId, CardBatch, CardId, CheckState, DiagnosticCode, RawDocumentId, RunId,
    SourceBatch, SourceSubtitle, SourceTitle, WordKind,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A typed failure that maps onto an HTTP status + JSON body.
pub struct StudioError {
    pub status: u16,
    pub message: String,
    pub problems: Vec<String>,
}

impl StudioError {
    fn new(status: u16, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            problems: Vec::new(),
        }
    }
    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, message)
    }
    fn internal(message: impl Into<String>) -> Self {
        Self::new(500, message)
    }
    /// 422: the model reply did not pass local validation. Canonical data is
    /// untouched; `problems` is the fixable list the UI renders inline.
    fn unprocessable(message: impl Into<String>, problems: Vec<String>) -> Self {
        Self {
            status: 422,
            message: message.into(),
            problems,
        }
    }

    pub fn to_json(&self) -> Value {
        json!({ "ok": false, "message": self.message, "problems": self.problems })
    }
}

type StudioResult = Result<Value, StudioError>;

fn open(root: &Path) -> Result<Composition, StudioError> {
    Composition::discover(root)
        .map_err(|error| StudioError::internal(format!("workspace could not be opened: {error}")))
}

/* ── GET /api/studio/status ──────────────────────────────────────────────── */

pub fn status(root: &Path) -> StudioResult {
    let composition = open(root)?;
    let context = composition
        .workspace
        .resolve()
        .map_err(|error| StudioError::internal(error.to_string()))?;
    let report = run_status(
        &StatusDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
        },
        StatusRequest::default(),
    )
    .map_err(|error| StudioError::internal(error.to_string()))?;

    let mut seen = BTreeSet::new();
    let mut batches = Vec::new();
    let mut total_sentences = 0usize;

    for row in &report.rows {
        seen.insert(row.batch.clone());
        total_sentences += row.audio.total();
        let check_problem = match row.check {
            CheckState::Problems { errors, warnings } => errors > 0 || warnings > 0,
            CheckState::Clean | CheckState::NotRun => false,
        };
        let audio_problem = row.cards_present && !row.audio.is_complete();
        let check = match row.check {
            CheckState::Clean => "done",
            CheckState::Problems { errors, .. } if errors > 0 => "next",
            CheckState::Problems { .. } => "done",
            CheckState::NotRun if row.cards_present => "next",
            CheckState::NotRun => "wait",
        };
        let audio = if row.audio.is_complete() {
            "done"
        } else if row.audio.present() > 0 {
            "partial"
        } else if row.cards_present {
            "next"
        } else {
            "wait"
        };
        batches.push(json!({
            "id": row.batch.as_str(),
            "title": humanize(row.batch.as_str()),
            "stages": {
                "raw": "done",
                "import": if row.source_present { "done" } else { "next" },
                "build": if row.cards_present { "done" } else if row.source_present { "next" } else { "wait" },
                "check": check,
                "audio": audio,
            },
            "audio": { "done": row.audio.present(), "total": row.audio.total() },
            "problems": { "check": check_problem, "audio": audio_problem },
        }));
    }

    // Raw documents that have not been imported yet are batches at stage 1.
    for raw in &report.pending_raw {
        let id = raw
            .batch_hint
            .as_ref()
            .map(|b| b.as_str().to_string())
            .unwrap_or_else(|| raw.id.as_str().to_string());
        if seen.iter().any(|b| b.as_str() == id) {
            continue;
        }
        batches.push(json!({
            "id": id,
            "title": humanize(&id),
            "rawId": raw.id.as_str(),
            "stages": { "raw": "done", "import": "next", "build": "wait", "check": "wait", "audio": "wait" },
            "audio": { "done": 0, "total": 0 },
            "problems": { "check": false, "audio": false },
        }));
    }

    Ok(json!({
        "workspace": {
            "name": workspace_name(root),
            "language": context.profile.language().as_str(),
            "batches": batches.len(),
            "sentences": total_sentences,
        },
        "batches": batches,
    }))
}

/* ── GET /api/studio/batch?batch=ID ──────────────────────────────────────── */

pub fn batch_detail(root: &Path, batch: &str) -> StudioResult {
    let composition = open(root)?;
    let batch_id = BatchId::parse(batch).map_err(StudioError::bad_request_from)?;

    let source = composition.workspace.load_source(&batch_id).ok();
    let cards = composition.workspace.load_cards(&batch_id).ok();

    let title = cards
        .as_ref()
        .map(|c| c.title().as_str().to_string())
        .or_else(|| source.as_ref().map(|s| s.title().as_str().to_string()))
        .unwrap_or_else(|| humanize(batch));
    let subtitle = cards
        .as_ref()
        .and_then(|c| c.subtitle().map(|s| s.as_str().to_string()))
        .or_else(|| {
            source
                .as_ref()
                .and_then(|s| s.subtitle().map(|s| s.as_str().to_string()))
        })
        .unwrap_or_default();

    let sentences = source
        .as_ref()
        .map(|s| {
            s.items()
                .iter()
                .map(|item| {
                    json!({
                        "target": item.target().as_str(),
                        "romanisation": item.romanisation().map(|r| r.as_str()),
                        "english": item.english().as_str(),
                        "tags": item.tags().values(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let card_dtos = cards
        .as_ref()
        .map(|c| {
            c.cards()
                .iter()
                .map(|card| {
                    json!({
                        "id": card.id().to_string(),
                        "target": card.target().as_str(),
                        "romanisation": card.romanisation().map(|r| r.as_str()),
                        "english": card.english().as_str(),
                        "literal": card.literal().as_str(),
                        "register": card.register().wire_name(),
                        "words": words_detail(card),
                        "audioUrl": card.audio().map(|a| format!("/{}", a.relative_path().as_str())),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    // Best-effort raw text/id for the Raw stage of an already-imported batch.
    let (raw_text, raw_id) = find_raw_for_batch(&composition, &batch_id);

    Ok(json!({
        "id": batch_id.as_str(),
        "title": title,
        "subtitle": subtitle,
        "raw": raw_text,
        "rawId": raw_id,
        "sentences": sentences,
        "cards": card_dtos,
    }))
}

/* ── POST /api/studio/meta ───────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct UpdateMetaRequest {
    batch: String,
    title: String,
    subtitle: Option<String>,
}

/// Update a batch's title/subtitle in place, without re-importing. Rewrites the
/// title/subtitle on whichever canonical artifacts exist (source YAML and/or
/// card JSON), leaving items and cards untouched.
pub fn update_meta(root: &Path, body: &str) -> StudioResult {
    let request: UpdateMetaRequest = parse_body(body)?;
    let composition = open(root)?;
    let batch_id = BatchId::parse(request.batch).map_err(StudioError::bad_request_from)?;

    let title = SourceTitle::parse(request.title.trim().to_string())
        .map_err(StudioError::bad_request_from)?;
    let subtitle = request
        .subtitle
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(SourceSubtitle::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;

    let mut updated = false;

    if let Ok(source) = composition.workspace.load_source(&batch_id) {
        let next = SourceBatch::try_new(
            source.batch_id().clone(),
            title.clone(),
            subtitle.clone(),
            source.items().to_vec(),
        )
        .map_err(|error| StudioError::bad_request(error.to_string()))?;
        composition
            .workspace
            .replace_source(&next)
            .map_err(|error| StudioError::internal(error.to_string()))?;
        updated = true;
    }

    if let Ok(cards) = composition.workspace.load_cards(&batch_id) {
        let next = CardBatch::try_new(
            cards.batch_id().clone(),
            title.clone(),
            subtitle.clone(),
            cards.cards().to_vec(),
        )
        .map_err(|error| StudioError::bad_request(error.to_string()))?;
        composition
            .workspace
            .replace_cards(&next)
            .map_err(|error| StudioError::internal(error.to_string()))?;
        updated = true;
    }

    if !updated {
        return Err(StudioError::bad_request(format!(
            "nothing to update: batch {} has no imported source or cards yet",
            batch_id.as_str()
        )));
    }

    Ok(json!({
        "batch": batch_id.as_str(),
        "title": title.as_str(),
        "subtitle": subtitle.as_ref().map(SourceSubtitle::as_str),
    }))
}

/* ── GET /api/studio/raw ─────────────────────────────────────────────────── */

pub fn list_raw(root: &Path) -> StudioResult {
    let composition = open(root)?;
    let raw = composition
        .workspace
        .list_raw()
        .map_err(|error| StudioError::internal(error.to_string()))?;
    let files = raw
        .into_iter()
        .map(|summary| {
            json!({
                "id": summary.id.as_str(),
                "batch": summary.batch_hint.as_ref().map(|b| b.as_str()),
                "path": summary.relative_path.to_string_lossy(),
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "files": files }))
}

/* ── POST /api/studio/raw ────────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct SaveRawRequest {
    batch: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    text: String,
    overwrite: Option<bool>,
}

pub fn save_raw(root: &Path, body: &str) -> StudioResult {
    let request: SaveRawRequest = parse_body(body)?;
    if request.text.trim().is_empty() {
        return Err(StudioError::bad_request("raw text is empty"));
    }
    let slug = slug(&request.batch.unwrap_or_default());
    let slug = if slug.is_empty() {
        slug_from_first_line(&request.text)
    } else {
        slug
    };
    if slug.is_empty() {
        return Err(StudioError::bad_request(
            "could not derive a batch id from the input",
        ));
    }
    let raw_dir = root.join("raw");
    std::fs::create_dir_all(&raw_dir)
        .map_err(|error| StudioError::internal(format!("could not create raw/: {error}")))?;
    let file = raw_dir.join(format!("{slug}.md"));
    if file.exists() && !request.overwrite.unwrap_or(false) {
        return Err(StudioError::new(
            409,
            format!("raw/{slug}.md already exists; choose the existing batch or re-import it"),
        ));
    }
    std::fs::write(&file, request.text.as_bytes()).map_err(|error| {
        StudioError::internal(format!("could not write {}: {error}", file.display()))
    })?;
    Ok(json!({
        "rawId": slug,
        "batch": slug,
        "path": format!("raw/{slug}.md"),
        "title": request.title,
        "subtitle": request.subtitle,
    }))
}

pub fn raw_detail(root: &Path, raw: &str) -> StudioResult {
    let composition = open(root)?;
    let id = RawDocumentId::parse(raw).map_err(StudioError::bad_request_from)?;
    let doc = composition.workspace.load_raw(&id).map_err(|error| {
        StudioError::bad_request(format!("raw document {raw} not found: {error}"))
    })?;
    let batch = composition
        .workspace
        .list_raw()
        .ok()
        .and_then(|raws| {
            raws.into_iter()
                .find(|summary| summary.id == id)
                .and_then(|summary| summary.batch_hint.map(|batch| batch.as_str().to_string()))
        })
        .unwrap_or_else(|| id.as_str().to_string());
    Ok(json!({
        "id": id.as_str(),
        "batch": batch,
        "title": humanize(&batch),
        "subtitle": "",
        "text": doc.text(),
    }))
}

/* ── POST /api/studio/import/prepare ─────────────────────────────────────── */

#[derive(Deserialize)]
pub struct PrepareImportRequest {
    #[serde(rename = "rawId")]
    raw_id: Option<String>,
    batch: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
}

pub fn import_prepare(root: &Path, body: &str) -> StudioResult {
    let request: PrepareImportRequest = parse_body(body)?;
    let composition = open(root)?;
    let raws = composition
        .workspace
        .list_raw()
        .map_err(|error| StudioError::internal(error.to_string()))?;

    let summary = match &request.raw_id {
        Some(raw_id) => {
            let id = RawDocumentId::parse(raw_id).map_err(StudioError::bad_request_from)?;
            raws.into_iter().find(|s| s.id == id).ok_or_else(|| {
                StudioError::bad_request(format!("raw document {raw_id} not found"))
            })?
        }
        None => {
            // Prefer a raw doc matching the requested batch (so re-import targets the
            // right source); otherwise fall back to the first un-imported file.
            let wanted = request
                .batch
                .as_deref()
                .and_then(|batch| BatchId::parse(batch).ok());
            let matched = wanted.as_ref().and_then(|batch| {
                raws.iter()
                    .find(|s| {
                        s.batch_hint.as_ref() == Some(batch) || s.id.as_str() == batch.as_str()
                    })
                    .cloned()
            });
            match matched {
                Some(summary) => summary,
                None => raws
                    .into_iter()
                    .next()
                    .ok_or_else(|| StudioError::bad_request("no raw files found under raw/"))?,
            }
        }
    };

    let batch = match request.batch {
        Some(value) => BatchId::parse(value).map_err(StudioError::bad_request_from)?,
        None => match summary.batch_hint.clone() {
            Some(batch) => batch,
            None => BatchId::parse(summary.id.as_str()).map_err(StudioError::bad_request_from)?,
        },
    };
    let title = SourceTitle::parse(
        request
            .title
            .unwrap_or_else(|| humanize(summary.id.as_str())),
    )
    .map_err(StudioError::bad_request_from)?;
    let subtitle = request
        .subtitle
        .filter(|s| !s.trim().is_empty())
        .map(SourceSubtitle::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;
    let id = run_id("import", &batch).map_err(|error| StudioError::internal(error.to_string()))?;

    let deps = ImportDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let preparation = prepare_import(
        &deps,
        PrepareImport {
            run_id: id.clone(),
            raw: summary.id,
            batch: batch.clone(),
            title,
            subtitle,
            journal: true,
        },
    )
    .map_err(import_error)?;

    Ok(json!({ "runId": id.as_str(), "packet": preparation.packet, "batch": batch.as_str() }))
}

/* ── POST /api/studio/import/apply ───────────────────────────────────────── */

#[derive(Deserialize)]
pub struct ApplyRequest {
    #[serde(rename = "runId")]
    run_id: String,
    reply: String,
}

pub fn import_apply(root: &Path, body: &str) -> StudioResult {
    let request: ApplyRequest = parse_body(body)?;
    if request.reply.trim().is_empty() {
        return Err(StudioError::bad_request(
            "reply is empty; canonical source unchanged",
        ));
    }
    let composition = open(root)?;
    let id = RunId::parse(request.run_id).map_err(StudioError::bad_request_from)?;
    let deps = ImportDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let report = apply_import(
        &deps,
        ApplyImport {
            run_id: id,
            reply: request.reply,
        },
    )
    .map_err(import_error)?;
    Ok(json!({ "ok": true, "batch": report.batch.as_str(), "items": report.items }))
}

/* ── POST /api/studio/build/prepare ──────────────────────────────────────── */

#[derive(Deserialize)]
pub struct PrepareBuildRequest {
    batch: Option<String>,
}

pub fn build_prepare(root: &Path, body: &str) -> StudioResult {
    let request: PrepareBuildRequest = parse_body(body)?;
    let composition = open(root)?;
    let batch = match request.batch {
        Some(value) => BatchId::parse(value).map_err(StudioError::bad_request_from)?,
        None => next_pending_build(&composition)?,
    };
    let id = run_id("build", &batch).map_err(|error| StudioError::internal(error.to_string()))?;
    let deps = BuildDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let preparation = prepare_build(
        &deps,
        PrepareBuild {
            run_id: id.clone(),
            batch: batch.clone(),
            journal: true,
        },
    )
    .map_err(build_error)?;
    Ok(json!({ "runId": id.as_str(), "packet": preparation.packet, "batch": batch.as_str() }))
}

/* ── POST /api/studio/build/apply ────────────────────────────────────────── */

pub fn build_apply(root: &Path, body: &str) -> StudioResult {
    let request: ApplyRequest = parse_body(body)?;
    if request.reply.trim().is_empty() {
        return Err(StudioError::bad_request(
            "reply is empty; canonical cards unchanged",
        ));
    }
    let composition = open(root)?;
    let id = RunId::parse(request.run_id).map_err(StudioError::bad_request_from)?;
    let deps = BuildDeps {
        workspace: &composition.workspace,
        context: &composition.workspace,
        prompts: &composition.prompts,
        runs: &composition.runs,
    };
    let report = apply_build(
        &deps,
        ApplyBuild {
            run_id: id,
            reply: request.reply,
        },
    )
    .map_err(build_error)?;
    Ok(json!({
        "ok": true,
        "batch": report.batch.as_str(),
        "cards": report.cards,
        "warnings": report.validation.warning_count(),
    }))
}

/* ── POST /api/studio/check ──────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct CheckBody {
    batch: Option<String>,
}

pub fn check(root: &Path, body: &str) -> StudioResult {
    let request: CheckBody = parse_body(body)?;
    let composition = open(root)?;
    let batch = request
        .batch
        .map(BatchId::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;
    let report = run_check(
        &CheckDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
        },
        CheckRequest { batch },
    )
    .map_err(|error| StudioError::internal(error.to_string()))?;

    let batches = report
        .batches
        .iter()
        .map(|checked| {
            let diagnostics = checked
                .report
                .diagnostics()
                .iter()
                .map(|d| {
                    json!({
                        "severity": d.severity().wire_name(),
                        "code": d.code().wire_name(),
                        "message": d.message(),
                        "audioWarning": d.code() == DiagnosticCode::MissingAudio,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "batch": checked.batch.as_str(),
                "clean": checked.report.is_clean(),
                "errors": checked.report.error_count(),
                "warnings": checked.report.warning_count(),
                "diagnostics": diagnostics,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({ "clean": report.is_clean(), "batches": batches }))
}

/* ── GET /api/studio/voices ──────────────────────────────────────────────── */

pub fn voices(root: &Path) -> StudioResult {
    let composition = open(root)?;
    let context = composition
        .workspace
        .resolve()
        .map_err(|error| StudioError::internal(error.to_string()))?;
    let Some(name) = context.audio.elevenlabs_key_env.as_deref() else {
        return Err(StudioError::bad_request(
            "ElevenLabs API key is not configured; set [audio.elevenlabs].api_key",
        ));
    };
    let key = crate::secrets::resolve(composition.workspace.layout().root().path(), name)
        .ok_or_else(|| {
            StudioError::bad_request(format!(
                "ElevenLabs API key not set — add it in the Audio stage or export ${name}"
            ))
        })?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| StudioError::internal(error.to_string()))?;
    let response = client
        .get("https://api.elevenlabs.io/v1/voices")
        .header("xi-api-key", &key)
        .send()
        .map_err(|error| StudioError::bad_request(format!("ElevenLabs request failed: {error}")))?;
    let http = response.status();
    let text = response.text().unwrap_or_default();
    if !http.is_success() {
        return Err(StudioError::bad_request(format!(
            "ElevenLabs returned HTTP {}: {}",
            http.as_u16(),
            text.trim()
        )));
    }
    let parsed: VoicesResponse =
        serde_json::from_str(&text).map_err(|error| StudioError::internal(error.to_string()))?;
    let voices = parsed
        .voices
        .into_iter()
        .map(|v| json!({ "id": v.voice_id, "name": v.name, "category": v.category }))
        .collect::<Vec<_>>();

    // The real free-tier gate is the account plan, not any per-voice flag: on the
    // free plan the API rejects ElevenLabs' default/library voices with HTTP 402.
    // Surface the plan tier so the UI can warn honestly.
    let tier = client
        .get("https://api.elevenlabs.io/v1/user/subscription")
        .header("xi-api-key", &key)
        .send()
        .ok()
        .filter(|response| response.status().is_success())
        .and_then(|response| response.text().ok())
        .and_then(|text| serde_json::from_str::<SubscriptionDto>(&text).ok())
        .map(|subscription| subscription.tier);

    Ok(json!({ "voices": voices, "tier": tier }))
}

/* ── POST /api/studio/audio ──────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct AudioBody {
    batch: Option<String>,
    backend: Option<String>,
    voice: Option<String>,
    force: Option<bool>,
    #[serde(default)]
    cards: Vec<String>,
}

pub fn audio(root: &Path, body: &str) -> StudioResult {
    let request: AudioBody = parse_body(body)?;
    let composition = open(root)?;
    let backend = request
        .backend
        .as_deref()
        .map(AudioBackendId::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;
    let voice = request.voice.filter(|v| !v.trim().is_empty());
    if voice.is_some() && backend == Some(AudioBackendId::Gtts) {
        return Err(StudioError::bad_request(
            "a voice can only be used with the ElevenLabs backend",
        ));
    }
    let backend = if voice.is_some() {
        Some(AudioBackendId::ElevenLabs)
    } else {
        backend
    };
    let batch = request
        .batch
        .map(BatchId::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;
    let cards = request
        .cards
        .iter()
        .map(|id| CardId::parse(id))
        .collect::<Result<Vec<_>, _>>()
        .map_err(StudioError::bad_request_from)?;
    // An explicit selection means "regenerate exactly these", regardless of the
    // force flag the whole-batch path uses.
    let mode = if !cards.is_empty() || request.force.unwrap_or(false) {
        AudioMode::ReplaceAll
    } else {
        AudioMode::MissingOnly
    };

    let report = synthesize_audio(
        &AudioDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            synthesizer: &composition.audio,
        },
        AudioCommand {
            batch,
            mode,
            backend,
            elevenlabs_voice: voice,
            cards,
        },
    )
    .map_err(audio_error)?;

    let batches = report
        .batches
        .iter()
        .map(|b| {
            let total = b.cards.len();
            json!({ "batch": b.batch.as_str(), "done": total, "total": total })
        })
        .collect::<Vec<_>>();
    Ok(json!({
        "updated": report.counts.updated,
        "skipped": report.counts.skipped,
        "batches": batches,
    }))
}

/* ── POST /api/studio/package ────────────────────────────────────────────── */

#[derive(Deserialize)]
pub struct PackageBody {
    destination: Option<String>,
    batch: Option<String>,
}

/// Publish a portable `lingo.package/v1` folder — the same operation as
/// `lingo package --dest <DEST>`. This is the Grasp-compatible export format;
/// there is no separate Grasp build path.
pub fn package(root: &Path, body: &str) -> StudioResult {
    let request: PackageBody = parse_body(body)?;
    let destination = request
        .destination
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| StudioError::bad_request("a destination folder is required"))?;
    let destination = expand_tilde(destination);
    let batch = request
        .batch
        .filter(|value| !value.trim().is_empty())
        .map(BatchId::parse)
        .transpose()
        .map_err(StudioError::bad_request_from)?;

    let composition = open(root)?;
    let report = run_package(
        &PackageDeps {
            workspace: &composition.workspace,
            context: &composition.workspace,
            publisher: &composition.package,
        },
        PackageRequest {
            batch,
            destination: Some(destination),
        },
    )
    .map_err(package_error)?;

    Ok(json!({
        "path": report.artifact.path.to_string_lossy(),
        "files": report.artifact.files,
        "bytes": report.artifact.bytes,
        "batches": report.batches,
        "cards": report.cards,
    }))
}

/* ── POST /api/studio/package/browse ─────────────────────────────────────── */

/// Open a native folder picker on the machine running the viewer server and
/// return the chosen absolute path. The server is always local, so the dialog
/// appears on the user's own desktop. macOS only — elsewhere the user types the
/// path directly.
pub fn pick_folder(_root: &Path) -> StudioResult {
    #[cfg(target_os = "macos")]
    {
        let script =
            "POSIX path of (choose folder with prompt \"Choose a folder to export the package into\")";
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|error| {
                StudioError::internal(format!("could not open the folder picker: {error}"))
            })?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(json!({ "path": path }));
        }
        // AppleScript exits non-zero with error -128 when the user cancels.
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("-128") || stderr.contains("User canceled") {
            return Ok(json!({ "path": Value::Null, "cancelled": true }));
        }
        Err(StudioError::internal(format!(
            "folder picker failed: {}",
            stderr.trim()
        )))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = _root;
        Err(StudioError::bad_request(
            "the folder picker is only available on macOS; type the destination path instead",
        ))
    }
}

/// Expand a leading `~` to the user's home directory, matching how a shell would
/// interpret the path the user typed into the destination field.
fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

fn package_error(error: PackageError) -> StudioError {
    match error {
        PackageError::MissingAudio(card) => StudioError::bad_request(format!(
            "card {card} has no audio yet; finish the Audio stage before exporting"
        )),
        PackageError::EmptySelection => {
            StudioError::bad_request("no card batches are ready to export")
        }
        other => StudioError::internal(other.to_string()),
    }
}

/* ── GET / POST /api/studio/config ───────────────────────────────────────── */

pub fn config_get(root: &Path) -> StudioResult {
    let composition = open(root)?;
    let context = composition
        .workspace
        .resolve()
        .map_err(|error| StudioError::internal(error.to_string()))?;
    let key_present = context
        .audio
        .elevenlabs_key_env
        .as_deref()
        .is_some_and(|name| {
            crate::secrets::present(composition.workspace.layout().root().path(), name)
        });
    Ok(json!({
        "display": {
            "lead": context.display.lead().wire_name(),
            "showSecondary": context.display.show_secondary(),
        },
        "audio": {
            "backend": context.audio.primary.wire_name(),
            "fallback": context.audio.fallback.map(|f| f.wire_name()),
            "voice": context.audio.elevenlabs_voice,
            "model": context.audio.elevenlabs_model,
            "keyEnv": context.audio.elevenlabs_key_env,
            "keyPresent": key_present,
        },
    }))
}

#[derive(Deserialize)]
pub struct ConfigBody {
    lead: Option<String>,
    #[serde(rename = "showSecondary")]
    show_secondary: Option<bool>,
    backend: Option<String>,
    voice: Option<String>,
    #[serde(rename = "elevenlabsApiKey")]
    elevenlabs_api_key: Option<String>,
}

pub fn config_set(root: &Path, body: &str) -> StudioResult {
    let request: ConfigBody = parse_body(body)?;
    if let Some(lead) = request.lead.as_deref() {
        if lead != "romanisation" && lead != "target" {
            return Err(StudioError::bad_request(
                "lead must be 'romanisation' or 'target'",
            ));
        }
    }
    if let Some(backend) = request.backend.as_deref() {
        AudioBackendId::parse(backend).map_err(StudioError::bad_request_from)?;
    }
    let composition = open(root)?;
    let root_path = composition.workspace.layout().root().path();
    let config = composition.workspace.layout().config_file();

    // The ElevenLabs key never lands in config.toml. Store it in the gitignored
    // secrets file and make sure config.toml references that env var name.
    if let Some(key) = request
        .elevenlabs_api_key
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let context = composition
            .workspace
            .resolve()
            .map_err(|error| StudioError::internal(error.to_string()))?;
        let env_name = context
            .audio
            .elevenlabs_key_env
            .clone()
            .unwrap_or_else(|| "ELEVENLABS_API_KEY".to_string());
        crate::secrets::store(root_path, &env_name, key)
            .map_err(|error| StudioError::internal(format!("could not store the API key: {error}")))?;
        if context.audio.elevenlabs_key_env.is_none() {
            ensure_key_env(&config, &env_name)?;
        }
    }

    apply_config(&config, &request)?;
    config_get(root)
}

/// Ensure `[audio.elevenlabs].api_key` references `env:NAME` so the resolver
/// knows which environment variable / stored secret to read.
fn ensure_key_env(config: &Path, env_name: &str) -> Result<(), StudioError> {
    use toml::Value as Toml;
    let text = std::fs::read_to_string(config)
        .map_err(|error| StudioError::internal(format!("could not read config.toml: {error}")))?;
    let mut value: Toml =
        toml::from_str(&text).map_err(|error| StudioError::internal(error.to_string()))?;
    let root = value
        .as_table_mut()
        .ok_or_else(|| StudioError::internal("config.toml root is not a table"))?;
    let audio = root
        .entry("audio")
        .or_insert_with(|| Toml::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| StudioError::internal("[audio] is not a table"))?;
    let elevenlabs = audio
        .entry("elevenlabs")
        .or_insert_with(|| Toml::Table(Default::default()))
        .as_table_mut()
        .ok_or_else(|| StudioError::internal("[audio.elevenlabs] is not a table"))?;
    elevenlabs.insert("api_key".into(), Toml::String(format!("env:{env_name}")));
    let rendered =
        toml::to_string_pretty(&value).map_err(|error| StudioError::internal(error.to_string()))?;
    std::fs::write(config, rendered)
        .map_err(|error| StudioError::internal(format!("could not write config.toml: {error}")))?;
    Ok(())
}

fn apply_config(config: &Path, request: &ConfigBody) -> Result<(), StudioError> {
    use toml::Value as Toml;
    let text = std::fs::read_to_string(config)
        .map_err(|error| StudioError::internal(format!("could not read config.toml: {error}")))?;
    let mut value: Toml =
        toml::from_str(&text).map_err(|error| StudioError::internal(error.to_string()))?;
    let root = value
        .as_table_mut()
        .ok_or_else(|| StudioError::internal("config.toml root is not a table"))?;

    if request.lead.is_some() || request.show_secondary.is_some() {
        let display = root
            .entry("display")
            .or_insert_with(|| Toml::Table(Default::default()))
            .as_table_mut()
            .ok_or_else(|| StudioError::internal("[display] is not a table"))?;
        if let Some(lead) = &request.lead {
            display.insert("lead".into(), Toml::String(lead.clone()));
        }
        if let Some(secondary) = request.show_secondary {
            display.insert("show_secondary".into(), Toml::Boolean(secondary));
        }
    }
    if request.backend.is_some() || request.voice.is_some() {
        let audio = root
            .entry("audio")
            .or_insert_with(|| Toml::Table(Default::default()))
            .as_table_mut()
            .ok_or_else(|| StudioError::internal("[audio] is not a table"))?;
        if let Some(backend) = &request.backend {
            audio.insert("backend".into(), Toml::String(backend.clone()));
        }
        if let Some(voice) = &request.voice {
            let elevenlabs = audio
                .entry("elevenlabs")
                .or_insert_with(|| Toml::Table(Default::default()))
                .as_table_mut()
                .ok_or_else(|| StudioError::internal("[audio.elevenlabs] is not a table"))?;
            elevenlabs.insert("voice".into(), Toml::String(voice.clone()));
        }
    }
    let rendered =
        toml::to_string_pretty(&value).map_err(|error| StudioError::internal(error.to_string()))?;
    std::fs::write(config, rendered)
        .map_err(|error| StudioError::internal(format!("could not write config.toml: {error}")))?;
    Ok(())
}

/* ── helpers ─────────────────────────────────────────────────────────────── */

fn parse_body<T: for<'de> Deserialize<'de>>(body: &str) -> Result<T, StudioError> {
    let body = if body.trim().is_empty() { "{}" } else { body };
    serde_json::from_str(body)
        .map_err(|error| StudioError::bad_request(format!("invalid request body: {error}")))
}

fn next_pending_build(composition: &Composition) -> Result<BatchId, StudioError> {
    let completed = composition
        .workspace
        .list_card_batches()
        .map_err(|error| StudioError::internal(error.to_string()))?
        .into_iter()
        .collect::<BTreeSet<_>>();
    composition
        .workspace
        .list_source_batches()
        .map_err(|error| StudioError::internal(error.to_string()))?
        .into_iter()
        .find(|batch| !completed.contains(batch))
        .ok_or_else(|| StudioError::bad_request("no source batch needs a build"))
}

fn find_raw_for_batch(composition: &Composition, batch: &BatchId) -> (String, Option<String>) {
    let Ok(raws) = composition.workspace.list_raw() else {
        return (String::new(), None);
    };
    let Some(summary) = raws
        .into_iter()
        .find(|s| s.batch_hint.as_ref() == Some(batch) || s.id.as_str() == batch.as_str())
    else {
        return (String::new(), None);
    };
    let id = summary.id.as_str().to_string();
    let text = composition
        .workspace
        .load_raw(&summary.id)
        .map(|doc| doc.text().to_string())
        .unwrap_or_default();
    (text, Some(id))
}

/// Structured per-word detail for the Build review, mirroring the shape the
/// Sentences tab renders so the two card surfaces look the same.
fn words_detail(card: &lingo_domain::Card) -> Vec<Value> {
    card.words()
        .iter()
        .map(|word| {
            json!({
                "target": word.target().as_str(),
                "romanisation": word.romanisation().map(|r| r.as_str()),
                "meaning": word.meaning().as_str(),
                "kind": word_kind_label(word.kind()),
            })
        })
        .collect()
}

const fn word_kind_label(kind: WordKind) -> &'static str {
    match kind {
        WordKind::Noun => "noun",
        WordKind::Verb => "verb",
        WordKind::Adjective => "adj",
        WordKind::Adverb => "adv",
        WordKind::Pronoun => "pron",
        WordKind::Postposition => "postp",
        WordKind::Particle => "part",
        WordKind::Conjunction => "conj",
        WordKind::Interjection => "interj",
        WordKind::Numeral => "num",
        WordKind::ProperNoun => "proper",
        WordKind::Other => "other",
    }
}

fn workspace_name(root: &Path) -> String {
    root.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("workspace")
        .to_string()
}

fn humanize(id: &str) -> String {
    id.split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn slug(value: &str) -> String {
    let mut result = String::new();
    let mut separator = false;
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
            separator = false;
        } else if !separator && !result.is_empty() {
            result.push('-');
            separator = true;
        }
    }
    result.trim_end_matches('-').to_string()
}

fn slug_from_first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(slug)
        .unwrap_or_default()
}

impl StudioError {
    fn bad_request_from(error: impl std::fmt::Display) -> Self {
        Self::bad_request(error.to_string())
    }
}

/// Map import use-case errors to user-facing studio errors.
fn import_error(error: ImportError) -> StudioError {
    match error {
        ImportError::InvalidDraft(message) => {
            StudioError::unprocessable("reply rejected — input/ unchanged", vec![message])
        }
        ImportError::Prompt(failure) => StudioError::unprocessable(
            "reply rejected — input/ unchanged",
            vec![failure.to_string()],
        ),
        ImportError::Run(_) => {
            StudioError::bad_request("prepared run expired; re-prepare the packet")
        }
        ImportError::ProfileChanged { .. } => StudioError::bad_request(
            "this deck's language changed since the packet was generated; re-prepare the packet",
        ),
        other => StudioError::internal(other.to_string()),
    }
}

/// Map build use-case errors to user-facing studio errors.
fn build_error(error: BuildError) -> StudioError {
    match error {
        BuildError::InvalidDraft(message) => {
            StudioError::unprocessable("reply rejected — output/ unchanged", vec![message])
        }
        BuildError::Prompt(failure) => StudioError::unprocessable(
            "reply rejected — output/ unchanged",
            vec![failure.to_string()],
        ),
        BuildError::ValidationFailed(count) => StudioError::unprocessable(
            "reply rejected — output/ unchanged",
            vec![format!(
                "{count} card(s) failed deterministic validation; open Check for the specific diagnostics"
            )],
        ),
        BuildError::Run(_) => {
            StudioError::bad_request("prepared run expired; re-prepare the packet")
        }
        BuildError::ProfileChanged { .. } => StudioError::bad_request(
            "this deck's language changed since the packet was generated; re-prepare the packet",
        ),
        other => StudioError::internal(other.to_string()),
    }
}

fn audio_error(error: AudioError) -> StudioError {
    match error {
        AudioError::Synthesis(failure) => StudioError::bad_request(failure.to_string()),
        other => StudioError::internal(other.to_string()),
    }
}

#[derive(Deserialize)]
struct VoicesResponse {
    voices: Vec<VoiceDto>,
}

#[derive(Deserialize)]
struct VoiceDto {
    voice_id: String,
    name: String,
    category: Option<String>,
}

#[derive(Deserialize)]
struct SubscriptionDto {
    tier: String,
}
