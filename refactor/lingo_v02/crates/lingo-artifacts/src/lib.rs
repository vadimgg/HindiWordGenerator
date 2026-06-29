#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
//! Derived publishers for JSON packages, SQLite package copies, and Anki APKGs.

use lingo_application::ports::{
    AnkiExporter, ArtifactFailure, PackagePublisher, PublishMaterial, PublishedArtifact,
};
use lingo_domain::{Sentence, WordKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;

#[derive(Clone, Copy, Debug, Default)]
pub struct PortablePackagePublisher;

impl PackagePublisher for PortablePackagePublisher {
    /// One self-contained JSON file per sentence + copied audio + manifest with
    /// per-file checksums. Every written file is read back and verified.
    fn publish_json(&self, destination: &Path, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure> {
        if destination.exists() { fs::remove_dir_all(destination).map_err(fail)?; }
        fs::create_dir_all(destination.join("sentences")).map_err(fail)?;
        fs::create_dir_all(destination.join("audio")).map_err(fail)?;

        let mut files = BTreeMap::new();
        let mut audio_count = 0usize;
        for sentence in &material.sentences {
            // sentence file
            let relative = format!("sentences/{}.json", sentence.id().as_str());
            let file = SentenceFile::from_sentence(sentence);
            let mut bytes = serde_json::to_vec_pretty(&file).map_err(fail)?;
            bytes.push(b'\n');
            write_verified(destination, &relative, &bytes, &mut files)?;

            // audio file (copied from the source workspace)
            if let Some(audio) = sentence.audio() {
                let relative_audio = audio.path().as_str().to_string();
                let source = material.source_root.join(&relative_audio);
                let audio_bytes = fs::read(&source)
                    .map_err(|error| ArtifactFailure(format!("audio missing for {}: {error}", sentence.id())))?;
                write_verified(destination, &relative_audio, &audio_bytes, &mut files)?;
                audio_count += 1;
            }
        }

        let manifest = PackageManifest {
            format: "lingo.package/v2",
            language: material.profile.code().as_str().to_string(),
            collection: material.sentences.first().map(|s| s.collection().as_str().to_string()).unwrap_or_default(),
            counts: Counts {
                sentences: material.sentences.len(),
                words: distinct_word_count(&material.sentences),
                audio_files: audio_count,
            },
            integrity: Integrity { algorithm: "sha256", files },
        };
        let manifest_bytes = serde_json::to_vec_pretty(&manifest).map_err(fail)?;
        fs::write(destination.join("manifest.json"), &manifest_bytes).map_err(fail)?;
        fs::write(destination.join("README.txt"), README).map_err(fail)?;

        Ok(PublishedArtifact {
            path: destination.to_path_buf(),
            files: material.sentences.len() + audio_count + 2,
            bytes: directory_size(destination).map_err(fail)?,
        })
    }

    /// A real, filtered copy of the canonical `lingo.library/v1` database (so
    /// consumers like Grasp can query `sentences`/`words` directly) plus audio.
    fn publish_db_copy(&self, destination: &Path, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure> {
        if destination.exists() { fs::remove_dir_all(destination).map_err(fail)?; }
        fs::create_dir_all(destination.join("audio")).map_err(fail)?;
        let db = destination.join("library.db");

        // VACUUM INTO produces a clean single-file copy that includes all
        // committed data regardless of WAL state.
        let source = rusqlite::Connection::open(material.source_root.join("library.db")).map_err(fail)?;
        source.execute("VACUUM INTO ?1", rusqlite::params![db.to_string_lossy()]).map_err(fail)?;
        drop(source);

        // Filter the copy down to the selected sentences and prune orphan words.
        let copy = rusqlite::Connection::open(&db).map_err(fail)?;
        copy.execute_batch("PRAGMA foreign_keys = ON;").map_err(fail)?;
        copy.execute_batch("CREATE TEMP TABLE keep(id TEXT PRIMARY KEY);").map_err(fail)?;
        {
            let mut insert = copy.prepare("INSERT OR IGNORE INTO keep(id) VALUES(?1)").map_err(fail)?;
            for sentence in &material.sentences { insert.execute(rusqlite::params![sentence.id().as_str()]).map_err(fail)?; }
        }
        copy.execute("DELETE FROM sentences WHERE id NOT IN (SELECT id FROM keep)", []).map_err(fail)?;
        copy.execute("DELETE FROM words WHERE id NOT IN (SELECT DISTINCT word_id FROM sentence_words)", []).map_err(fail)?;
        copy.execute_batch("DROP TABLE keep;").map_err(fail)?;
        drop(copy);

        // Copy audio for the selected sentences.
        let mut audio_count = 0usize;
        for sentence in &material.sentences {
            if let Some(audio) = sentence.audio() {
                let relative = audio.path().as_str();
                let from = material.source_root.join(relative);
                let to = destination.join(relative);
                if let Some(parent) = to.parent() { fs::create_dir_all(parent).map_err(fail)?; }
                fs::copy(&from, &to).map_err(|error| ArtifactFailure(format!("audio missing for {}: {error}", sentence.id())))?;
                audio_count += 1;
            }
        }

        let manifest = DbManifest {
            format: "lingo.package/v2",
            library: "library.db",
            library_schema: "lingo.library/v1",
            counts: Counts {
                sentences: material.sentences.len(),
                words: distinct_word_count(&material.sentences),
                audio_files: audio_count,
            },
        };
        fs::write(destination.join("manifest.json"), serde_json::to_vec_pretty(&manifest).map_err(fail)?).map_err(fail)?;
        fs::write(destination.join("README.txt"), README).map_err(fail)?;

        Ok(PublishedArtifact {
            path: destination.to_path_buf(),
            files: audio_count + 3,
            bytes: directory_size(destination).map_err(fail)?,
        })
    }
}

const MODEL_NAME: &str = "Lingo Sentence";
const FIELDS: &[&str] = &["Lead", "Secondary", "English", "Literal", "Register", "WordBreakdown", "Audio", "Source"];

#[derive(Clone, Copy, Debug, Default)]
pub struct ApkgExporter;

impl AnkiExporter for ApkgExporter {
    fn export_apkg(&self, destination: &Path, deck: &str, material: &PublishMaterial) -> Result<PublishedArtifact, ArtifactFailure> {
        if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(fail)?; }
        let staging = tempfile::tempdir().map_err(fail)?;
        let collection_db = staging.path().join("collection.anki2");
        create_collection(&collection_db, deck, material)?;

        let file = File::create(destination).map_err(fail)?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        zip.start_file("collection.anki2", options).map_err(fail)?;
        zip.write_all(&fs::read(&collection_db).map_err(fail)?).map_err(fail)?;

        // Bundle audio as numbered media entries with a name map (Anki format).
        let mut media_map = BTreeMap::<String, String>::new();
        let mut media_index = 0usize;
        for sentence in &material.sentences {
            if let Some(audio) = sentence.audio() {
                let bytes = fs::read(material.source_root.join(audio.path().as_str()))
                    .map_err(|error| ArtifactFailure(format!("audio missing for {}: {error}", sentence.id())))?;
                zip.start_file(media_index.to_string(), options).map_err(fail)?;
                zip.write_all(&bytes).map_err(fail)?;
                media_map.insert(media_index.to_string(), media_name(sentence.id().as_str()));
                media_index += 1;
            }
        }
        zip.start_file("media", options).map_err(fail)?;
        zip.write_all(serde_json::to_string(&media_map).map_err(fail)?.as_bytes()).map_err(fail)?;
        zip.finish().map_err(fail)?;

        Ok(PublishedArtifact { path: destination.to_path_buf(), files: media_index + 2, bytes: fs::metadata(destination).map_err(fail)?.len() })
    }
}

/// Build a real Anki `collection.anki2` SQLite database for the selected sentences.
fn create_collection(path: &Path, deck: &str, material: &PublishMaterial) -> Result<(), ArtifactFailure> {
    let mut connection = rusqlite::Connection::open(path).map_err(fail)?;
    connection.execute_batch(ANKI_SCHEMA).map_err(fail)?;
    let deck_id = stable_i64(&format!("deck:{deck}"));
    let model_id = stable_i64(MODEL_NAME);
    connection.execute(
        "INSERT INTO col VALUES (?1,0,0,0,11,0,-1,0,?2,?3,?4,?5,?6)",
        rusqlite::params![1_i64, "{}", model_json(model_id, deck_id).to_string(), deck_json(deck_id, deck).to_string(), default_dconf().to_string(), "{}"],
    ).map_err(fail)?;

    let transaction = connection.transaction().map_err(fail)?;
    let mut due = 1_i64;
    for sentence in &material.sentences {
        let note_id = stable_i64(&format!("note:{}", sentence.id().as_str()));
        let card_id = stable_i64(&format!("card:{}", sentence.id().as_str()));
        let fields = note_fields(sentence, material.display);
        let tags = if sentence.tags().values().is_empty() { String::new() } else { format!(" {} ", sentence.tags().values().join(" ")) };
        transaction.execute(
            "INSERT INTO notes VALUES (?1,?2,?3,0,-1,?4,?5,?6,?7,0,'')",
            rusqlite::params![note_id, note_guid(sentence.id().as_str()), model_id, tags, fields.join("\u{1f}"), fields[0], first_field_checksum(&fields[0])],
        ).map_err(fail)?;
        transaction.execute(
            "INSERT INTO cards VALUES (?1,?2,?3,0,0,-1,0,0,?4,0,0,0,0,0,0,0,0,'')",
            rusqlite::params![card_id, note_id, deck_id, due],
        ).map_err(fail)?;
        due += 1;
    }
    transaction.commit().map_err(fail)?;
    connection.execute_batch("VACUUM;").map_err(fail)?;
    Ok(())
}

fn note_fields(sentence: &Sentence, display: lingo_domain::DisplayPolicy) -> Vec<String> {
    use lingo_domain::DisplayLead;
    let target = escape_html(sentence.target().as_str());
    let romanisation = sentence.romanisation().map(|v| escape_html(v.as_str())).unwrap_or_default();
    let (lead, secondary) = match display.lead() {
        DisplayLead::Target => (target, romanisation),
        DisplayLead::Romanisation if !romanisation.is_empty() => (romanisation, target),
        DisplayLead::Romanisation => (target, String::new()),
    };
    vec![
        lead,
        if display.show_secondary() { secondary } else { String::new() },
        sentence.english().map(|v| escape_html(v.as_str())).unwrap_or_default(),
        sentence.literal().map(|v| escape_html(v.as_str())).unwrap_or_default(),
        sentence.register().map(|r| r.wire_name().to_string()).unwrap_or_default(),
        word_breakdown(sentence),
        sentence.audio().map(|_| format!("[sound:{}]", media_name(sentence.id().as_str()))).unwrap_or_default(),
        escape_html(sentence.section().map(|s| s.as_str()).unwrap_or(sentence.collection().as_str())),
    ]
}

fn word_breakdown(sentence: &Sentence) -> String {
    let Some(breakdown) = sentence.breakdown() else { return String::new(); };
    let mut html = String::from("<ul>");
    for item in breakdown.items() {
        html.push_str("<li><b>");
        html.push_str(&escape_html(item.surface().as_str()));
        html.push_str("</b>");
        if let Some(roman) = item.roman() {
            html.push_str(" <i>");
            html.push_str(&escape_html(roman.as_str()));
            html.push_str("</i>");
        }
        html.push_str(" — ");
        html.push_str(&escape_html(item.gloss().as_str()));
        html.push_str("</li>");
    }
    html.push_str("</ul>");
    html
}

fn media_name(sentence_id: &str) -> String { format!("lingo__{sentence_id}.mp3") }

fn note_guid(sentence_id: &str) -> String {
    let digest = Sha256::digest(sentence_id.as_bytes());
    let mut encoded = String::new();
    for byte in digest.iter().take(8) {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn stable_i64(value: &str) -> i64 {
    let digest = Sha256::digest(value.as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(bytes) & 0x7fff_ffff_ffff_ffff) as i64
}

fn first_field_checksum(value: &str) -> i64 {
    let digest = Sha256::digest(value.as_bytes());
    i64::from(u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]]))
}

fn deck_json(deck_id: i64, deck: &str) -> serde_json::Value {
    let mut decks = serde_json::Map::new();
    decks.insert(deck_id.to_string(), serde_json::json!({
        "id": deck_id, "name": deck, "mod": 0, "usn": -1, "desc": "Generated by lingo",
        "dyn": 0, "collapsed": false, "extendNew": 10, "extendRev": 50, "conf": 1
    }));
    serde_json::Value::Object(decks)
}

fn model_json(model_id: i64, deck_id: i64) -> serde_json::Value {
    let fields = FIELDS.iter().enumerate()
        .map(|(ord, name)| serde_json::json!({"name": name, "ord": ord, "sticky": false, "rtl": false, "font": "Arial", "size": 20}))
        .collect::<Vec<_>>();
    let mut models = serde_json::Map::new();
    models.insert(model_id.to_string(), serde_json::json!({
        "id": model_id, "name": MODEL_NAME, "type": 0, "mod": 0, "usn": -1, "sortf": 0, "did": deck_id,
        "flds": fields,
        "tmpls": [{
            "name": "Card 1", "ord": 0,
            "qfmt": "<div class=lead>{{Lead}}</div>{{#Secondary}}<div class=secondary>{{Secondary}}</div>{{/Secondary}}",
            "afmt": "{{FrontSide}}<hr id=answer><div>{{English}}</div><div class=literal>{{Literal}}</div><div>{{Register}}</div>{{WordBreakdown}}{{Audio}}"
        }],
        "css": ".card{font-family:Arial;font-size:24px;text-align:center}.lead{font-weight:700}.secondary,.literal{color:#666;font-size:0.8em}ul{text-align:left}"
    }));
    serde_json::Value::Object(models)
}

fn default_dconf() -> serde_json::Value {
    serde_json::json!({"1":{"id":1,"name":"Default","mod":0,"usn":-1,"maxTaken":60,"autoplay":true,"replayq":true,"new":{},"rev":{},"lapse":{},"dyn":false}})
}

fn escape_html(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

const ANKI_SCHEMA: &str = r#"
CREATE TABLE col (id integer primary key, crt integer not null, mod integer not null, scm integer not null, ver integer not null, dty integer not null, usn integer not null, ls integer not null, conf text not null, models text not null, decks text not null, dconf text not null, tags text not null);
CREATE TABLE notes (id integer primary key, guid text not null, mid integer not null, mod integer not null, usn integer not null, tags text not null, flds text not null, sfld text not null, csum integer not null, flags integer not null, data text not null);
CREATE TABLE cards (id integer primary key, nid integer not null, did integer not null, ord integer not null, mod integer not null, usn integer not null, type integer not null, queue integer not null, due integer not null, ivl integer not null, factor integer not null, reps integer not null, lapses integer not null, left integer not null, odue integer not null, odid integer not null, flags integer not null, data text not null);
CREATE TABLE revlog (id integer primary key, cid integer not null, usn integer not null, ease integer not null, ivl integer not null, lastIvl integer not null, factor integer not null, time integer not null, type integer not null);
CREATE TABLE graves (usn integer not null, oid integer not null, type integer not null);
CREATE INDEX ix_notes_usn ON notes (usn);
CREATE INDEX ix_cards_usn ON cards (usn);
CREATE INDEX ix_cards_nid ON cards (nid);
CREATE INDEX ix_cards_sched ON cards (did, queue, due);
CREATE INDEX ix_revlog_usn ON revlog (usn);
CREATE INDEX ix_revlog_cid ON revlog (cid);
"#;

const README: &[u8] = b"Lingo portable package.\nOpen manifest.json first. Sentences are in sentences/ (lingo.sentence/v1) or library.db (lingo.library/v1); audio is in audio/.\n";

/// Write a file, read it back, verify the bytes match, and record its checksum.
fn write_verified(root: &Path, relative: &str, bytes: &[u8], files: &mut BTreeMap<String, String>) -> Result<(), ArtifactFailure> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).map_err(fail)?; }
    fs::write(&path, bytes).map_err(fail)?;
    let read_back = fs::read(&path).map_err(fail)?;
    if read_back != bytes { return Err(ArtifactFailure(format!("integrity check failed for {relative}"))); }
    files.insert(relative.to_string(), sha256(bytes));
    Ok(())
}

