mod cli;
mod doctor;
mod project;

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
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
