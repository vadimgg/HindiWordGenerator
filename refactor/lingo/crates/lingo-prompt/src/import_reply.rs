use crate::error::{PromptAdapterError, strip_one_optional_fence};
use lingo_application::ports::{ImportDraftItem, PromptStage, SourceBatchDraft};
use serde::Deserialize;

const FORMAT: &str = "lingo.import-reply/v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportReplyDto {
    format: String,
    items: Vec<ImportItemDto>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ImportItemDto {
    target: String,
    romanisation: Option<String>,
    english: String,
    #[serde(default)]
    tags: Vec<String>,
}

pub(crate) fn parse_import_reply(raw: &str) -> Result<SourceBatchDraft, PromptAdapterError> {
    let body = strip_one_optional_fence(raw, PromptStage::Import)?;
    let dto: ImportReplyDto = serde_yaml::from_str(body).map_err(|error| {
        PromptAdapterError::invalid(PromptStage::Import, yaml_path(&error), error)
    })?;
    if dto.format != FORMAT {
        return Err(PromptAdapterError::invalid(
            PromptStage::Import,
            "format",
            format!("expected {FORMAT:?}, found {:?}", dto.format),
        ));
    }
    if dto.items.is_empty() {
        return Err(PromptAdapterError::invalid(
            PromptStage::Import,
            "items",
            "at least one item is required",
        ));
    }
    let items = dto
        .items
        .into_iter()
        .enumerate()
        .map(|(index, item)| validate_item(index, item))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(SourceBatchDraft { items })
}

fn validate_item(index: usize, item: ImportItemDto) -> Result<ImportDraftItem, PromptAdapterError> {
    require_non_empty(
        PromptStage::Import,
        format!("items[{index}].target"),
        &item.target,
    )?;
    require_non_empty(
        PromptStage::Import,
        format!("items[{index}].english"),
        &item.english,
    )?;
    if item
        .romanisation
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(PromptAdapterError::invalid(
            PromptStage::Import,
            format!("items[{index}].romanisation"),
            "empty strings must be omitted, not used for absence",
        ));
    }
    Ok(ImportDraftItem {
        target: item.target,
        romanisation: item.romanisation,
        english: item.english,
        tags: item.tags,
    })
}

fn require_non_empty(
    stage: PromptStage,
    path: String,
    value: &str,
) -> Result<(), PromptAdapterError> {
    if value.trim().is_empty() {
        Err(PromptAdapterError::invalid(
            stage,
            path,
            "value cannot be empty",
        ))
    } else {
        Ok(())
    }
}

fn yaml_path(error: &serde_yaml::Error) -> String {
    error
        .location()
        .map(|location| format!("line {} column {}", location.line(), location.column()))
        .unwrap_or_else(|| "$".to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_import_reply;

    #[test]
    fn accepts_plain_and_fenced_yaml() {
        let reply = "format: lingo.import-reply/v1\nitems:\n  - target: नमस्ते\n    romanisation: namaste\n    english: Hello\n";
        assert_eq!(parse_import_reply(reply).unwrap().items.len(), 1);
        assert_eq!(
            parse_import_reply(&format!("```yaml\n{reply}```"))
                .unwrap()
                .items
                .len(),
            1
        );
    }

    #[test]
    fn rejects_surrounding_prose() {
        let reply = "Here you go:\n```yaml\nformat: lingo.import-reply/v1\nitems: []\n```";
        assert!(parse_import_reply(reply).is_err());
    }
}
