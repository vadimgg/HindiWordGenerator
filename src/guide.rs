use crate::doctor;
use crate::export;
use crate::sentence_audio;
use crate::sentence_generate;
use crate::sentence_plan;
use crate::sentence_quality;
use crate::source_ids;
use crate::viewer;
use std::fmt::Display;
use std::io::{self, IsTerminal, Write};

#[derive(Debug)]
pub enum GuideError {
    Io(io::Error),
    Step(String),
    NonInteractive,
}

impl std::fmt::Display for GuideError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuideError::Io(error) => write!(formatter, "Could not read guided-mode input.\n\n{error}"),
            GuideError::Step(error) => write!(formatter, "{error}"),
            GuideError::NonInteractive => write!(
                formatter,
                "Guided mode needs an interactive terminal.\n\nRun the workflow directly:\n  hindi doctor\n  hindi source ids check\n  hindi sentences plan --max-batches 1\n  hindi sentences generate --max-batches 1\n  hindi sentences audio\n  hindi sentences review-output\n  hindi export\n  hindi viewer"
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuideOutcome {
    pub success: bool,
}

pub fn run_from_current_dir(max_batches: usize) -> Result<GuideOutcome, GuideError> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err(GuideError::NonInteractive);
    }

    println!("Hindi Guide\n");
    println!("This walks through the normal sentence workflow one step at a time.");
    println!("Max batches: {max_batches}\n");

    section(1, "Doctor");
    let doctor_report =
        doctor::run_from_current_dir(&doctor::HttpOllamaChecker).map_err(step_error)?;
    println!("{}", doctor_report.render());
    if !doctor_report.required_checks_passed() {
        println!("\nStopped because required checks did not pass.");
        return Ok(GuideOutcome { success: false });
    }

    section(2, "Source IDs");
    let source_report = source_ids::check_from_current_dir().map_err(step_error)?;
    println!("{}", source_report.render_check());
    if !source_report.is_complete() {
        println!("\nStopped because source YAML IDs need attention.");
        println!("Run\n  hindi source ids migrate");
        return Ok(GuideOutcome { success: false });
    }

    section(3, "Plan Sentences");
    let plan = sentence_plan::plan_from_current_dir(max_batches).map_err(step_error)?;
    println!("{}", plan.render());
    if plan.has_errors() {
        println!("\nStopped because the sentence plan found problems.");
        return Ok(GuideOutcome { success: false });
    }

    if ask_yes_no("Generate the planned sentence batch now?", true)? {
        section(4, "Generate Sentences");
        let generated =
            sentence_generate::generate_from_current_dir(max_batches).map_err(step_error)?;
        println!("{}", generated.render());
        if !generated.success {
            println!("\nStopped because sentence generation did not complete.");
            return Ok(GuideOutcome { success: false });
        }
    } else {
        println!("Skipped generation. Existing accepted output can still be reviewed/exported.");
    }

    if ask_yes_no("Backfill missing sentence audio now?", true)? {
        section(5, "Sentence Audio");
        let audio = sentence_audio::audio_from_current_dir().map_err(step_error)?;
        println!("{}", audio.render());
        if !audio.success {
            println!("\nStopped because audio did not complete.");
            return Ok(GuideOutcome { success: false });
        }
    } else {
        println!("Skipped audio.");
    }

    if ask_yes_no("Review accepted output now?", true)? {
        section(6, "Review Output");
        let quality = sentence_quality::quality_from_current_dir().map_err(step_error)?;
        println!("{}", quality.render());
        if quality.has_problems()
            && !ask_yes_no("Problems were found. Continue to export anyway?", false)?
        {
            println!("Stopped before export.");
            return Ok(GuideOutcome { success: false });
        }
    } else {
        println!("Skipped output review.");
    }

    if ask_yes_no("Export Anki TSV now?", true)? {
        section(7, "Export");
        let exported = export::export_from_current_dir(None, None).map_err(step_error)?;
        println!("{}", exported.render());
    } else {
        println!("Skipped export.");
    }

    if ask_yes_no(
        "Open the viewer now? This runs until you press Ctrl-C.",
        false,
    )? {
        section(8, "Viewer");
        viewer::run_from_current_dir().map_err(step_error)?;
    } else {
        println!("Skipped viewer.");
        println!("Run\n  hindi viewer");
    }

    Ok(GuideOutcome { success: true })
}

fn section(number: usize, title: &str) {
    println!("\nStep {number}: {title}");
    println!("{}", "-".repeat(title.len() + 8));
}

fn ask_yes_no(question: &str, default_yes: bool) -> Result<bool, GuideError> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        print!("{question} {suffix} ");
        io::stdout().flush().map_err(GuideError::Io)?;
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(GuideError::Io)?;
        let answer = input.trim().to_ascii_lowercase();
        match answer.as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("Please answer y or n."),
        }
    }
}

fn step_error(error: impl Display) -> GuideError {
    GuideError::Step(error.to_string())
}
