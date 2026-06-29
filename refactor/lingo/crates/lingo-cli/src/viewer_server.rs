use crate::studio::{self, StudioError};
use lingo_application::{ViewerCard, ViewerPlan};
use serde::Serialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

#[derive(Debug, Error)]
pub enum ViewerServerError {
    #[error("viewer server could not bind: {0}")]
    Bind(String),
    #[error("viewer response could not be encoded: {0}")]
    Json(#[from] serde_json::Error),
    #[error("viewer request failed: {0}")]
    Io(#[from] std::io::Error),
}

pub fn serve(
    port: u16,
    workspace_root: PathBuf,
    dist_root: PathBuf,
    plan: ViewerPlan,
) -> Result<(), ViewerServerError> {
    let server = Server::http(("127.0.0.1", port))
        .map_err(|error| ViewerServerError::Bind(error.to_string()))?;
    let session_json = serde_json::to_string(&ViewerSessionDto::from_plan(&plan))?;
    for request in server.incoming_requests() {
        route(request, &workspace_root, &dist_root, &session_json)?;
    }
    Ok(())
}

fn route(
    mut request: Request,
    workspace_root: &Path,
    dist_root: &Path,
    session_json: &str,
) -> Result<(), ViewerServerError> {
    let method = request.method().clone();
    let url = request.url().to_string();
    let (path, query) = match url.split_once('?') {
        Some((path, query)) => (path.to_string(), Some(query.to_string())),
        None => (url.clone(), None),
    };

    // Studio API: a small, local, write-capable surface over lingo-application.
    if let Some(endpoint) = path.strip_prefix("/api/studio/") {
        let body = if method == Method::Post {
            let mut body = String::new();
            request.as_reader().read_to_string(&mut body).ok();
            body
        } else {
            String::new()
        };
        let result = dispatch_studio(endpoint, &method, query.as_deref(), &body, workspace_root);
        return respond_studio(request, result);
    }

    if !matches!(method, Method::Get | Method::Head) {
        request.respond(with_content_type(
            Response::empty(StatusCode(405)),
            "text/plain; charset=utf-8",
        ))?;
        return Ok(());
    }

    let head_only = method == Method::Head;
    match path.as_str() {
        "/api/session" => respond_text(
            request,
            session_json,
            "application/json; charset=utf-8",
            head_only,
        )?,
        "/api/generated/sentences" => {
            respond_generated_sentences(request, workspace_root, head_only)?
        }
        "/" | "/index.html" => serve_static(request, dist_root, "index.html", head_only)?,
        _ => {
            if let Some(relative) = path.strip_prefix("/audio/") {
                serve_audio(request, workspace_root, relative, head_only)?;
            } else {
                serve_static(request, dist_root, path.trim_start_matches('/'), head_only)?;
            }
        }
    }
    Ok(())
}

fn serve_static(
    request: Request,
    dist_root: &Path,
    relative: &str,
    head_only: bool,
) -> Result<(), ViewerServerError> {
    if !is_safe_relative_path(relative) {
        request.respond(Response::empty(StatusCode(400)))?;
        return Ok(());
    }
    let path = dist_root.join(relative);
    let Some(content_type) = content_type_for(relative) else {
        request.respond(Response::empty(StatusCode(404)))?;
        return Ok(());
    };
    if head_only {
        let status = if path.is_file() {
            StatusCode(200)
        } else {
            StatusCode(404)
        };
        request.respond(with_content_type(Response::empty(status), content_type))?;
        return Ok(());
    }
    match fs::read(path) {
        Ok(bytes) => {
            request.respond(with_content_type(Response::from_data(bytes), content_type))?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            request.respond(Response::empty(StatusCode(404)))?
        }
        Err(error) => return Err(ViewerServerError::Io(error)),
    }
    Ok(())
}

fn respond_text(
    request: Request,
    body: &str,
    content_type: &str,
    head_only: bool,
) -> Result<(), ViewerServerError> {
    if head_only {
        request.respond(with_content_type(
            Response::empty(StatusCode(200)),
            content_type,
        ))?;
    } else {
        request.respond(with_content_type(Response::from_string(body), content_type))?;
    }
    Ok(())
}

fn serve_audio(
    request: Request,
    workspace_root: &Path,
    relative: &str,
    head_only: bool,
) -> Result<(), ViewerServerError> {
    if !is_safe_relative_path(relative) {
        request.respond(Response::empty(StatusCode(400)))?;
        return Ok(());
    }
    let path = workspace_root.join("audio").join(relative);
    if head_only {
        let status = if path.is_file() {
            StatusCode(200)
        } else {
            StatusCode(404)
        };
        request.respond(with_content_type(Response::empty(status), "audio/mpeg"))?;
        return Ok(());
    }
    match fs::read(path) {
        Ok(bytes) => {
            request.respond(with_content_type(Response::from_data(bytes), "audio/mpeg"))?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            request.respond(Response::empty(StatusCode(404)))?
        }
        Err(error) => return Err(ViewerServerError::Io(error)),
    }
    Ok(())
}

fn respond_generated_sentences(
    request: Request,
    workspace_root: &Path,
    head_only: bool,
) -> Result<(), ViewerServerError> {
    let dir = workspace_root.join("output").join("sentences");
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(file_name) = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let Some(stem) = path
                .file_stem()
                .and_then(|name| name.to_str())
                .map(str::to_string)
            else {
                continue;
            };
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let Ok(data) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            files.push(json!({ "file": file_name, "stem": stem, "data": data }));
        }
    }
    files.sort_by(|a, b| {
        a.get("file")
            .and_then(Value::as_str)
            .cmp(&b.get("file").and_then(Value::as_str))
    });

