use crate::error::{ArtifactError, io_at};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn publish_directory(
    destination: &Path,
    build: impl FnOnce(&Path) -> Result<(), ArtifactError>,
) -> Result<(), ArtifactError> {
    let parent = parent_or_create(destination)?;
    let stage = unique_sibling(destination, "stage");
    fs::create_dir(&stage).map_err(|error| io_at(&stage, error))?;
    if let Err(error) = build(&stage) {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    swap_directory(&stage, destination, &parent)
}

pub(crate) fn publish_file(
    destination: &Path,
    build: impl FnOnce(&Path) -> Result<(), ArtifactError>,
) -> Result<(), ArtifactError> {
    let parent = parent_or_create(destination)?;
    let stage = unique_sibling(destination, "stage");
    if let Err(error) = build(&stage) {
        let _ = fs::remove_file(&stage);
        return Err(error);
    }
    let backup = unique_sibling(destination, "backup");
    let had_destination = destination.exists();
    if had_destination {
        fs::rename(destination, &backup).map_err(|error| io_at(destination, error))?;
    }
    if let Err(error) = fs::rename(&stage, destination) {
        if had_destination {
            let _ = fs::rename(&backup, destination);
        }
        let _ = fs::remove_file(&stage);
        return Err(io_at(destination, error));
    }
    if had_destination {
        fs::remove_file(&backup).map_err(|error| io_at(&backup, error))?;
    }
    sync_directory(&parent)
}

fn swap_directory(stage: &Path, destination: &Path, parent: &Path) -> Result<(), ArtifactError> {
    let backup = unique_sibling(destination, "backup");
    let had_destination = destination.exists();
    if had_destination {
        fs::rename(destination, &backup).map_err(|error| io_at(destination, error))?;
    }
    if let Err(error) = fs::rename(stage, destination) {
        if had_destination {
            let _ = fs::rename(&backup, destination);
        }
        let _ = fs::remove_dir_all(stage);
        return Err(io_at(destination, error));
    }
    if had_destination {
        fs::remove_dir_all(&backup).map_err(|error| io_at(&backup, error))?;
    }
    sync_directory(parent)
}

fn parent_or_create(destination: &Path) -> Result<PathBuf, ArtifactError> {
    let parent = destination
        .parent()
        .ok_or_else(|| ArtifactError::Invalid("destination has no parent".to_string()))?;
    fs::create_dir_all(parent).map_err(|error| io_at(parent, error))?;
    Ok(parent.to_path_buf())
}

fn unique_sibling(destination: &Path, label: &str) -> PathBuf {
    let name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("lingo-artifact");
    loop {
        let count = STAGE_COUNTER.fetch_add(1, Ordering::Relaxed);
        let candidate =
            destination.with_file_name(format!(".{name}.{label}-{}-{count}", std::process::id()));
        if !candidate.exists() {
            return candidate;
        }
    }
}

fn sync_directory(directory: &Path) -> Result<(), ArtifactError> {
    let file = fs::File::open(directory).map_err(|error| io_at(directory, error))?;
    file.sync_all().map_err(|error| io_at(directory, error))
}
