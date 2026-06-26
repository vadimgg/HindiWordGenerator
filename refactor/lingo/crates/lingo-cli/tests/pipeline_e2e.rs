#![allow(clippy::expect_used, clippy::unwrap_used)]

mod support;

use predicates::prelude::*;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use support::TestWorkspace;

#[test]
fn complete_pipeline_produces_verified_package_and_anki_export() {
    let workspace = TestWorkspace::new();

    workspace
        .command()
        .args(["init", "--lang", "hindi"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));

    let raw = workspace.write_raw("chapter-01.txt", "यह एक किताब है।\n");
    let import_reply = workspace.write_valid_import_reply();
    workspace
        .command()
        .arg("import")
        .arg(&raw)
        .args(["--batch", "chapter-01", "--title", "Chapter 01", "--apply"])
        .arg(&import_reply)
        .assert()
        .success()
        .stdout(predicate::str::contains("lingo build --batch chapter-01"));

    let source_path = workspace.path("input/sentences/chapter-01.yaml");
    let source: serde_yaml::Value = serde_yaml::from_slice(
        &fs::read(&source_path).expect("canonical source should be written"),
    )
    .expect("canonical source should decode");
    assert_eq!(source["format"].as_str(), Some("lingo.source/v1"));
    assert_eq!(source["batch"].as_str(), Some("chapter-01"));
    let source_item = workspace.source_item_id("chapter-01");

    workspace
        .command()
        .arg("import")
        .arg(&raw)
        .args(["--batch", "chapter-01", "--title", "Chapter 01", "--apply"])
        .arg(&import_reply)
        .assert()
        .success()
        .stdout(predicate::str::contains("lingo build --batch chapter-01"));
    assert_eq!(workspace.source_item_id("chapter-01"), source_item);

    let build_reply = workspace.write_valid_build_reply(&source_item);
    workspace
        .command()
        .args(["build", "--batch", "chapter-01", "--apply"])
        .arg(&build_reply)
        .assert()
        .success()
        .stdout(predicate::str::contains("lingo check --batch chapter-01"));

    let card_path = workspace.path("output/sentences/chapter-01.json");
    let cards: Value = serde_json::from_slice(
        &fs::read(&card_path).expect("canonical card batch should be written"),
    )
    .expect("canonical card batch should decode");
    assert_eq!(cards["format"].as_str(), Some("lingo.cards/v1"));
    assert_eq!(
        cards["cards"][0]["source"]["item"].as_str(),
        Some(source_item.as_str())
    );

    workspace
        .command()
        .args(["check", "--batch", "chapter-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s), 1 warning(s)"));

    workspace
        .command()
        .args(["export", "--batch", "chapter-01", "--dest"])
        .arg(workspace.path("exports/missing-audio.apkg"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("has no audio"));

    workspace.install_fake_uv();
    workspace
        .command()
        .args(["audio", "--batch", "chapter-01", "--backend", "gtts"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated 1"));

    workspace
        .command()
        .args(["check", "--batch", "chapter-01"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0 error(s), 0 warning(s)"));

    let package = workspace.path("published-package");
    workspace
        .command()
        .args(["package", "--batch", "chapter-01", "--dest"])
        .arg(&package)
        .assert()
        .success()
        .stdout(predicate::str::contains(package.display().to_string()));
    verify_package(&package);

    let export = workspace.path("exports/chapter-01.apkg");
    workspace
        .command()
        .args(["export", "--batch", "chapter-01", "--dest"])
        .arg(&export)
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
    verify_apkg(&export);

    assert!(workspace.path("config.toml").is_file());
    assert!(!workspace.path("hindi.toml").exists());
    assert!(!workspace.path("input/words").exists());
    assert!(!workspace.path("eval").exists());
}

fn verify_package(root: &std::path::Path) {
    let manifest_path = root.join("manifest.json");
    let manifest: Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("package manifest should exist"))
            .expect("package manifest should decode");
    assert_eq!(manifest["format"].as_str(), Some("lingo.package/v1"));
    assert_eq!(manifest["counts"]["batches"].as_u64(), Some(1));
    assert_eq!(manifest["counts"]["cards"].as_u64(), Some(1));
    assert_eq!(manifest["counts"]["audio_files"].as_u64(), Some(1));
    assert!(root.join("cards/chapter-01.json").is_file());
    assert!(root.join("cards.jsonl").is_file());

    let checksums = manifest["integrity"]["files"]
        .as_object()
        .expect("manifest should contain checksums");
    assert!(!checksums.is_empty());
    for (relative, expected) in checksums {
        let bytes = fs::read(root.join(relative)).expect("checksummed file should exist");
        assert_eq!(
            expected.as_str(),
            Some(sha256(&bytes).as_str()),
            "checksum mismatch for {relative}"
        );
    }
}

fn verify_apkg(path: &std::path::Path) {
    let file = fs::File::open(path).expect("Anki package should exist");
    let mut archive = zip::ZipArchive::new(file).expect("Anki package should be a zip archive");
    assert!(archive.by_name("collection.anki2").is_ok());
    let mut media = String::new();
    archive
        .by_name("media")
        .expect("Anki package should contain a media map")
        .read_to_string(&mut media)
        .expect("media map should be readable");
    let media: Value = serde_json::from_str(&media).expect("media map should be JSON");
    assert_eq!(media.as_object().map(serde_json::Map::len), Some(1));
}

fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::from("sha256:");
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
