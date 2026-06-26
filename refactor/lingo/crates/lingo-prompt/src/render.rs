use crate::build_reply::parse_build_reply;
use crate::error::PromptAdapterError;
use crate::import_reply::parse_import_reply;
use crate::packet::{build_build_packet, build_import_packet};
use handlebars::Handlebars;
use lingo_application::ports::{
    BuildPromptRequest, CardBatchDraft, ImportPromptRequest, PromptEngine, PromptFailure,
    PromptPacket, SourceBatchDraft,
};
use serde::Serialize;

#[derive(Debug)]
pub struct HandlebarsPromptEngine {
    registry: Handlebars<'static>,
}

impl HandlebarsPromptEngine {
    pub fn strict() -> Self {
        let mut registry = Handlebars::new();
        registry.set_strict_mode(true);
        registry.register_escape_fn(handlebars::no_escape);
        Self { registry }
    }

    fn render_import_guidance(
        &self,
        request: &ImportPromptRequest,
    ) -> Result<String, PromptAdapterError> {
        self.registry
            .render_template(
                &request.context.import_prompt.content,
                &TemplateContext::from_import(request),
            )
            .map_err(|error| PromptAdapterError::render("import", error))
    }

    fn render_build_guidance(
        &self,
        request: &BuildPromptRequest,
    ) -> Result<String, PromptAdapterError> {
        self.registry
            .render_template(
                &request.context.build_prompt.content,
                &TemplateContext::from_build(request),
            )
            .map_err(|error| PromptAdapterError::render("build", error))
    }
}

impl Default for HandlebarsPromptEngine {
    fn default() -> Self {
        Self::strict()
    }
}

impl PromptEngine for HandlebarsPromptEngine {
    fn render_import(&self, request: &ImportPromptRequest) -> Result<PromptPacket, PromptFailure> {
        let guidance = self.render_import_guidance(request)?;
        Ok(build_import_packet(request, &guidance))
    }

    fn parse_import_reply(&self, reply: &str) -> Result<SourceBatchDraft, PromptFailure> {
        parse_import_reply(reply).map_err(Into::into)
    }

    fn render_build(&self, request: &BuildPromptRequest) -> Result<PromptPacket, PromptFailure> {
        let guidance = self.render_build_guidance(request)?;
        Ok(build_build_packet(request, &guidance))
    }

    fn parse_build_reply(&self, reply: &str) -> Result<CardBatchDraft, PromptFailure> {
        parse_build_reply(reply).map_err(Into::into)
    }
}

#[derive(Serialize)]
struct TemplateContext<'a> {
    target: TargetContext<'a>,
    romanisation: RomanisationContext<'a>,
    learner: LearnerContext<'a>,
    source: SourceContext<'a>,
}

#[derive(Serialize)]
struct TargetContext<'a> {
    language: &'a str,
    code: &'a str,
    script: &'a str,
}

#[derive(Serialize)]
struct RomanisationContext<'a> {
    convention: &'a str,
}

#[derive(Serialize)]
struct LearnerContext<'a> {
    native_languages: &'a [String],
    location: Option<&'a str>,
    goal: Option<&'a str>,
    notes: Option<&'a str>,
}

#[derive(Serialize)]
struct SourceContext<'a> {
    title: &'a str,
    subtitle: Option<&'a str>,
    batch: &'a str,
}

impl<'a> TemplateContext<'a> {
    fn from_import(request: &'a ImportPromptRequest) -> Self {
        Self::new(
            &request.context,
            request.title.as_str(),
            request.subtitle.as_ref().map(|value| value.as_str()),
            request.batch.as_str(),
        )
    }

    fn from_build(request: &'a BuildPromptRequest) -> Self {
        Self::new(
            &request.context,
            request.source.title().as_str(),
            request.source.subtitle().map(|value| value.as_str()),
            request.source.batch_id().as_str(),
        )
    }

    fn new(
        context: &'a lingo_application::ports::DeckContext,
        title: &'a str,
        subtitle: Option<&'a str>,
        batch: &'a str,
    ) -> Self {
        Self {
            target: TargetContext {
                language: context.profile.language().as_str(),
                code: context.profile.code().as_str(),
                script: context.profile.script().as_str(),
            },
            romanisation: RomanisationContext {
                convention: context.profile.romanisation().wire_name(),
            },
            learner: LearnerContext {
                native_languages: &context.learner.native_languages,
                location: context.learner.location.as_deref(),
                goal: context.learner.goal.as_deref(),
                notes: context.learner.notes.as_deref(),
            },
            source: SourceContext {
                title,
                subtitle,
                batch,
            },
        }
    }
}
