use crate::error::{ArtifactError, io_at};
use crate::staging::publish_file;
use lingo_application::ports::{AnkiExporter, ArtifactFailure, PublishMaterial, PublishedArtifact};
use lingo_domain::{Card, DisplayLead, Word};
use rusqlite::{Connection, params};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;

const MODEL_NAME: &str = "Lingo Sentence";
const FIELDS: &[&str] = &[
    "Lead",
    "Secondary",
    "English",
    "Literal",
    "Register",
    "WordBreakdown",
    "Audio",
    "Source",
];

#[derive(Clone, Copy, Debug, Default)]
pub struct ApkgExporter;

impl AnkiExporter for ApkgExporter {
    fn export(
        &self,
        destination: &Path,
        deck: &str,
        material: &PublishMaterial,
    ) -> Result<PublishedArtifact, ArtifactFailure> {
        publish_file(destination, |stage| build_apkg(stage, deck, material))?;
        let bytes = fs::metadata(destination)
            .map_err(|error| io_at(destination, error))?
            .len();
        Ok(PublishedArtifact {
            path: destination.to_path_buf(),
            files: 1,
            bytes,
        })
    }
}

fn build_apkg(
    destination: &Path,
    deck: &str,
    material: &PublishMaterial,
) -> Result<(), ArtifactError> {
    let directory = tempfile::tempdir().map_err(|error| io_at(destination, error))?;
    let database = directory.path().join("collection.anki2");
    create_collection(&database, deck, material)?;
    let file = File::create(destination).map_err(|error| io_at(destination, error))?;
    let mut archive = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);
    write_file_to_zip(&mut archive, &database, "collection.anki2", options)?;

    let mut media_map = BTreeMap::<String, String>::new();
    let mut media_index = 0usize;
    for batch in &material.batches {
        for asset in &batch.audio {
            let name = media_name(asset.reference.card_id());
            let key = media_index.to_string();
            archive.start_file(&key, options)?;
            archive
                .write_all(&asset.bytes)
                .map_err(|error| io_at(destination, error))?;
            media_map.insert(key, name);
            media_index += 1;
        }
    }
    archive.start_file("media", options)?;
    archive
        .write_all(&serde_json::to_vec(&media_map)?)
        .map_err(|error| io_at(destination, error))?;
    archive.finish()?;

    let mut zip =
        zip::ZipArchive::new(File::open(destination).map_err(|error| io_at(destination, error))?)?;
    if zip.by_name("collection.anki2").is_err() || zip.by_name("media").is_err() {
        return Err(ArtifactError::Integrity(destination.to_path_buf()));
    }
    Ok(())
}

fn create_collection(
    path: &Path,
    deck: &str,
    material: &PublishMaterial,
) -> Result<(), ArtifactError> {
    let mut connection = Connection::open(path)?;
    connection.execute_batch(SCHEMA)?;
    let deck_id = stable_i64(&format!("deck:{deck}"));
    let model_id = stable_i64(MODEL_NAME);
    let decks = deck_json(deck_id, deck);
    let models = model_json(model_id, deck_id);
    connection.execute(
        "INSERT INTO col VALUES (?1,0,0,0,11,0,-1,0,?2,?3,?4,?5,?6)",
        params![
            1_i64,
            "{}",
            models.to_string(),
            decks.to_string(),
            default_dconf().to_string(),
            "{}"
        ],
    )?;

    let transaction = connection.transaction()?;
    let mut due = 1_i64;
    for batch in &material.batches {
        for card in batch.cards.cards() {
            let note_id = stable_i64(&format!("note:{}", card.id()));
            let card_id = stable_i64(&format!("card:{}", card.id()));
            let fields = note_fields(card, material.display);
            let tags = if card.tags().values().is_empty() {
                String::new()
            } else {
                format!(" {} ", card.tags().values().join(" "))
            };
            let checksum = first_field_checksum(&fields[0]);
            transaction.execute(
                "INSERT INTO notes VALUES (?1,?2,?3,0,-1,?4,?5,?6,?7,0,'')",
                params![
                    note_id,
                    note_guid(card),
                    model_id,
                    tags,
                    fields.join("\u{1f}"),
                    fields[0],
                    checksum,
                ],
            )?;
            transaction.execute(
                "INSERT INTO cards VALUES (?1,?2,?3,0,0,-1,0,0,?4,0,0,0,0,0,0,0,0,'')",
                params![card_id, note_id, deck_id, due],
            )?;
            due += 1;
        }
    }
    transaction.commit()?;
    connection.execute_batch("VACUUM;")?;
    Ok(())
}

