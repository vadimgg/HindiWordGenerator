#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Prompt rendering and strict reply parsing for extract/enrich.
//!
//! Prompts are per-language Handlebars templates. Defaults are embedded; a deck
//! can override a stage by dropping `prompts/<stage>.md.hbs` in its workspace.

use handlebars::Handlebars;
use lingo_application::ports::{
    DeckContext, EnrichDraft, EnrichPromptRequest, EnrichSentenceDraft, ExtractDraft,
    ExtractPromptRequest, ExtractSentenceDraft, PromptEngine, PromptFailure, PromptPacket,
};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;

const EXTRACT_DEFAULT: &str = include_str!("templates/extract.md.hbs");
const ENRICH_DEFAULT: &str = include_str!("templates/enrich.md.hbs");

/// Embedded per-language style fragments, keyed by language code. A deck can
/// override any of these by dropping `prompts/style.<code>.md` in its workspace.
const STYLE_FRAGMENTS: &[(&str, &str)] = &[("hi", include_str!("templates/style.hi.md"))];

#[derive(Clone, Debug, Default)]
pub struct HandlebarsPromptEngine {
    /// Optional deck-local override directory (`<deck>/prompts`).
    overrides: Option<PathBuf>,
}

impl HandlebarsPromptEngine {
    pub fn strict() -> Self { Self { overrides: None } }

    /// Use deck-local `prompts/<stage>.md.hbs` overrides when present.
    pub fn with_overrides(dir: impl Into<PathBuf>) -> Self {
        Self { overrides: Some(dir.into()) }
    }

    /// Resolve a stage template: deck override if present, else the embedded default.
    fn template(&self, stage: &str, default: &str) -> Result<String, PromptFailure> {
        if let Some(dir) = &self.overrides {
            let path = dir.join(format!("{stage}.md.hbs"));
            if path.is_file() {
                return std::fs::read_to_string(&path)
                    .map_err(|error| PromptFailure::Render(format!("{}: {error}", path.display())));
            }
        }
        Ok(default.to_string())
    }

    fn render(&self, stage: &str, default: &str, data: &serde_json::Value) -> Result<String, PromptFailure> {
        let template = self.template(stage, default)?;
        let mut engine = Handlebars::new();
        engine.set_strict_mode(false);
        engine.render_template(&template, data).map_err(|error| PromptFailure::Render(error.to_string()))
    }

    /// Language-specific romanisation/style rules injected into both stages.
    /// Deck override (`prompts/style.<code>.md`) wins over the embedded default;
    /// an unknown language yields an empty fragment (a generic, language-neutral
    /// prompt) rather than an error.
    fn style_rules(&self, code: &str) -> String {
        if let Some(dir) = &self.overrides {
            let path = dir.join(format!("style.{code}.md"));
            if path.is_file() {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    return text;
                }
            }
        }
        STYLE_FRAGMENTS
            .iter()
            .find(|(key, _)| *key == code)
            .map(|(_, text)| (*text).to_string())
            .unwrap_or_default()
    }
}

/// Language facts every template can interpolate.
fn language_context(context: &DeckContext) -> serde_json::Value {
    json!({
        "language": context.profile.language().as_str(),
        "code": context.profile.code().as_str(),
        "script": context.profile.script().as_str(),
        "romanisation": context.profile.romanisation().wire_name(),
    })
}

impl PromptEngine for HandlebarsPromptEngine {
    fn render_extract(&self, request: &ExtractPromptRequest) -> Result<PromptPacket, PromptFailure> {
        let mut data = language_context(&request.context);
        data["collection"] = json!(request.collection_title.as_str());
        data["section"] = json!(request.section.as_ref().map(|s| s.as_str()));
        data["style_rules"] = json!(self.style_rules(request.context.profile.code().as_str()));
        let mut packet = self.render("extract", EXTRACT_DEFAULT, &data)?;
        packet.push_str("\n```text\n");
        packet.push_str(&request.raw);
        packet.push_str("\n```\n");
        Ok(PromptPacket { run_id: request.run_id.clone(), content: packet })
    }

    fn parse_extract_reply(&self, reply: &str) -> Result<ExtractDraft, PromptFailure> {
        let body = strip_one_optional_fence(reply).map_err(PromptFailure::InvalidReply)?;
        let dto: ExtractReplyDto = serde_yaml::from_str(body).map_err(|error| PromptFailure::InvalidReply(error.to_string()))?;
        if dto.format != "lingo.extract/v1" { return Err(PromptFailure::InvalidReply(format!("expected lingo.extract/v1, found {}", dto.format))); }
        if dto.sentences.is_empty() { return Err(PromptFailure::InvalidReply("sentences must not be empty".to_string())); }
        Ok(ExtractDraft { sentences: dto.sentences })
    }

    fn render_enrich(&self, request: &EnrichPromptRequest) -> Result<PromptPacket, PromptFailure> {
        let mut data = language_context(&request.context);
        data["style_rules"] = json!(self.style_rules(request.context.profile.code().as_str()));
        let mut packet = self.render("enrich", ENRICH_DEFAULT, &data)?;
        packet.push_str("\n```json\n");
        let payload = request.run.sentences.iter().map(|sentence| {
            json!({
                "id": sentence.id().as_str(),
                "target": sentence.target().as_str(),
                "romanisation": sentence.romanisation().map(|v| v.as_str()),
                "english": sentence.english().map(|v| v.as_str()),
                "authority": sentence.authority(),
            })
        }).collect::<Vec<_>>();
        packet.push_str(&serde_json::to_string_pretty(&payload).map_err(|error| PromptFailure::Render(error.to_string()))?);
        packet.push_str("\n```\n");
        Ok(PromptPacket { run_id: request.run.run_id.clone(), content: packet })
    }

