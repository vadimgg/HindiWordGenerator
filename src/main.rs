mod accepted_writer;
mod cli;
mod config;
mod doctor;
mod eval;
mod export;
mod ollama;
mod project;
mod run_report;
mod sentence_audio;
mod sentence_enrichment;
mod sentence_generate;
mod sentence_plan;
mod sentence_schema;
mod sentence_validate;
mod source_identity;
mod source_ids;
mod tts;
mod viewer;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::parse(std::env::args().skip(1)) {
        Ok(cli::Command::Help) => {
            println!("{}", cli::help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::DoctorHelp) => {
            println!("{}", cli::doctor_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::ViewerHelp) => {
            println!("{}", cli::viewer_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::ExportHelp) => {
            println!("{}", cli::export_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::Doctor) => {
            match doctor::run_from_current_dir(&doctor::HttpOllamaChecker) {
                Ok(report) => {
                    println!("{}", report.render());
                    if report.required_checks_passed() {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(1)
                }
            }
        }
        Ok(cli::Command::SourceIdsHelp) => {
            println!("{}", cli::source_ids_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::SourceIdsCheck) => match source_ids::check_from_current_dir() {
            Ok(report) => {
                println!("{}", report.render_check());
                if report.is_complete() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Ok(cli::Command::SourceIdsMigrate { dry_run }) => {
            match source_ids::migrate_from_current_dir(dry_run) {
                Ok(report) => {
                    println!("{}", report.render_migration());
                    if report.has_errors() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(1)
                }
            }
        }
        Ok(cli::Command::SentencesHelp) => {
            println!("{}", cli::sentences_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::SentencesPlan { max_batches }) => {
            match sentence_plan::plan_from_current_dir(max_batches) {
                Ok(report) => {
                    println!("{}", report.render());
                    if report.has_errors() {
                        ExitCode::from(1)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(1)
                }
            }
        }
        Ok(cli::Command::SentencesGenerate { max_batches }) => {
            match sentence_generate::generate_from_current_dir(max_batches) {
                Ok(report) => {
                    println!("{}", report.render());
                    if report.success {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(1)
                }
            }
        }
        Ok(cli::Command::SentencesAudio) => match sentence_audio::audio_from_current_dir() {
            Ok(report) => {
                println!("{}", report.render());
                if report.success {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Ok(cli::Command::EvalHelp) => {
            println!("{}", cli::eval_help_text());
            ExitCode::SUCCESS
        }
        Ok(cli::Command::EvalRun {
            input,
            prompt_id,
            fields,
            max_items,
        }) => match eval::run_from_current_dir(&input, &prompt_id, fields.as_deref(), max_items) {
            Ok(report) => {
                println!("{}", report.render());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Ok(cli::Command::EvalGrade { run }) => match eval::grade_from_current_dir(&run) {
            Ok(report) => {
                println!("{}", report.render());
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Ok(cli::Command::Viewer) => match viewer::run_from_current_dir() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(1)
            }
        },
        Ok(cli::Command::Export { source, topic }) => {
            match export::export_from_current_dir(&source, &topic) {
                Ok(report) => {
                    println!("{}", report.render());
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(1)
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
