use crate::checksum::sha256;
use crate::error::{ArtifactError, io_at};
use crate::manifest::PackageManifest;
use crate::model::{ArtifactFile, ArtifactPath, PublicationPlan};
use crate::staging::publish_directory;
use lingo_application::ports::{
    ArtifactFailure, PackagePublisher, PublishMaterial, PublishedArtifact,
};
use lingo_domain::{Card, CardToken, SourceRef, Word};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default)]
pub struct PortablePackagePublisher;

impl PackagePublisher for PortablePackagePublisher {
    fn publish(
        &self,
        destination: &Path,
        material: &PublishMaterial,
    ) -> Result<PublishedArtifact, ArtifactFailure> {
        let plan = build_plan(material)?;
        let payload_files = plan.files().len();
        let payload_bytes = plan.total_bytes();
        publish_directory(destination, |stage| {
            write_and_verify(stage, material, &plan)
        })?;
        let manifest_bytes = fs::metadata(destination.join("manifest.json"))
            .map_err(|error| io_at(destination.join("manifest.json"), error))?
            .len();
        Ok(PublishedArtifact {
            path: destination.to_path_buf(),
            files: payload_files + 1,
            bytes: payload_bytes + manifest_bytes,
        })
    }
}

fn build_plan(material: &PublishMaterial) -> Result<PublicationPlan, ArtifactError> {
    let mut files = Vec::new();
    let mut stream = Vec::new();
    for batch in &material.batches {
        for card in batch.cards.cards() {
            let audio_path = package_audio_path(card)?;

            // One self-contained file per sentence: cards/<batch>/<item>.json.
            let card_file = PackageCardFile {
                format: "lingo.package-card/v1",
                batch: batch.cards.batch_id().as_str(),
                title: batch.cards.title().as_str(),
                subtitle: batch.cards.subtitle().map(|value| value.as_str()),
                card: PackageCard::from_card(card, audio_path.as_str()),
            };
            let mut bytes = serde_json::to_vec_pretty(&card_file)?;
            bytes.push(b'\n');
            files.push(ArtifactFile {
                path: ArtifactPath::parse(format!(
                    "cards/{}/{}.json",
                    batch.cards.batch_id().as_str(),
                    card.id().source_item().as_str()
                ))?,
                bytes,
            });

            // Aggregate index — one line per sentence across the whole package.
            let flattened = FlattenedCard {
                batch: batch.cards.batch_id().as_str(),
                title: batch.cards.title().as_str(),
                subtitle: batch.cards.subtitle().map(|value| value.as_str()),
                card: PackageCard::from_card(card, audio_path.as_str()),
            };
            serde_json::to_writer(&mut stream, &flattened)?;
            stream.push(b'\n');
        }
        for asset in &batch.audio {
            let path = ArtifactPath::parse(format!(
                "audio/{}/{}.mp3",
                asset.reference.card_id().batch().as_str(),
                asset.reference.card_id().source_item().as_str()
            ))?;
            if sha256(&asset.bytes).as_str() != asset.reference.content_hash().as_str() {
                return Err(ArtifactError::Integrity(path.join_to(Path::new("."))));
            }
            files.push(ArtifactFile {
                path,
                bytes: asset.bytes.clone(),
            });
        }
    }
    files.push(ArtifactFile {
        path: ArtifactPath::parse("cards.jsonl")?,
        bytes: stream,
    });
    files.push(ArtifactFile {
        path: ArtifactPath::parse("README.txt")?,
        bytes: b"Lingo portable sentence package. Read manifest.json first.\n".to_vec(),
    });
    PublicationPlan::try_new(files)
}

fn write_and_verify(
    stage: &Path,
    material: &PublishMaterial,
    plan: &PublicationPlan,
) -> Result<(), ArtifactError> {
    let mut checksums = BTreeMap::new();
    let mut card_files = Vec::new();
    let mut audio_files = Vec::new();
    for file in plan.files() {
        let target = file.path.join_to(stage);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| io_at(parent, error))?;
        }
        fs::write(&target, &file.bytes).map_err(|error| io_at(&target, error))?;
        let read_back = fs::read(&target).map_err(|error| io_at(&target, error))?;
        if read_back != file.bytes {
            return Err(ArtifactError::Integrity(target));
        }
        checksums.insert(file.path.as_str().to_string(), sha256(&read_back));
        if file.path.as_str().starts_with("cards/") {
            card_files.push(file.path.as_str().to_string());
        } else if file.path.as_str().starts_with("audio/") {
            audio_files.push(file.path.as_str().to_string());
        }
    }
    let manifest = PackageManifest::new(material, card_files, audio_files, checksums);
    let mut bytes = serde_json::to_vec_pretty(&manifest)?;
    bytes.push(b'\n');
    let manifest_path = stage.join("manifest.json");
    fs::write(&manifest_path, &bytes).map_err(|error| io_at(&manifest_path, error))?;
    let decoded: PackageManifest = serde_json::from_slice(
        &fs::read(&manifest_path).map_err(|error| io_at(&manifest_path, error))?,
    )?;
    if decoded.format() != "lingo.package/v1" {
        return Err(ArtifactError::Integrity(manifest_path));
    }
    Ok(())
}

fn package_audio_path(card: &Card) -> Result<ArtifactPath, ArtifactError> {
    if card.audio().is_none() {
        return Err(ArtifactError::Invalid(format!(
            "card {} has no audio",
            card.id()
        )));
    }
    ArtifactPath::parse(format!(
        "audio/{}/{}.mp3",
        card.id().batch().as_str(),
        card.id().source_item().as_str()
    ))
}

#[derive(Serialize)]
struct PackageCardFile<'a> {
    format: &'static str,
    batch: &'a str,
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<&'a str>,
    card: PackageCard<'a>,
}

#[derive(Serialize)]
struct FlattenedCard<'a> {
    batch: &'a str,
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    subtitle: Option<&'a str>,
    #[serde(flatten)]
    card: PackageCard<'a>,
}

#[derive(Serialize)]
struct PackageCard<'a> {
    id: String,
    target: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    romanisation: Option<&'a str>,
    english: &'a str,
    literal: &'a str,
    register: &'static str,
    tokens: &'a [CardToken],
    words: &'a [Word],
    tags: &'a [String],
    audio: String,
    source: &'a SourceRef,
}

impl<'a> PackageCard<'a> {
    fn from_card(card: &'a Card, audio: &str) -> Self {
        Self {
            id: card.id().to_string(),
            target: card.target().as_str(),
            romanisation: card.romanisation().map(|value| value.as_str()),
            english: card.english().as_str(),
            literal: card.literal().as_str(),
            register: card.register().wire_name(),
            tokens: card.tokens(),
            words: card.words(),
            tags: card.tags().values(),
            audio: audio.to_string(),
            source: card.source(),
        }
    }
}
