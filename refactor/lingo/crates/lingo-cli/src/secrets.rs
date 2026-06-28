//! Local, gitignored secret storage for the viewer and CLI.
//!
//! `config.toml` only ever references audio secrets by environment variable name
//! (`api_key = "env:ELEVENLABS_API_KEY"`) — it never stores a literal key. This
//! module lets the UI persist the key itself in a sibling `.lingo.secrets.toml`
//! (gitignored, 0600). Secrets are resolved from the real environment first and
//! the file second, so an exported variable always wins and nothing mutates the
//! process environment (the crate forbids `unsafe`, so `set_var` is off-limits).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const SECRETS_FILE: &str = ".lingo.secrets.toml";

pub fn secrets_path(root: &Path) -> PathBuf {
    root.join(SECRETS_FILE)
}

fn read_table(root: &Path) -> BTreeMap<String, toml::Value> {
    std::fs::read_to_string(secrets_path(root))
        .ok()
        .and_then(|text| toml::from_str(&text).ok())
        .unwrap_or_default()
}

/// The stored secret for an environment variable name, if any.
pub fn read(root: &Path, name: &str) -> Option<String> {
    read_table(root)
        .get(name)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

/// Resolve a secret from the real environment first, then the local file.
pub fn resolve(root: &Path, name: &str) -> Option<String> {
    std::env::var(name).ok().or_else(|| read(root, name))
}

/// Whether a secret is available either in the environment or the local file.
pub fn present(root: &Path, name: &str) -> bool {
    std::env::var_os(name).is_some() || read(root, name).is_some()
}

/// Persist `value` under the environment variable `name` in the deck's
/// gitignored secrets file.
pub fn store(root: &Path, name: &str, value: &str) -> std::io::Result<()> {
    let path = secrets_path(root);
    let mut table = read_table(root);
    table.insert(name.to_string(), toml::Value::String(value.to_string()));
    let rendered = toml::to_string_pretty(&table)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    std::fs::write(&path, rendered)?;
    restrict_permissions(&path);
    ensure_gitignored(root);
    Ok(())
}

#[cfg(unix)]
fn restrict_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_permissions(_path: &Path) {}

/// Make sure the secrets file can never be committed.
fn ensure_gitignored(root: &Path) {
    let gitignore = root.join(".gitignore");
    let existing = std::fs::read_to_string(&gitignore).unwrap_or_default();
    if existing.lines().any(|line| line.trim() == SECRETS_FILE) {
        return;
    }
    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(SECRETS_FILE);
    updated.push('\n');
    let _ = std::fs::write(gitignore, updated);
}
