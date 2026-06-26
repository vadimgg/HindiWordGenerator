pub mod audio;
pub mod build;
pub mod check;
pub mod doctor;
pub mod export;
pub mod import;
pub mod init;
pub mod lang;
pub mod package;
pub mod status;
pub mod viewer;

use crate::exit::ExitStatus;
use lingo_domain::{BatchId, RunId};
use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub type CommandResult = Result<ExitStatus, Box<dyn Error>>;

pub(crate) fn parse_batch(raw: Option<&str>) -> Result<Option<BatchId>, Box<dyn Error>> {
    raw.map(BatchId::parse)
        .transpose()
        .map_err(|error| Box::new(error) as Box<dyn Error>)
}

pub(crate) fn run_id(stage: &str, batch: &BatchId) -> Result<RunId, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    RunId::parse(format!(
        "{stage}-{}-{millis}-{}",
        batch.as_str(),
        std::process::id()
    ))
    .map_err(|error| Box::new(error) as Box<dyn Error>)
}

pub(crate) fn current_dir() -> Result<PathBuf, Box<dyn Error>> {
    std::env::current_dir().map_err(|error| Box::new(error) as Box<dyn Error>)
}

pub(crate) fn command_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(std::io::Error::other(message.into()))
}
