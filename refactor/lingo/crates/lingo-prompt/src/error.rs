use lingo_application::ports::{PromptFailure, PromptStage};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PromptAdapterError {
    #[error("could not render {stage} prompt: {message}")]
    Render {
        stage: &'static str,
        message: String,
    },
    #[error("invalid {stage:?} reply at {path}: {message}")]
    InvalidReply {
        stage: PromptStage,
        path: String,
        message: String,
    },
}

impl PromptAdapterError {
    pub(crate) fn render(stage: &'static str, error: impl std::fmt::Display) -> Self {
        Self::Render {
            stage,
            message: error.to_string(),
        }
    }

    pub(crate) fn invalid(
        stage: PromptStage,
        path: impl Into<String>,
        error: impl std::fmt::Display,
    ) -> Self {
        Self::InvalidReply {
            stage,
            path: path.into(),
            message: error.to_string(),
        }
    }
}

impl From<PromptAdapterError> for PromptFailure {
    fn from(error: PromptAdapterError) -> Self {
        match error {
            PromptAdapterError::Render { .. } => PromptFailure::Render(error.to_string()),
            PromptAdapterError::InvalidReply { .. } => {
                PromptFailure::InvalidReply(error.to_string())
            }
        }
    }
}

pub(crate) fn strip_one_optional_fence(
    raw: &str,
    stage: PromptStage,
) -> Result<&str, PromptAdapterError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(PromptAdapterError::invalid(stage, "$", "reply is empty"));
    }
    if !trimmed.starts_with("```") {
        if trimmed.contains("```") {
            return Err(PromptAdapterError::invalid(
                stage,
                "$",
                "code fences may only wrap the complete reply",
            ));
        }
        return Ok(trimmed);
    }

    let opening_end = trimmed
        .find('\n')
        .ok_or_else(|| PromptAdapterError::invalid(stage, "$", "opening code fence has no body"))?;
    let opening = &trimmed[..opening_end];
    let fence_len = opening.chars().take_while(|value| *value == '`').count();
    if fence_len < 3
        || !opening[fence_len..]
            .chars()
            .all(|value| value.is_ascii_alphabetic())
    {
        return Err(PromptAdapterError::invalid(
            stage,
            "$",
            "invalid opening code fence",
        ));
    }
    let fence = "`".repeat(fence_len);
    let closing_start = trimmed
        .rfind(&fence)
        .filter(|index| *index > opening_end)
        .ok_or_else(|| PromptAdapterError::invalid(stage, "$", "missing closing code fence"))?;
    if !trimmed[closing_start + fence_len..].trim().is_empty() {
        return Err(PromptAdapterError::invalid(
            stage,
            "$",
            "surrounding prose after the code fence is not allowed",
        ));
    }
    let body = trimmed[opening_end + 1..closing_start].trim();
    if body.contains(&fence) {
        return Err(PromptAdapterError::invalid(
            stage,
            "$",
            "multiple fenced documents are not allowed",
        ));
    }
    if body.is_empty() {
        return Err(PromptAdapterError::invalid(stage, "$", "reply is empty"));
    }
    Ok(body)
}
