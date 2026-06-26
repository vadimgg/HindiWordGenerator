use lingo_application::{ViewerCard, ViewerPlan};
use serde::Serialize;
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
    request: Request,
    workspace_root: &Path,
    dist_root: &Path,
    session_json: &str,
) -> Result<(), ViewerServerError> {
    let method = request.method().clone();
    if !matches!(method, Method::Get | Method::Head) {
        request.respond(with_content_type(
            Response::empty(StatusCode(405)),
            "text/plain; charset=utf-8",
        ))?;
        return Ok(());
    }

    let path = request.url().split('?').next().unwrap_or("/").to_string();
    let head_only = method == Method::Head;
    match path.as_str() {
        "/api/session" => respond_text(
            request,
            session_json,
            "application/json; charset=utf-8",
            head_only,
        )?,
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
