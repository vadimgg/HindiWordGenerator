use crate::composition::clipboard_command;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InteractionError {
    #[error("no clipboard command was found")]
    ClipboardUnavailable,
    #[error("clipboard command failed: {0}")]
    Clipboard(String),
    #[error("$VISUAL or $EDITOR is not configured")]
    EditorUnavailable,
    #[error("editor failed: {0}")]
    Editor(String),
    #[error("reply file could not be read: {0}")]
    Reply(String),
}

pub fn copy_to_clipboard(text: &str) -> Result<(), InteractionError> {
    let command = clipboard_command().ok_or(InteractionError::ClipboardUnavailable)?;
    let program = command
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let mut process = Command::new(&command);
    if program == "xclip" {
        process.args(["-selection", "clipboard"]);
    }
    let mut child = process
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|error| InteractionError::Clipboard(error.to_string()))?;
    child
        .stdin
        .take()
        .ok_or_else(|| InteractionError::Clipboard("stdin was not piped".to_string()))?
        .write_all(text.as_bytes())
        .map_err(|error| InteractionError::Clipboard(error.to_string()))?;
    let status = child
        .wait()
        .map_err(|error| InteractionError::Clipboard(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(InteractionError::Clipboard(format!(
            "process exited with {status}"
        )))
    }
}

pub fn edit_reply(path: &Path) -> Result<String, InteractionError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| InteractionError::Reply(error.to_string()))?;
    }
    if !path.exists() {
        fs::write(path, "").map_err(|error| InteractionError::Reply(error.to_string()))?;
    }
    open_editor(path)?;
    fs::read_to_string(path).map_err(|error| InteractionError::Reply(error.to_string()))
}

pub fn open_editor(path: &Path) -> Result<(), InteractionError> {
    let editor = std::env::var("VISUAL")
        .ok()
        .or_else(|| std::env::var("EDITOR").ok())
        .ok_or(InteractionError::EditorUnavailable)?;
    let mut parts = editor.split_whitespace();
    let program = parts.next().ok_or(InteractionError::EditorUnavailable)?;
    let status = Command::new(program)
        .args(parts)
        .arg(path)
        .status()
        .map_err(|error| InteractionError::Editor(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(InteractionError::Editor(format!(
            "editor exited with {status}"
        )))
    }
}
