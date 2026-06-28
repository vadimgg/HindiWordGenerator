#!/usr/bin/env python3
"""Throw-away converter from the old Grasp sentence package to lingo.package/v1.

This is intentionally a small migration helper, not production package code.
It converts:

  output/sentences/*.json -> cards/*.json
  indexes/sentences.jsonl -> cards.jsonl, rebuilt from converted cards
  audio/sentences/**      -> audio/<batch>/*.mp3

Usage:
  python3 tools/convert_old_grasp_package.py OLD_PACKAGE_DIR NEW_PACKAGE_DIR
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ConvertedBatch:
    batch_id: str
    title: str
    subtitle: str | None
    cards: list[dict[str, Any]]
    card_file: str


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Convert an old Grasp sentence_package_* directory to lingo.package/v1."
    )
    parser.add_argument("old_package", type=Path)
    parser.add_argument("new_package", type=Path)
    parser.add_argument(
        "--force",
        action="store_true",
        help="delete the destination first if it already exists",
    )
    args = parser.parse_args()

    old_package = args.old_package.resolve()
    new_package = args.new_package.resolve()

    if not old_package.is_dir():
        parser.error(f"old package does not exist or is not a directory: {old_package}")
    if new_package.exists():
        if not args.force:
            parser.error(f"destination already exists, pass --force to replace it: {new_package}")
        shutil.rmtree(new_package)

    old_manifest_path = old_package / "manifest.json"
    old_manifest = read_json(old_manifest_path)
    sentence_files = old_manifest.get("files", {}).get("sentences")
    if not isinstance(sentence_files, list) or not sentence_files:
        parser.error(f"old manifest has no files.sentences list: {old_manifest_path}")

    (new_package / "cards").mkdir(parents=True)
    (new_package / "audio").mkdir(parents=True)

    converted_batches: list[ConvertedBatch] = []
    audio_files: list[str] = []

    for relative_sentence_file in sentence_files:
        sentence_path = old_package / relative_sentence_file
        batch = convert_sentence_file(old_package, new_package, sentence_path)
        converted_batches.append(batch)
        for card in batch.cards:
            audio = card.get("audio")
            if isinstance(audio, str):
                audio_files.append(audio)

    card_files = [batch.card_file for batch in converted_batches]
    write_cards_stream(new_package / "cards.jsonl", converted_batches)
    write_readme(new_package / "README.txt")

    integrity_files = card_files + ["cards.jsonl", "README.txt"] + sorted(set(audio_files))
    integrity = {
        relative: sha256_file(new_package / relative)
        for relative in integrity_files
        if (new_package / relative).is_file()
    }

    manifest = {
        "format": "lingo.package/v1",
        "language": {
            "name": "Hindi",
            "code": "hi",
            "script": "Devanagari",
            "romanisation": "iast-tilde",
        },
        "display": {
            "lead": "romanisation",
            "show_secondary": True,
        },
        "counts": {
            "batches": len(converted_batches),
            "cards": sum(len(batch.cards) for batch in converted_batches),
            "audio_files": len(set(audio_files)),
        },
        "groups": [
            {
                "batch": batch.batch_id,
                "title": batch.title,
                **({"subtitle": batch.subtitle} if batch.subtitle else {}),
                "cards": len(batch.cards),
            }
            for batch in converted_batches
        ],
        "files": {
            "cards": card_files,
            "stream": "cards.jsonl",
            "audio": sorted(set(audio_files)),
        },
        "integrity": {
            "algorithm": "sha256",
            "files": integrity,
        },
        "converted_from": {
            "format": old_manifest.get("package_type", "hindi.sentences"),
            "path": str(old_package),
            "created_at_unix": int(time.time()),
        },
    }
    write_json(new_package / "manifest.json", manifest)

    print(f"converted {len(converted_batches)} batches")
    print(f"converted {manifest['counts']['cards']} cards")
    print(f"copied {manifest['counts']['audio_files']} audio files")
    print(new_package)
    return 0


def convert_sentence_file(
    old_package: Path, new_package: Path, sentence_path: Path
) -> ConvertedBatch:
    old_batch = read_json(sentence_path)
    title = require_string(old_batch, "title", sentence_path)
    subtitle = optional_string(old_batch, "subtitle")
    old_sentences = old_batch.get("sentences")
    if not isinstance(old_sentences, list):
        raise ValueError(f"{sentence_path} has no sentences list")

    batch_id = sanitize_batch_id(sentence_path.stem)
    cards: list[dict[str, Any]] = []
    used_item_ids: set[str] = set()

    for index, sentence in enumerate(old_sentences, start=1):
        if not isinstance(sentence, dict):
            raise ValueError(f"{sentence_path} sentence {index} is not an object")
        source_ref = sentence.get("source_ref") if isinstance(sentence.get("source_ref"), dict) else {}
        item_id = sanitize_item_id(source_ref.get("item_id") or f"{index:04}")
        while item_id in used_item_ids:
            item_id = sanitize_item_id(f"{item_id}_{index:04}")
        used_item_ids.add(item_id)

        audio = copy_audio(old_package, new_package, batch_id, item_id, sentence.get("audio"))
        fingerprint = source_ref.get("fingerprint")
        if not is_sha256_fingerprint(fingerprint):
            fingerprint = source_fingerprint(sentence)

        card = {
            "id": f"{batch_id}:{item_id}",
            "target": require_string(sentence, "hindi", sentence_path),
            **optional_field("romanisation", sentence.get("romanisation")),
            "english": require_string(sentence, "english", sentence_path),
            "literal": sentence.get("literal") or sentence.get("english") or "",
            "register": normalize_register(sentence.get("register")),
            "tokens": [convert_token(token) for token in sentence.get("tokens", [])],
            "words": [convert_word(word) for word in sentence.get("words", [])],
            "tags": sentence.get("anki_tags") if isinstance(sentence.get("anki_tags"), list) else [],
            **optional_field("audio", audio),
            "source": {
                "batch": batch_id,
                "item": item_id,
                "fingerprint": fingerprint,
            },
        }
        cards.append(card)

    package_batch = {
        "format": "lingo.package-cards/v1",
        "batch": batch_id,
        "title": title,
        **({"subtitle": subtitle} if subtitle else {}),
        "cards": cards,
    }
    card_file = f"cards/{batch_id}.json"
    write_json(new_package / card_file, package_batch)
    return ConvertedBatch(batch_id, title, subtitle, cards, card_file)


def convert_token(token: Any) -> dict[str, Any]:
    if not isinstance(token, dict):
        return {"target": str(token), "word_id": "w"}
    return {
        "target": token.get("target") or token.get("hindi") or "",
        **optional_field("romanisation", token.get("romanisation") or token.get("roman")),
        "word_id": sanitize_word_id(token.get("word_id") or token.get("id") or "w"),
    }


def convert_word(word: Any) -> dict[str, Any]:
    if not isinstance(word, dict):
        return {
            "id": "w",
            "target": str(word),
            "meaning": "",
            "kind": "other",
            "grammar": [],
        }
    return {
        "id": sanitize_word_id(word.get("id") or "w"),
        "target": word.get("target") or word.get("hindi") or "",
        **optional_field("romanisation", word.get("romanisation") or word.get("roman")),
        "meaning": word.get("meaning") or "",
        "kind": normalize_word_kind(word.get("kind"), word.get("grammar")),
        "grammar": normalize_grammar(word.get("grammar")),
    }


def copy_audio(
    old_package: Path,
    new_package: Path,
    batch_id: str,
    item_id: str,
    old_audio: Any,
) -> str | None:
    if not isinstance(old_audio, str) or not old_audio:
        return None
    source = old_package / old_audio
    if not source.is_file():
        print(f"warning: missing audio: {old_audio}", file=sys.stderr)
        return None
    destination_relative = f"audio/{batch_id}/{item_id}.mp3"
    destination = new_package / destination_relative
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    return destination_relative


def write_cards_stream(path: Path, batches: list[ConvertedBatch]) -> None:
    with path.open("w", encoding="utf-8") as handle:
        for batch in batches:
            for card in batch.cards:
                flattened = {
                    "batch": batch.batch_id,
                    "title": batch.title,
                    **({"subtitle": batch.subtitle} if batch.subtitle else {}),
                    **card,
                }
                handle.write(json.dumps(flattened, ensure_ascii=False, separators=(",", ":")))
                handle.write("\n")


def source_fingerprint(sentence: dict[str, Any]) -> str:
    parts = [
        str(sentence.get("hindi") or "").strip(),
        str(sentence.get("romanisation") or "").strip(),
        str(sentence.get("english") or "").strip(),
    ]
    digest = hashlib.sha256("\n".join(parts).encode("utf-8")).hexdigest()
    return f"sha256:{digest}"


def sanitize_batch_id(raw: Any) -> str:
    return sanitize_id(str(raw), max_len=80, fallback="batch")


def sanitize_item_id(raw: Any) -> str:
    return sanitize_id(str(raw), max_len=48, fallback="item")


def sanitize_word_id(raw: Any) -> str:
    return sanitize_id(str(raw), max_len=32, fallback="w")


def sanitize_id(raw: str, max_len: int, fallback: str) -> str:
    value = raw.strip().lower()
    value = re.sub(r"[^a-z0-9_-]+", "_", value)
    value = re.sub(r"_+", "_", value).strip("_-")
    if not value:
        value = fallback
    return value[:max_len].strip("_-") or fallback


def normalize_register(raw: Any) -> str:
    if isinstance(raw, str) and raw.strip():
        return raw.strip().lower().replace(" ", "_")
    return "neutral"


def normalize_word_kind(raw: Any, grammar: Any) -> str:
    if isinstance(raw, str) and raw.strip():
        return raw.strip().lower().replace(" ", "_")
    if isinstance(grammar, dict):
        pos = grammar.get("pos")
        if isinstance(pos, str) and pos.strip():
            return pos.strip().lower().replace(" ", "_")
    return "other"


def normalize_grammar(raw: Any) -> list[str]:
    if isinstance(raw, list):
        return [str(item) for item in raw if str(item).strip()]
    if isinstance(raw, dict):
        tags = []
        for key, value in raw.items():
            if value is None or value == "":
                continue
            if isinstance(value, bool):
                if value:
                    tags.append(str(key))
            else:
                tags.append(f"{key}:{value}")
        return tags
    if isinstance(raw, str) and raw.strip():
        return [raw.strip()]
    return []


def optional_field(name: str, value: Any) -> dict[str, Any]:
    if isinstance(value, str) and value:
        return {name: value}
    return {}


def require_string(data: dict[str, Any], key: str, path: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value:
        raise ValueError(f"{path} missing required string field: {key}")
    return value


def optional_string(data: dict[str, Any], key: str) -> str | None:
    value = data.get(key)
    if isinstance(value, str) and value:
        return value
    return None


def is_sha256_fingerprint(value: Any) -> bool:
    return (
        isinstance(value, str)
        and value.startswith("sha256:")
        and len(value) == 71
        and all(character in "0123456789abcdefABCDEF" for character in value[7:])
    )


def read_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path} did not contain a JSON object")
    return data


def write_json(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(data, handle, ensure_ascii=False, indent=2)
        handle.write("\n")


def write_readme(path: Path) -> None:
    path.write_text(
        "Lingo portable sentence package converted from old Grasp package.\n"
        "This package is for migration/testing; prefer `lingo package` for new data.\n",
        encoding="utf-8",
    )


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


if __name__ == "__main__":
    raise SystemExit(main())