fn note_fields(card: &Card, display: lingo_domain::DisplayPolicy) -> Vec<String> {
    let target = escape_html(card.target().as_str());
    let romanisation = card
        .romanisation()
        .map(|value| escape_html(value.as_str()))
        .unwrap_or_default();
    let (lead, secondary) = match display.lead() {
        DisplayLead::Target => (target, romanisation),
        DisplayLead::Romanisation if !romanisation.is_empty() => (romanisation, target),
        DisplayLead::Romanisation => (target, String::new()),
    };
    vec![
        lead,
        if display.show_secondary() {
            secondary
        } else {
            String::new()
        },
        escape_html(card.english().as_str()),
        escape_html(card.literal().as_str()),
        card.register().wire_name().to_string(),
        word_breakdown(card.words()),
        card.audio()
            .map(|_| format!("[sound:{}]", media_name(card.id())))
            .unwrap_or_default(),
        escape_html(&format!(
            "{} / {}",
            card.source().batch(),
            card.source().item()
        )),
    ]
}

fn word_breakdown(words: &[Word]) -> String {
    let mut html = String::from("<ul>");
    for word in words {
        html.push_str("<li><b>");
        html.push_str(&escape_html(word.target().as_str()));
        html.push_str("</b>");
        if let Some(romanisation) = word.romanisation() {
            html.push_str(" <i>");
            html.push_str(&escape_html(romanisation.as_str()));
            html.push_str("</i>");
        }
        html.push_str(" — ");
        html.push_str(&escape_html(word.meaning().as_str()));
        html.push_str("</li>");
    }
    html.push_str("</ul>");
    html
}

fn media_name(card: &lingo_domain::CardId) -> String {
    format!(
        "lingo__{}__{}.mp3",
        card.batch().as_str(),
        card.source_item().as_str()
    )
}

fn note_guid(card: &Card) -> String {
    let digest = Sha256::digest(card.id().to_string().as_bytes());
    lower_hex_prefix(&digest, 8)
}

fn lower_hex_prefix(bytes: &[u8], byte_count: usize) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(byte_count.saturating_mul(2));
    for byte in bytes.iter().take(byte_count) {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn stable_i64(value: &str) -> i64 {
    let digest = Sha256::digest(value.as_bytes());
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(bytes) & 0x7fff_ffff_ffff_ffff) as i64
}

fn first_field_checksum(value: &str) -> i64 {
    let digest = Sha256::digest(value.as_bytes());
    i64::from(u32::from_be_bytes([
        digest[0], digest[1], digest[2], digest[3],
    ]))
}

fn deck_json(deck_id: i64, deck: &str) -> serde_json::Value {
    let mut decks = serde_json::Map::new();
    decks.insert(
        deck_id.to_string(),
        json!({
            "id": deck_id, "name": deck, "mod": 0, "usn": -1,
            "desc": "Generated by lingo", "dyn": 0, "collapsed": false,
            "extendNew": 10, "extendRev": 50, "conf": 1
        }),
    );
    serde_json::Value::Object(decks)
}

fn model_json(model_id: i64, deck_id: i64) -> serde_json::Value {
    let fields = FIELDS
        .iter()
        .enumerate()
        .map(|(ord, name)| json!({"name": name, "ord": ord, "sticky": false, "rtl": false, "font": "Arial", "size": 20}))
        .collect::<Vec<_>>();
    let mut models = serde_json::Map::new();
    models.insert(
        model_id.to_string(),
        json!({
            "id": model_id, "name": MODEL_NAME, "type": 0, "mod": 0, "usn": -1,
            "sortf": 0, "did": deck_id,
            "flds": fields,
            "tmpls": [{
                "name": "Card 1", "ord": 0,
                "qfmt": "<div class=lead>{{Lead}}</div>{{#Secondary}}<div class=secondary>{{Secondary}}</div>{{/Secondary}}",
                "afmt": "{{FrontSide}}<hr id=answer><div>{{English}}</div><div class=literal>{{Literal}}</div><div>{{Register}}</div>{{WordBreakdown}}{{Audio}}"
            }],
            "css": ".card{font-family:Arial;font-size:24px;text-align:center}.lead{font-weight:700}.secondary,.literal{color:#666;font-size:0.8em}ul{text-align:left}"
        }),
    );
    serde_json::Value::Object(models)
}

fn default_dconf() -> serde_json::Value {
    json!({"1":{"id":1,"name":"Default","mod":0,"usn":-1,"maxTaken":60,"autoplay":true,"replayq":true,"new":{},"rev":{},"lapse":{},"dyn":false}})
}

fn write_file_to_zip(
    archive: &mut zip::ZipWriter<File>,
    source: &Path,
    name: &str,
    options: SimpleFileOptions,
) -> Result<(), ArtifactError> {
    archive.start_file(name, options)?;
    let mut file = File::open(source).map_err(|error| io_at(source, error))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|error| io_at(source, error))?;
    archive
        .write_all(&buffer)
        .map_err(|error| io_at(source, error))
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

const SCHEMA: &str = r#"
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

#[cfg(test)]
mod tests {
    use super::stable_i64;

    #[test]
    fn stable_ids_do_not_change() {
        assert_eq!(stable_i64("deck:Hindi"), stable_i64("deck:Hindi"));
    }
}
