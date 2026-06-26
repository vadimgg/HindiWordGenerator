use console::Style;
use lingo_application::ports::BootstrapChangeKind;
use lingo_application::{
    AudioReport, BuildReport, CheckReport, DoctorReport, ExportReport, ImportReport, InitReport,
    LanguageListReport, LanguageShowReport, PackageReport, PromptOriginReport, StatusReport,
};
use lingo_domain::{CheckState, Severity};

#[derive(Clone, Debug)]
pub struct Output {
    color: bool,
}

impl Output {
    pub fn new(color: bool) -> Self {
        console::set_colors_enabled(color);
        Self { color }
    }

    pub fn packet(&self, packet: &str) {
        println!("{packet}");
    }

    pub fn prepared(&self, stage: &str, prompt: &std::path::Path, reply: &std::path::Path) {
        println!("{}  prompt packet ready\n", self.heading(stage));
        println!("  prompt  {}", self.path(prompt));
        println!("  reply   {}", self.path(reply));
        println!(
            "\n  {} Paste the packet into ChatGPT or Claude, then save its reply.",
            self.warn("▸")
        );
    }

    pub fn init(&self, report: &InitReport) {
        println!("{}  {}\n", self.heading("Workspace"), report.root.display());
        for change in &report.changes {
            let label = match change.kind {
                BootstrapChangeKind::Created => self.ok("created"),
                BootstrapChangeKind::Kept => self.dim("kept"),
            };
            println!(
                "  {label:<12} {}",
                self.dim(&change.relative_path.display().to_string())
            );
        }
        self.next(&report.next);
    }

    pub fn imported(&self, report: &ImportReport) {
        println!(
            "{}  {}\n\n  items   {}\n  written {}",
            self.heading("Import"),
            report.batch,
            report.items,
            self.dim(&report.stored.relative_path().display().to_string())
        );
        self.next(&report.next);
    }

    pub fn built(&self, report: &BuildReport) {
        println!(
            "{}  {}\n\n  cards   {}\n  written {}\n  errors  {}\n  warnings {}",
            self.heading("Build"),
            report.batch,
            report.cards,
            self.dim(&report.stored.relative_path().display().to_string()),
            report.validation.error_count(),
            report.validation.warning_count()
        );
        self.next(&report.next);
    }

    pub fn checked(&self, report: &CheckReport) {
        println!(
            "{}  {} batch(es)\n",
            self.heading("Check"),
            report.batches.len()
        );
        for batch in &report.batches {
            let (marker, state) = if batch.report.is_clean() {
                (self.ok("✓"), self.ok("clean"))
            } else {
                (self.problem("✗"), self.problem("problems"))
            };
            println!(
                "  {} {}  {:<10} {} error(s), {} warning(s)",
                marker,
                batch.batch,
                state,
                batch.report.error_count(),
                batch.report.warning_count()
            );
            for diagnostic in batch.report.diagnostics() {
                let marker = match diagnostic.severity() {
                    Severity::Error => self.problem("✗"),
                    Severity::Warning => self.warn("⚠"),
                };
                println!(
                    "    {marker} {:<28} {}",
                    diagnostic.code().wire_name(),
                    diagnostic.message()
                );
            }
        }
        println!(
            "\n  {} error(s), {} warning(s)",
            report.error_count(),
            report.warning_count()
        );
    }

    pub fn audio(&self, report: &AudioReport) {
        println!(
            "{}  {} batch(es)\n",
            self.heading("Audio"),
            report.batches.len()
        );
        for batch in &report.batches {
            println!("  {}", batch.batch);
            for card in &batch.cards {
                if card.skipped {
                    println!("    {}  {}", self.dim("–"), card.card);
                } else {
                    println!(
                        "    {}  {}  {}",
                        self.ok("✓"),
                        card.card,
                        self.command(card.backend.map_or("unknown", |value| value.wire_name()))
                    );
                }
            }
        }
        println!(
            "\n  updated {}  skipped {}",
            report.counts.updated, report.counts.skipped
        );
        self.next(&report.next);
    }

    pub fn packaged(&self, report: &PackageReport) {
        println!(
            "{}\n\n  batches {}\n  cards   {}\n  files   {}\n  bytes   {}\n  path    {}",
            self.heading("Package"),
            report.batches,
            report.cards,
            report.artifact.files,
            report.artifact.bytes,
            self.dim(&report.artifact.path.display().to_string())
        );
        self.next(&report.next);
    }