    fn parse_enrich_reply(&self, reply: &str) -> Result<EnrichDraft, PromptFailure> {
        let body = strip_one_optional_fence(reply).map_err(PromptFailure::InvalidReply)?;
        let dto: EnrichReplyDto = serde_json::from_str(body).map_err(|error| PromptFailure::InvalidReply(error.to_string()))?;
        if dto.format != "lingo.enrich/v1" { return Err(PromptFailure::InvalidReply(format!("expected lingo.enrich/v1, found {}", dto.format))); }
        if dto.sentences.is_empty() { return Err(PromptFailure::InvalidReply("sentences must not be empty".to_string())); }
        Ok(EnrichDraft { sentences: dto.sentences })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExtractReplyDto {
    format: String,
    sentences: Vec<ExtractSentenceDraft>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EnrichReplyDto {
    format: String,
    sentences: Vec<EnrichSentenceDraft>,
}

fn strip_one_optional_fence(raw: &str) -> Result<&str, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return Err("reply is empty".to_string()); }
    if !trimmed.starts_with("```") {
        if trimmed.contains("```") { return Err("code fences may only wrap the complete reply".to_string()); }
        return Ok(trimmed);
    }
    let opening_end = trimmed.find('\n').ok_or_else(|| "opening code fence has no body".to_string())?;
    let opening = &trimmed[..opening_end];
    if !opening.starts_with("```") { return Err("invalid opening fence".to_string()); }
    let closing_start = trimmed.rfind("```").ok_or_else(|| "missing closing fence".to_string())?;
    if closing_start <= opening_end { return Err("missing closing fence".to_string()); }
    if !trimmed[closing_start + 3..].trim().is_empty() { return Err("surrounding prose after fence is not allowed".to_string()); }
    let body = trimmed[opening_end + 1..closing_start].trim();
    if body.is_empty() { return Err("reply is empty".to_string()); }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lingo_application::ports::PromptEngine;

    #[test]
    fn parse_extract_rejects_unknown_fields() {
        let reply = "format: lingo.extract/v1\nsentences: []\nextra: true\n";
        assert!(HandlebarsPromptEngine::strict().parse_extract_reply(reply).is_err());
    }

    #[test]
    fn hindi_has_embedded_style_rules() {
        let rules = HandlebarsPromptEngine::strict().style_rules("hi");
        assert!(rules.contains("maĩ"), "hindi fragment should carry the maĩ rule");
    }

    #[test]
    fn unknown_language_yields_no_style_rules() {
        // A language with no fragment renders a generic, language-neutral prompt.
        assert!(HandlebarsPromptEngine::strict().style_rules("ja").is_empty());
    }

    #[test]
    fn parse_extract_accepts_documented_example() {
        // Mirrors the ```yaml example in templates/extract.md.hbs.
        let reply = "```yaml\nformat: lingo.extract/v1\nsentences:\n  - target: \"अध्यापक जी, यहाँ कितने विद्यार्थी हैं?\"\n    romanisation: \"adhyāpak jī, yahā̃ kitne vidyārthī haĩ?\"\n    english: \"Teacher ji, how many students are here?\"\n    authority:\n      english: human\n    tags:\n      - classroom\n      - question\n  - target: \"मैं ठीक हूँ।\"\n    romanisation: \"maĩ ṭhīk hū̃.\"\n    english: \"I am fine.\"\n```";
        let draft = HandlebarsPromptEngine::strict().parse_extract_reply(reply).unwrap();
        assert_eq!(draft.sentences.len(), 2);
        assert_eq!(draft.sentences[0].authority.get("english").map(String::as_str), Some("human"));
        assert_eq!(draft.sentences[0].tags, vec!["classroom", "question"]);
    }

    #[test]
    fn parse_enrich_accepts_documented_example() {
        // Mirrors the ```json example in templates/enrich.md.hbs.
        let reply = r#"```json
{
  "format": "lingo.enrich/v1",
  "sentences": [
    {
      "id": "sen-0001",
      "romanisation": "kaise ho Pratāp?",
      "english": "How are you, Pratap?",
      "literal": "how are Pratap",
      "register": "informal",
      "breakdown": [
        { "surface": "कैसे", "roman": "kaise", "gloss": "how", "kind": "adverb" },
        { "surface": "हो", "roman": "ho", "gloss": "are", "kind": "verb" },
        { "surface": "प्रताप", "roman": "Pratāp", "gloss": "Pratap", "kind": "noun" }
      ]
    }
  ]
}
```"#;
        let draft = HandlebarsPromptEngine::strict().parse_enrich_reply(reply).unwrap();
        assert_eq!(draft.sentences.len(), 1);
        assert_eq!(draft.sentences[0].register.as_deref(), Some("informal"));
        assert_eq!(draft.sentences[0].breakdown.len(), 3);
    }

    #[test]
    fn parse_enrich_accepts_fenced_json() {
        let reply = r#"```json
{"format":"lingo.enrich/v1","sentences":[{"id":"sen-1","romanisation":"namaste","english":"Hello","literal":"hello","register":"standard","breakdown":[{"surface":"नमस्ते","roman":"namaste","gloss":"hello"}]}]}
```"#;
        assert_eq!(HandlebarsPromptEngine::strict().parse_enrich_reply(reply).unwrap().sentences.len(), 1);
    }
}
