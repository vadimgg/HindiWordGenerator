#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    pub const fn wire_name(self) -> &'static str {
        match self { Self::Warning => "warning", Self::Error => "error" }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticCode {
    MissingRomanisation,
    MissingEnglish,
    MissingLiteral,
    MissingBreakdown,
    MissingAudio,
    AuthorityViolation,
}

impl DiagnosticCode {
    pub const fn wire_name(self) -> &'static str {
        match self {
            Self::MissingRomanisation => "missing_romanisation",
            Self::MissingEnglish => "missing_english",
            Self::MissingLiteral => "missing_literal",
            Self::MissingBreakdown => "missing_breakdown",
            Self::MissingAudio => "missing_audio",
            Self::AuthorityViolation => "authority_violation",
        }
    }

    pub const fn default_severity(self) -> Severity {
        match self { Self::MissingAudio => Severity::Warning, _ => Severity::Error }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    sentence: Option<crate::SentenceId>,
    message: String,
}

impl Diagnostic {
    pub fn new(code: DiagnosticCode, sentence: Option<crate::SentenceId>, message: impl Into<String>) -> Self {
        Self { code, severity: code.default_severity(), sentence, message: message.into() }
    }
    pub const fn code(&self) -> DiagnosticCode { self.code }
    pub const fn severity(&self) -> Severity { self.severity }
    pub fn sentence(&self) -> Option<&crate::SentenceId> { self.sentence.as_ref() }
    pub fn message(&self) -> &str { &self.message }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn new() -> Self { Self::default() }
    pub fn push(&mut self, diagnostic: Diagnostic) { self.diagnostics.push(diagnostic); }
    pub fn diagnostics(&self) -> &[Diagnostic] { &self.diagnostics }
    pub fn is_clean(&self) -> bool { self.error_count() == 0 }
    pub fn error_count(&self) -> usize { self.diagnostics.iter().filter(|d| d.severity == Severity::Error).count() }
    pub fn warning_count(&self) -> usize { self.diagnostics.len().saturating_sub(self.error_count()) }
}
