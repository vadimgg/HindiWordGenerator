//! `lingo import-package --from <DIR>`
//!
//! Imports the sentences of a portable package (as produced by `lingo package`)
//! into the workspace's per-sentence layer (`sentences/<batch>__<item>.json`).
//! It is the inverse of the one-file-per-sentence export: each package card file
//! becomes one canonical sentence file carrying an `order` integer plus the
//! batch title/subtitle, and its audio is copied into the workspace.
//!
//! Old multi-card packages (`cards/<batch>.json`) are intentionally not handled
//! here — convert them to the one-file-per-sentence layout first.

use crate::cli::ImportPackageArgs;
use crate::commands::{CommandResult, command_error, current_dir};
use crate::composition::Composition;
use crate::exit::ExitStatus;
use crate::output::Output;
use serde_json::{Map, Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// One sentence located inside the source package, ready to import.
struct PackageSentence {
    batch: String,
    item: String,
    path: PathBuf,
}

pub fn run(args: ImportPackageArgs, output: &Output) -> CommandResult {
    let composition = Composition::discover(&current_dir()?)?;
    let layout = composition.workspace.layout();
    let sentences_dir = layout.sentences_dir();
    let audio_root = layout.audio_dir();

    let from = &args.from;
    let cards_dir = from.join("cards");
    if !cards_dir.is_dir() {
        return Err(command_error(format!(
            "{} does not look like a package: no cards/ directory",
            from.display()
        )));
    }

    let mut sentences = collect_sentences(&cards_dir)?;
    if sentences.is_empty() {
        return Err(command_error(format!(
            "no per-sentence card files found under {}. Old multi-card packages \
             (cards/<batch>.json) must be converted to one file per sentence first.",
            cards_dir.display()
        )));
    }
    // Stable, reproducible order: by batch, then by item id.
    sentences.sort_by(|a, b| (a.batch.as_str(), a.item.as_str()).cmp(&(b.batch.as_str(), b.item.as_str())));

    fs::create_dir_all(&sentences_dir)
        .map_err(|error| command_error(format!("could not create {}: {error}", sentences_dir.display())))?;

    let mut imported = 0usize;
    let mut audio_copied = 0usize;
    let mut audio_missing = 0usize;

    for (index, sentence) in sentences.iter().enumerate() {
        let order = index + 1;
        let raw = fs::read_to_string(&sentence.path)
            .map_err(|error| command_error(format!("could not read {}: {error}", sentence.path.display())))?;
        let package_file: Value = serde_json::from_str(&raw)
            .map_err(|error| command_error(format!("{} is not valid JSON: {error}", sentence.path.display())))?;

        let title = package_file.get("title").and_then(Value::as_str).unwrap_or("").to_string();
        let subtitle = package_file
            .get("subtitle")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_string);
        let mut card = package_file
            .get("card")
            .cloned()
            .ok_or_else(|| command_error(format!("{} has no \"card\" object", sentence.path.display())))?;

        // Relocate audio into the workspace and rewrite the card's path to match.
        match relocate_audio(&card, from, &sentence.batch, &sentence.item, &audio_root)? {
            AudioOutcome::Copied(workspace_path) => {
                set_audio(&mut card, Some(workspace_path));
                audio_copied += 1;
            }
            AudioOutcome::Missing => {
                set_audio(&mut card, None);
                audio_missing += 1;
            }
            AudioOutcome::None => {}
        }

        let mut sentence_file = Map::new();
        sentence_file.insert("format".to_string(), json!("lingo.sentence/v1"));
        sentence_file.insert("order".to_string(), json!(order));
        sentence_file.insert("batch".to_string(), json!(sentence.batch));
        sentence_file.insert("title".to_string(), json!(title));
        if let Some(subtitle) = subtitle {
            sentence_file.insert("subtitle".to_string(), json!(subtitle));
        }
        sentence_file.insert("card".to_string(), card);

        let dest = sentences_dir.join(format!("{}__{}.json", sentence.batch, sentence.item));
        let mut bytes = serde_json::to_vec_pretty(&Value::Object(sentence_file))
            .map_err(|error| command_error(format!("could not encode {}: {error}", dest.display())))?;
        bytes.push(b'\n');
        fs::write(&dest, &bytes)
            .map_err(|error| command_error(format!("could not write {}: {error}", dest.display())))?;
        imported += 1;
    }

    output.note(&format!(
        "Imported {imported} sentence(s) into {} ({audio_copied} audio copied{}).",
        sentences_dir.display(),
        if audio_missing > 0 { format!(", {audio_missing} missing") } else { String::new() }
    ));
    Ok(ExitStatus::Success)
}

/// Walk `cards/<batch>/<item>.json` (the one-file-per-sentence layout).
fn collect_sentences(cards_dir: &Path) -> Result<Vec<PackageSentence>, Box<dyn std::error::Error>> {
    let mut sentences = Vec::new();
    let batch_entries = fs::read_dir(cards_dir)
        .map_err(|error| command_error(format!("could not read {}: {error}", cards_dir.display())))?;
    for batch_entry in batch_entries.flatten() {
        let batch_path = batch_entry.path();
        if !batch_path.is_dir() {
            continue; // old-format cards/<batch>.json files are skipped
        }
        let Some(batch) = batch_path.file_name().and_then(|name| name.to_str()).map(str::to_string) else {
            continue;
        };
        let card_entries = fs::read_dir(&batch_path)
            .map_err(|error| command_error(format!("could not read {}: {error}", batch_path.display())))?;
        for card_entry in card_entries.flatten() {
            let path = card_entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(item) = path.file_stem().and_then(|name| name.to_str()).map(str::to_string) else {
                continue;
            };
            sentences.push(PackageSentence { batch: batch.clone(), item, path });
        }
    }
    Ok(sentences)
}

enum AudioOutcome {
    /// Audio was copied; carries the new workspace-relative path.
    Copied(String),
    /// The card referenced audio, but the file was not in the package.
    Missing,
    /// The card had no audio reference.
    None,
}

/// Copy the card's audio into `audio/sentences/<batch>/<item>.mp3` if present.
fn relocate_audio(
    card: &Value,
    from: &Path,
    batch: &str,
    item: &str,
    audio_root: &Path,
) -> Result<AudioOutcome, Box<dyn std::error::Error>> {
    let Some(source_rel) = card.get("audio").and_then(Value::as_str).filter(|value| !value.is_empty()) else {
        return Ok(AudioOutcome::None);
    };
    let source = from.join(source_rel);
    if !source.is_file() {
        return Ok(AudioOutcome::Missing);
    }
    let dest = audio_root.join(batch).join(format!("{item}.mp3"));
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| command_error(format!("could not create {}: {error}", parent.display())))?;
    }
    fs::copy(&source, &dest)
        .map_err(|error| command_error(format!("could not copy {} -> {}: {error}", source.display(), dest.display())))?;
    Ok(AudioOutcome::Copied(format!("audio/sentences/{batch}/{item}.mp3")))
}

/// Set (or clear) the card's `audio` field.
fn set_audio(card: &mut Value, value: Option<String>) {
    if let Value::Object(map) = card {
        match value {
            Some(path) => {
                map.insert("audio".to_string(), json!(path));
            }
            None => {
                map.insert("audio".to_string(), Value::Null);
            }
        }
    }
}