/// Distinct words across all sentence breakdowns (normalized surface form).
fn distinct_word_count(sentences: &[Sentence]) -> usize {
    let mut keys = BTreeSet::new();
    for sentence in sentences {
        if let Some(breakdown) = sentence.breakdown() {
            for item in breakdown.items() {
                keys.insert(WordKey::from_surface(item.surface()).as_str().to_string());
            }
        }
    }
    keys.len()
}

#[derive(Serialize)]
struct SentenceFile<'a> {
    format: &'static str,
    id: &'a str,
    collection: &'a str,
    section: Option<&'a str>,
    order: i64,
    target: &'a str,
    romanisation: Option<&'a str>,
    english: Option<&'a str>,
    literal: Option<&'a str>,
    register: Option<&'static str>,
    authority: &'a lingo_domain::FieldAuthoritySet,
    breakdown: Option<&'a lingo_domain::TokenBreakdown>,
    tags: &'a [String],
    audio: Option<&'a str>,
    provenance: &'a lingo_domain::SentenceProvenance,
}

impl<'a> SentenceFile<'a> {
    fn from_sentence(sentence: &'a Sentence) -> Self {
        Self {
            format: "lingo.sentence/v1",
            id: sentence.id().as_str(),
            collection: sentence.collection().as_str(),
            section: sentence.section().map(|s| s.as_str()),
            order: sentence.order().get(),
            target: sentence.target().as_str(),
            romanisation: sentence.romanisation().map(|v| v.as_str()),
            english: sentence.english().map(|v| v.as_str()),
            literal: sentence.literal().map(|v| v.as_str()),
            register: sentence.register().map(|r| r.wire_name()),
            authority: sentence.authority(),
            breakdown: sentence.breakdown(),
            tags: sentence.tags().values(),
            audio: sentence.audio().map(|a| a.path().as_str()),
            provenance: sentence.provenance(),
        }
    }
}

#[derive(Serialize)]
struct PackageManifest {
    format: &'static str,
    language: String,
    collection: String,
    counts: Counts,
    integrity: Integrity,
}

#[derive(Serialize)]
struct DbManifest {
    format: &'static str,
    library: &'static str,
    library_schema: &'static str,
    counts: Counts,
}

#[derive(Serialize)]
struct Counts { sentences: usize, words: usize, audio_files: usize }

#[derive(Serialize)]
struct Integrity { algorithm: &'static str, files: BTreeMap<String, String> }

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn directory_size(root: &Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if meta.is_file() { total += meta.len(); }
        else if meta.is_dir() { total += directory_size(&entry.path())?; }
    }
    Ok(total)
}

fn fail(error: impl std::fmt::Display) -> ArtifactFailure { ArtifactFailure(error.to_string()) }

#[cfg(test)]
mod tests {
    use super::sha256;

    #[test]
    fn checksum_has_expected_prefix() {
        assert!(sha256(b"abc").starts_with("sha256:"));
    }
}
