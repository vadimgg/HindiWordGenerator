#![allow(dead_code)]

use assert_cmd::Command;
use assert_cmd::cargo::cargo_bin_cmd;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestWorkspace {
    temp: TempDir,
    deck: PathBuf,
    config_home: PathBuf,
    home: PathBuf,
    bin: PathBuf,
}

impl TestWorkspace {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().expect("temporary test root should be created");
        let deck = temp.path().join("deck");
        let config_home = temp.path().join("config");
        let home = temp.path().join("home");
        let bin = temp.path().join("bin");
        for directory in [&deck, &config_home, &home, &bin] {
            fs::create_dir_all(directory).expect("test directory should be created");
        }
        Self {
            temp,
            deck,
            config_home,
            home,
            bin,
        }
    }

    pub fn root(&self) -> &Path {
        &self.deck
    }

    pub fn path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.deck.join(relative)
    }

    pub fn command(&self) -> Command {
        let mut command = cargo_bin_cmd!("lingo");
        command
            .current_dir(&self.deck)
            .env("XDG_CONFIG_HOME", &self.config_home)
            .env("HOME", &self.home)
            .env("NO_COLOR", "1")
            .env("PATH", self.test_path());
        command
    }

    pub fn write_raw(&self, name: &str, content: &str) -> PathBuf {
        let path = self.path(Path::new("raw").join(name));
        write_text(&path, content);
        path
    }

    pub fn write_valid_import_reply(&self) -> PathBuf {
        let value = json!({
            "format": "lingo.import-reply/v1",
            "items": [{
                "target": "यह एक किताब है।",
                "romanisation": "yah ek kitāb hai.",
                "english": "This is a book.",
                "tags": ["chapter-01"]
            }]
        });
        let text = serde_yaml::to_string(&value).expect("valid import fixture should serialize");
        let path = self.path("import-reply.yaml");
        write_text(&path, &text);
        path
    }

    pub fn source_item_id(&self, batch: &str) -> String {
        let path = self.path(format!("input/sentences/{batch}.yaml"));
        let bytes = fs::read(&path).expect("accepted source file should exist");
        let value: serde_yaml::Value =
            serde_yaml::from_slice(&bytes).expect("accepted source file should decode");
        value
            .get("items")
            .and_then(serde_yaml::Value::as_sequence)
            .and_then(|items| items.first())
            .and_then(|item| item.get("id"))
            .and_then(serde_yaml::Value::as_str)
            .expect("accepted source should contain an item id")
            .to_string()
    }

    pub fn write_valid_build_reply(&self, source_item: &str) -> PathBuf {
        let value = json!({
            "format": "lingo.build-reply/v1",
            "cards": [{
                "source_item": source_item,
                "literal": "this one book is",
                "register": "standard",
                "tokens": [
                    {"target": "यह", "romanisation": "yah", "word_id": "w1"},
                    {"target": "एक", "romanisation": "ek", "word_id": "w2"},
                    {"target": "किताब", "romanisation": "kitāb", "word_id": "w3"},
                    {"target": "है", "romanisation": "hai", "word_id": "w4"}
                ],
                "words": [
                    {"id": "w1", "target": "यह", "romanisation": "yah", "meaning": "this", "kind": "pronoun"},
                    {"id": "w2", "target": "एक", "romanisation": "ek", "meaning": "one", "kind": "numeral"},
                    {"id": "w3", "target": "किताब", "romanisation": "kitāb", "meaning": "book", "kind": "noun", "grammar": ["feminine", "singular"]},
                    {"id": "w4", "target": "है", "romanisation": "hai", "meaning": "is", "kind": "verb", "grammar": ["present", "singular"]}
                ],
                "tags": ["chapter-01"]
            }]
        });
        let text =
            serde_json::to_string_pretty(&value).expect("valid build fixture should serialize");
        let path = self.path("build-reply.json");
        write_text(&path, &format!("{text}\n"));
        path
    }

    pub fn install_fake_uv(&self) {
        #[cfg(unix)]
        install_fake_uv_unix(&self.bin);
        #[cfg(windows)]
        install_fake_uv_windows(&self.bin);
    }

    fn test_path(&self) -> String {
        let existing = env::var_os("PATH").unwrap_or_default();
        env::join_paths(std::iter::once(self.bin.clone()).chain(env::split_paths(&existing)))
            .expect("test PATH should be representable")
            .to_string_lossy()
            .into_owned()
    }
}

fn write_text(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("fixture parent should be created");
    }
    fs::write(path, content).expect("fixture should be written");
}

#[cfg(unix)]
fn install_fake_uv_unix(bin: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let path = bin.join("uv");
    write_text(
        &path,
        r#"#!/bin/sh
set -eu
output=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --output)
      output="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
[ -n "$output" ]
printf 'ID3lingo-test-audio' > "$output"
"#,
    );
    let mut permissions = fs::metadata(&path)
        .expect("fake uv metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("fake uv should be executable");
}

#[cfg(windows)]
fn install_fake_uv_windows(bin: &Path) {
    write_text(
        &bin.join("uv.cmd"),
        r#"@echo off
set output=
:loop
if "%~1"=="" goto done
if "%~1"=="--output" (
  set output=%~2
  shift
  shift
  goto loop
)
shift
goto loop
:done
> "%output%" echo ID3lingo-test-audio
"#,
    );
}
