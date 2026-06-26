use crate::checksum::Checksum;
use lingo_application::ports::PublishMaterial;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct PackageManifest {
    format: String,
    language: ManifestLanguage,
    display: ManifestDisplay,
    counts: ManifestCounts,
    groups: Vec<ManifestGroup>,
    files: ManifestFiles,
    integrity: ManifestIntegrity,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestLanguage {
    name: String,
    code: String,
    script: String,
    romanisation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestDisplay {
    lead: String,
    show_secondary: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestCounts {
    batches: usize,
    cards: usize,
    audio_files: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestGroup {
    batch: String,
    title: String,
    subtitle: Option<String>,
    cards: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestFiles {
    cards: Vec<String>,
    stream: String,
    audio: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ManifestIntegrity {
    algorithm: String,
    files: BTreeMap<String, String>,
}

impl PackageManifest {
    pub fn new(
        material: &PublishMaterial,
        card_files: Vec<String>,
        audio_files: Vec<String>,
        checksums: BTreeMap<String, Checksum>,
    ) -> Self {
        let groups = material
            .batches
            .iter()
            .map(|batch| ManifestGroup {
                batch: batch.cards.batch_id().as_str().to_string(),
                title: batch.cards.title().as_str().to_string(),
                subtitle: batch
                    .cards
                    .subtitle()
                    .map(|value| value.as_str().to_string()),
                cards: batch.cards.cards().len(),
            })
            .collect::<Vec<_>>();
        let cards = groups.iter().map(|group| group.cards).sum();
        Self {
            format: "lingo.package/v1".to_string(),
            language: ManifestLanguage {
                name: material.profile.language().as_str().to_string(),
                code: material.profile.code().as_str().to_string(),
                script: material.profile.script().as_str().to_string(),
                romanisation: material.profile.romanisation().wire_name().to_string(),
            },
            display: ManifestDisplay {
                lead: material.display.lead().wire_name().to_string(),
                show_secondary: material.display.show_secondary(),
            },
            counts: ManifestCounts {
                batches: groups.len(),
                cards,
                audio_files: audio_files.len(),
            },
            groups,
            files: ManifestFiles {
                cards: card_files,
                stream: "cards.jsonl".to_string(),
                audio: audio_files,
            },
            integrity: ManifestIntegrity {
                algorithm: "sha256".to_string(),
                files: checksums
                    .into_iter()
                    .map(|(path, checksum)| (path, checksum.as_str().to_string()))
                    .collect(),
            },
        }
    }

    pub fn format(&self) -> &str {
        &self.format
    }
}
