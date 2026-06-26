use lingo_application::ports::{
    BuildPromptRequest, ImportPromptRequest, PromptPacket, PromptStage,
};

pub(crate) fn build_import_packet(
    request: &ImportPromptRequest,
    language_guidance: &str,
) -> PromptPacket {
    let mut packet = String::new();
    section(&mut packet, "LINGO IMPORT PACKET");
    section_with_body(&mut packet, "LANGUAGE GUIDANCE", language_guidance);
    section_with_body(
        &mut packet,
        "LEARNER CONTEXT",
        &learner_context(&request.context),
    );
    section_with_body(
        &mut packet,
        "SOURCE IDENTITY",
        &format!(
            "batch: {}\ntitle: {}\nsubtitle: {}\nraw_file: {}",
            request.batch,
            request.title,
            request
                .subtitle
                .as_ref()
                .map_or("(none)", |value| value.as_str()),
            request.raw.relative_path().display()
        ),
    );
    section_with_body(&mut packet, "CRITICAL RULES", IMPORT_RULES);
    section_with_body(&mut packet, "OUTPUT CONTRACT", IMPORT_CONTRACT);
    section_with_body(&mut packet, "WORKED EXAMPLE", IMPORT_EXAMPLE);
    section_with_body(
        &mut packet,
        "RAW MATERIAL",
        &fenced("text", request.raw.text()),
    );
    PromptPacket {
        stage: PromptStage::Import,
        content: packet,
    }
}

pub(crate) fn build_build_packet(
    request: &BuildPromptRequest,
    language_guidance: &str,
) -> PromptPacket {
    let source_json = serde_json::to_string_pretty(&request.source)
        .unwrap_or_else(|_| "<source serialization failed>".to_string());
    let mut packet = String::new();
    section(&mut packet, "LINGO BUILD PACKET");
    section_with_body(&mut packet, "LANGUAGE GUIDANCE", language_guidance);
    section_with_body(
        &mut packet,
        "LEARNER CONTEXT",
        &learner_context(&request.context),
    );
    section_with_body(
        &mut packet,
        "SOURCE IDENTITY",
        &format!(
            "batch: {}\ntitle: {}\nsubtitle: {}",
            request.source.batch_id(),
            request.source.title(),
            request
                .source
                .subtitle()
                .map_or("(none)", |value| value.as_str())
        ),
    );
    section_with_body(&mut packet, "CRITICAL RULES", BUILD_RULES);
    section_with_body(&mut packet, "OUTPUT CONTRACT", BUILD_CONTRACT);
    section_with_body(&mut packet, "WORKED EXAMPLE", BUILD_EXAMPLE);
    section_with_body(
        &mut packet,
        "CANONICAL SOURCE",
        &fenced("json", &source_json),
    );
    PromptPacket {
        stage: PromptStage::Build,
        content: packet,
    }
}

fn learner_context(context: &lingo_application::ports::DeckContext) -> String {
    format!(
        "target_language: {}\ntarget_script: {}\nromanisation: {}\nnative_languages: {}\nlocation: {}\ngoal: {}\nnotes: {}",
        context.profile.language(),
        context.profile.script(),
        context.profile.romanisation().wire_name(),
        if context.learner.native_languages.is_empty() {
            "(not configured)".to_string()
        } else {
            context.learner.native_languages.join(", ")
        },
        context
            .learner
            .location
            .as_deref()
            .unwrap_or("(not configured)"),
        context
            .learner
            .goal
            .as_deref()
            .unwrap_or("(not configured)"),
        context
            .learner
            .notes
            .as_deref()
            .unwrap_or("(not configured)")
    )
}

fn section(output: &mut String, title: &str) {
    output.push_str("# ");
    output.push_str(title);
    output.push_str("\n\n");
}

fn section_with_body(output: &mut String, title: &str, body: &str) {
    output.push_str("## ");
    output.push_str(title);
    output.push_str("\n\n");
    output.push_str(body.trim());
    output.push_str("\n\n");
}

fn fenced(language: &str, body: &str) -> String {
    let longest = longest_backtick_run(body).max(2) + 1;
    let fence = "`".repeat(longest);
    format!("{fence}{language}\n{body}\n{fence}")
}

fn longest_backtick_run(body: &str) -> usize {
    let mut current = 0;
    let mut longest = 0;
    for character in body.chars() {
        if character == '`' {
            current += 1;
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

const IMPORT_RULES: &str = r#"- Return exactly one YAML document and no prose.
- Use the exact format tag `lingo.import-reply/v1` once.
- Segment complete, useful learner sentences; omit page chrome and fragments.
- Never invent facts.
- For each item return target, English, optional romanisation, and tags only.
- Romanise every item when the selected profile requires romanisation."#;

const IMPORT_CONTRACT: &str = r#"format: lingo.import-reply/v1
items:
  - target: "<target-language sentence>"
    romanisation: "<profile-conformant romanisation>"
    english: "<natural English gloss>"
    tags: ["<lowercase-tag>"]"#;

const IMPORT_EXAMPLE: &str = r#"format: lingo.import-reply/v1
items:
  - target: "यह किताब कैसी है?"
    romanisation: "yah kitāb kaisī hai?"
    english: "How is this book?"
    tags: ["example"]"#;

const BUILD_RULES: &str = r#"- Return raw JSON only and no surrounding prose.
- Use the exact format tag `lingo.build-reply/v1` once.
- Return exactly one card for every source item, using its exact `source_item` id.
- Never change source target, romanisation, or English; Lingo carries those forward.
- Tokens are ordered visible words only; never emit punctuation-only tokens.
- Every token word_id must name exactly one word entry.
- Every word entry must be referenced by at least one token.
- Token romanisation should reconstruct the source romanisation when combined with source spacing and punctuation.
- Tags must be non-empty and should include topic, register, and source/chapter tags when inferable.
- Use only documented register, kind, and grammar values."#;

const BUILD_CONTRACT: &str = r#"{
  "format": "lingo.build-reply/v1",
  "cards": [{
    "source_item": "<source item id>",
    "literal": "<close literal gloss>",
    "register": "informal|standard|formal",
    "tokens": [{"target":"...","romanisation":"...","word_id":"w1"}],
    "words": [{
      "id":"w1", "target":"...", "romanisation":"...",
      "meaning":"...", "kind":"noun|verb|adjective|adverb|pronoun|postposition|particle|conjunction|interjection|numeral|proper_noun|other",
      "grammar":["masculine|feminine|singular|plural|formal|informal|intimate|present|past|future|habitual|perfective|imperfective|subjunctive"]
    }],
    "tags": ["lowercase-tag"]
  }]
}"#;

const BUILD_EXAMPLE: &str = r#"{
  "format": "lingo.build-reply/v1",
  "cards": [{
    "source_item": "s-0123456789abcdef-01",
    "literal": "this book how is",
    "register": "standard",
    "tokens": [
      {"target":"यह","romanisation":"yah","word_id":"w1"},
      {"target":"किताब","romanisation":"kitāb","word_id":"w2"}
    ],
    "words": [
      {"id":"w1","target":"यह","romanisation":"yah","meaning":"this","kind":"pronoun","grammar":[]},
      {"id":"w2","target":"किताब","romanisation":"kitāb","meaning":"book","kind":"noun","grammar":["feminine","singular"]}
    ],
    "tags": ["example"]
  }]
}"#;

#[cfg(test)]
mod tests {
    use super::fenced;

    #[test]
    fn chooses_a_safe_fence() {
        assert!(fenced("text", "value ``` inside").starts_with("````text"));
    }
}
