use anyhow::{Context, Result};
use std::path::Path;

/// @intent Read a project file while preserving path context for CLI errors.
pub fn read_project_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))
}