    // Per-sentence layer (sentences/<batch>__<item>.json): the canonical unit
    // the Sentences/Organize tabs render, ordered by each file's `order`.
    let mut sentences = Vec::new();
    if let Ok(entries) = fs::read_dir(workspace_root.join("sentences")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                sentences.push(value);
            }
        }
    }
    sentences.sort_by(|a, b| {
        let order = |value: &Value| value.get("order").and_then(Value::as_i64).unwrap_or(i64::MAX);
        order(a).cmp(&order(b))
    });

    if head_only {
        request.respond(with_content_type(
            Response::empty(StatusCode(200)),
            "application/json; charset=utf-8",
        ))?;
        return Ok(());
    }
    let body = json!({ "files": files, "sentences": sentences });
    let text = serde_json::to_string(&body)?;
    let response = Response::from_string(text).with_status_code(StatusCode(200));
    request.respond(with_content_type(
        response,
        "application/json; charset=utf-8",
    ))?;
    Ok(())
}

fn dispatch_studio(
    endpoint: &str,
    method: &Method,
    query: Option<&str>,
    body: &str,
    root: &Path,
) -> Result<Value, StudioError> {
    match (method, endpoint) {
        (Method::Get, "status") => studio::status(root),
        (Method::Get, "raw") => {
            if let Some(raw) = query_param(query, "raw") {
                studio::raw_detail(root, &raw)
            } else {
                studio::list_raw(root)
            }
        }
        (Method::Get, "voices") => studio::voices(root),
        (Method::Get, "config") => studio::config_get(root),
        (Method::Get, "batch") => {
            let batch = query_param(query, "batch").ok_or_else(|| StudioError {
                status: 400,
                message: "missing batch query parameter".to_string(),
                problems: Vec::new(),
            })?;
            studio::batch_detail(root, &batch)
        }
        (Method::Post, "raw") => studio::save_raw(root, body),
        (Method::Post, "meta") => studio::update_meta(root, body),
        (Method::Post, "import/prepare") => studio::import_prepare(root, body),
        (Method::Post, "import/apply") => studio::import_apply(root, body),
        (Method::Post, "build/prepare") => studio::build_prepare(root, body),
        (Method::Post, "build/apply") => studio::build_apply(root, body),
        (Method::Post, "check") => studio::check(root, body),
        (Method::Post, "audio") => studio::audio(root, body),
        (Method::Post, "package") => studio::package(root, body),
        (Method::Post, "package/browse") => studio::pick_folder(root),
        (Method::Post, "config") => studio::config_set(root, body),
        _ => Err(StudioError {
            status: 404,
            message: format!("unknown studio endpoint: {} {endpoint}", method.as_str()),
            problems: Vec::new(),
        }),
    }
}