    pub fn exported(&self, report: &ExportReport) {
        println!(
            "{}\n\n  deck    {}\n  batches {}\n  cards   {}\n  path    {}",
            self.heading("Export"),
            report.deck,
            report.batches,
            report.cards,
            self.dim(&report.artifact.path.display().to_string())
        );
        self.next(&report.next);
    }

    pub fn status(&self, report: &StatusReport) {
        println!(
            "{}  {} batch(es)\n",
            self.heading("Status"),
            report.total_batches
        );
        println!("  BATCH                         SOURCE  CARDS   CHECK       AUDIO");
        for row in &report.rows {
            let check = match row.check {
                CheckState::NotRun => self.dim("–"),
                CheckState::Clean => self.ok("✓"),
                CheckState::Problems { errors, warnings } => {
                    self.problem(&format!("✗ {errors}e/{warnings}w"))
                }
            };
            let audio = if row.audio.total() == 0 {
                self.dim("–")
            } else if row.audio.is_complete() {
                self.ok("✓")
            } else {
                self.warn(&format!("● {}/{}", row.audio.present(), row.audio.total()))
            };
            println!(
                "  {:<29} {:<7} {:<7} {:<11} {}",
                row.batch,
                self.stage_glyph(row.source_present),
                self.stage_glyph(row.cards_present),
                check,
                audio
            );
            if let Some(problem) = &row.problem {
                println!("    {}", self.problem(problem));
            }
        }
        if !report.pending_raw.is_empty() {
            println!("\n  Pending raw");
            for raw in &report.pending_raw {
                println!("    {} {}", self.warn("●"), raw.id);
            }
        }
        self.next(&report.next);
    }

    pub fn languages(&self, report: &LanguageListReport) {
        println!("{}\n", self.heading("Languages"));
        for profile in &report.profiles {
            println!(
                "  {:<16} {:<20} {:<8} {}",
                profile.id,
                profile.language,
                self.dim(profile.code.as_str()),
                profile.romanisation
            );
        }
    }

    pub fn language(&self, report: &LanguageShowReport) {
        println!(
            "{}  {}\n\n  language       {}\n  code           {}\n  romanisation  {}",
            self.heading("Language"),
            report.profile.id,
            report.profile.language,
            report.profile.code,
            report.profile.romanisation
        );
    }

    pub fn prompt_origins(&self, report: &PromptOriginReport) {
        println!(
            "{}  {}\n\n  import  {}\n  build   {}",
            self.heading("Prompts"),
            report.profile,
            report.import_origin,
            report.build_origin
        );
    }

    pub fn doctor(&self, report: &DoctorReport) {
        println!("{}\n", self.heading("Doctor"));
        for check in &report.checks {
            let state = if check.available {
                self.ok("✓")
            } else if check.required {
                self.problem("✗")
            } else {
                self.warn("⚠")
            };
            println!(
                "  {:<18} {:<10} {}",
                check.kind.wire_name(),
                state,
                self.dim(check.detail.as_deref().unwrap_or_default())
            );
            if !check.available {
                if let Some(recovery) = &check.recovery {
                    println!("    {}", self.dim(recovery));
                }
            }
        }
    }

    pub fn next(&self, next: &lingo_application::NextAction) {
        if let Some(command) = next.command_hint() {
            println!(
                "\n{}\n  {}",
                self.heading("Next"),
                self.command(command.as_str())
            );
        }
    }

    pub fn note(&self, text: &str) {
        println!("{}", self.dim(text));
    }

    fn heading(&self, text: &str) -> String {
        self.style(Style::new().bold().cyan(), text)
    }

    fn command(&self, text: &str) -> String {
        self.style(Style::new().cyan(), text)
    }

    fn ok(&self, text: &str) -> String {
        self.style(Style::new().green(), text)
    }

    fn warn(&self, text: &str) -> String {
        self.style(Style::new().yellow(), text)
    }

    fn problem(&self, text: &str) -> String {
        self.style(Style::new().red(), text)
    }

    fn dim(&self, text: &str) -> String {
        self.style(Style::new().dim(), text)
    }

    fn path(&self, path: &std::path::Path) -> String {
        self.dim(&path.display().to_string())
    }

    fn stage_glyph(&self, present: bool) -> String {
        if present {
            self.ok("✓")
        } else {
            self.dim("–")
        }
    }

    fn style(&self, style: Style, text: &str) -> String {
        if self.color {
            style.apply_to(text).to_string()
        } else {
            text.to_string()
        }
    }
}