fn respond_studio(
    request: Request,
    result: Result<Value, StudioError>,
) -> Result<(), ViewerServerError> {
    let (code, body) = match result {
        Ok(value) => (200u16, value),
        Err(error) => (error.status, error.to_json()),
    };
    let text = serde_json::to_string(&body)?;
    let response = Response::from_string(text).with_status_code(StatusCode(code));
    request.respond(with_content_type(
        response,
        "application/json; charset=utf-8",
    ))?;
    Ok(())
}

/// Reads a single query parameter, percent-decoding its value.
fn query_param(query: Option<&str>, key: &str) -> Option<String> {
    query?.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        (name == key).then(|| percent_decode(value))
    })
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
                match hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                    Some(decoded) => {
                        out.push(decoded);
                        index += 3;
                    }
                    None => {
                        out.push(bytes[index]);
                        index += 1;
                    }
                }
            }
            other => {
                out.push(other);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn is_safe_relative_path(relative: &str) -> bool {
    !relative.is_empty()
        && relative
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}

fn with_content_type<R: std::io::Read>(response: Response<R>, value: &str) -> Response<R> {
    match Header::from_bytes("Content-Type", value) {
        Ok(header) => response.with_header(header),
        Err(_) => response,
    }
}

fn content_type_for(relative: &str) -> Option<&'static str> {
    if relative.ends_with(".html") {
        Some("text/html; charset=utf-8")
    } else if relative.ends_with(".js") {
        Some("text/javascript; charset=utf-8")
    } else if relative.ends_with(".css") {
        Some("text/css; charset=utf-8")
    } else if relative.ends_with(".svg") {
        Some("image/svg+xml")
    } else if relative.ends_with(".json") {
        Some("application/json; charset=utf-8")
    } else if relative.ends_with(".png") {
        Some("image/png")
    } else if relative.ends_with(".jpg") || relative.ends_with(".jpeg") {
        Some("image/jpeg")
    } else if relative.ends_with(".webp") {
        Some("image/webp")
    } else if relative.ends_with(".woff2") {
        Some("font/woff2")
    } else {
        None
    }
}

#[derive(Serialize)]
struct ViewerSessionDto {
    lead: String,
    cards: Vec<ViewerCardDto>,
}

impl ViewerSessionDto {
    fn from_plan(plan: &ViewerPlan) -> Self {
        Self {
            lead: plan.lead.wire_name().to_string(),
            cards: plan.cards.iter().map(ViewerCardDto::from).collect(),
        }
    }
}

#[derive(Serialize)]
struct ViewerCardDto {
    id: String,
    lead: String,
    secondary: Option<String>,
    english: String,
    literal: String,
    register: String,
    audio_url: Option<String>,
}

impl From<&ViewerCard> for ViewerCardDto {
    fn from(card: &ViewerCard) -> Self {
        Self {
            id: card.id.to_string(),
            lead: card.primary.clone(),
            secondary: card.secondary.clone(),
            english: card.english.clone(),
            literal: card.literal.clone(),
            register: card.register.clone(),
            audio_url: card.audio_url.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_safe_relative_path;

    #[test]
    fn rejects_audio_path_traversal() {
        assert!(!is_safe_relative_path("../cards.json"));
        assert!(!is_safe_relative_path("batch/../../secret"));
        assert!(!is_safe_relative_path("batch//clip.mp3"));
        assert!(is_safe_relative_path("sentences/chapter-01/clip.mp3"));
    }
}
