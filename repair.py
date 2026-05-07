#!/usr/bin/env python3
"""Repair and backfill helpers for generated Hindi card data."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from batch_planner import parse_csv_metadata
from pipeline_config import PIPELINES, PROJECT_ROOT
from schema_validator import ValidationError, validate_and_fix

SENTENCE_MARKS = ("?", "؟", "।", "!", ".")
COMMON_FINITE_FORMS = {
    "है", "हैं", "हूँ", "हो", "था", "थे", "थी", "थीं",
    "होगा", "होगी", "होंगे", "करता", "करती", "करते",
}


def _load_json(path: Path) -> dict | list:
    return json.loads(path.read_text(encoding="utf-8"))


def _batch_key(data: dict) -> str | None:
    if "words" in data:
        return "words"
    if "sentences" in data:
        return "sentences"
    return None


def _audio_missing(entries: list[dict]) -> int:
    return sum(1 for entry in entries if not entry.get("audio"))


def _sentence_token_missing(entries: list[dict]) -> int:
    return sum(1 for entry in entries if not entry.get("tokens"))


def _sentence_token_legacy(entries: list[dict]) -> int:
    return sum(1 for entry in entries if entry.get("tokens") and _needs_token_repair(entry))


def _needs_token_repair(sentence: dict) -> bool:
    tokens = sentence.get("tokens")
    words = sentence.get("words")
    if not tokens:
        return True
    if not isinstance(tokens, list) or not isinstance(words, list):
        return True
    if len(tokens) != len(words):
        return True
    return any(
        not isinstance(token, dict)
        or token.get("kind") != "word"
        or token.get("word_index") != index
        for index, token in enumerate(tokens)
    )


def _build_word_sentence_tokens(sentence: dict) -> list[dict] | None:
    hindi_text = sentence.get("hindi")
    roman_text = sentence.get("romanisation")
    words = sentence.get("words")
    if not isinstance(hindi_text, str) or not isinstance(roman_text, str) or not isinstance(words, list):
        return None

    tokens: list[dict] = []
    hindi_cursor = 0
    roman_cursor = 0

    for word_index, word in enumerate(words):
        if not isinstance(word, dict):
            return None
        hindi_word = word.get("hindi")
        roman_word = word.get("roman")
        if not isinstance(hindi_word, str) or not isinstance(roman_word, str):
            return None
        if not hindi_word or not roman_word:
            return None

        hindi_pos = hindi_text.find(hindi_word, hindi_cursor)
        roman_pos = roman_text.find(roman_word, roman_cursor)
        if hindi_pos < 0 or roman_pos < 0:
            return None

        tokens.append({
            "hindi": hindi_word,
            "roman": roman_word,
            "kind": "word",
            "word_index": word_index,
        })
        hindi_cursor = hindi_pos + len(hindi_word)
        roman_cursor = roman_pos + len(roman_word)

    return tokens or None


def _repair_sentence_tokens(sentence: dict, force: bool = False) -> bool:
    if not force and not _needs_token_repair(sentence):
        return False
    tokens = _build_word_sentence_tokens(sentence)
    if not tokens:
        return False
    sentence["tokens"] = tokens
    return True


def _audit_output_file(path: Path) -> list[dict]:
    issues: list[dict] = []
    try:
        data = _load_json(path)
    except json.JSONDecodeError as exc:
        return [{"path": str(path), "kind": "invalid-json", "detail": str(exc)}]

    if not isinstance(data, dict):
        return [{"path": str(path), "kind": "invalid-batch", "detail": "top-level JSON is not an object"}]

    if "chapter" in data:
        issues.append({"path": str(path), "kind": "legacy-metadata", "detail": "top-level chapter key"})
    if not data.get("title"):
        issues.append({"path": str(path), "kind": "missing-title", "detail": "top-level title is missing"})
    if "subtitle" not in data:
        issues.append({"path": str(path), "kind": "missing-subtitle", "detail": "top-level subtitle is missing"})

    key = _batch_key(data)
    if not key:
        issues.append({"path": str(path), "kind": "invalid-batch", "detail": "missing words/sentences array"})
        return issues

    entries = data.get(key, [])
    if not isinstance(entries, list):
        issues.append({"path": str(path), "kind": "invalid-batch", "detail": f"{key} is not a list"})
        return issues

    missing_audio = _audio_missing(entries)
    if missing_audio:
        issues.append({"path": str(path), "kind": "missing-audio", "detail": f"{missing_audio} {key} missing audio"})

    if key == "sentences":
        missing_tokens = _sentence_token_missing(entries)
        if missing_tokens:
            issues.append({"path": str(path), "kind": "missing-tokens", "detail": f"{missing_tokens} sentences missing tokens"})
        legacy_tokens = _sentence_token_legacy(entries)
        if legacy_tokens:
            issues.append({"path": str(path), "kind": "legacy-tokens", "detail": f"{legacy_tokens} sentences have non-word tokens"})

    return issues


def _looks_like_sentence_drill(line: str) -> bool:
    source = line.split(";", 1)[0].strip()
    hindi = re.sub(r"\([^)]*\)", "", source).strip()
    words = [part for part in re.split(r"\s+", hindi) if part]
    if not words:
        return False
    if any(hindi.endswith(mark) for mark in SENTENCE_MARKS):
        return False
    return not any(word in COMMON_FINITE_FORMS for word in words)


def _audit_sentence_input(path: Path) -> list[dict]:
    _, lines = parse_csv_metadata(path)
    issues = []
    for index, line in enumerate(lines, 1):
        if _looks_like_sentence_drill(line):
            issues.append({
                "path": str(path),
                "kind": "sentence-input-phrase-drill",
                "detail": f"content line {index} looks phrase-like: {line}",
            })
    return issues


def output_paths(pipeline_type: str | None) -> list[Path]:
    if pipeline_type:
        return sorted(PIPELINES[pipeline_type]["output"].glob("*.json"))
    return sorted((PROJECT_ROOT / "output").rglob("*.json"))


def audit(pipeline_type: str | None, include_inputs: bool) -> list[dict]:
    issues: list[dict] = []
    for path in output_paths(pipeline_type):
        issues.extend(_audit_output_file(path))

    if include_inputs and pipeline_type in (None, "sentences"):
        for path in sorted(PIPELINES["sentences"]["input"].glob("*.csv")):
            issues.extend(_audit_sentence_input(path))

    return issues


def cmd_audit(args: argparse.Namespace) -> int:
    issues = audit(args.type, args.inputs)
    print(json.dumps({"issues": issues, "count": len(issues)}, ensure_ascii=False, indent=2))
    return 1 if issues and args.fail_on_issues else 0


def cmd_audio(args: argparse.Namespace) -> int:
    from audio_generator import update_batch_audio

    candidates = [Path(args.path)] if args.path else output_paths(args.type)
    updated = 0
    for path in candidates:
        issues = [issue for issue in _audit_output_file(path) if issue["kind"] == "missing-audio"]
        if not issues and not args.force:
            continue
        update_batch_audio(path.resolve())
        print(f"audio updated: {path}")
        updated += 1
    print(f"{updated} batch file(s) updated")
    return 0


def cmd_tokens(args: argparse.Namespace) -> int:
    candidates = [Path(args.path)] if args.path else output_paths("sentences")
    report = {
        "checked": 0,
        "repairable": 0,
        "updated": 0,
        "unrepairable": [],
        "write": args.write,
    }

    for path in candidates:
        data = _load_json(path)
        if not isinstance(data, dict) or not isinstance(data.get("sentences"), list):
            report["unrepairable"].append({
                "path": str(path),
                "detail": "not a sentence batch",
            })
            continue

        changed = False
        for index, sentence in enumerate(data["sentences"]):
            if not args.force and not _needs_token_repair(sentence):
                continue
            report["checked"] += 1
            if _repair_sentence_tokens(sentence, args.force):
                report["repairable"] += 1
                changed = True
            else:
                report["unrepairable"].append({
                    "path": str(path),
                    "index": index,
                    "hindi": sentence.get("hindi"),
                    "detail": "could not align hindi/romanisation to words in order",
                })

        if changed and args.write:
            try:
                validate_and_fix("sentences", data)
            except ValidationError as exc:
                report["unrepairable"].append({
                    "path": str(path),
                    "detail": f"validation failed after token repair: {exc}",
                })
                continue
            path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            report["updated"] += 1

    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 1 if report["unrepairable"] else 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    p_audit = sub.add_parser("audit", help="Report output/input repair candidates without writing files.")
    p_audit.add_argument("--type", choices=["words", "sentences"], default=None)
    p_audit.add_argument("--inputs", action="store_true", help="Also audit sentence input files for phrase-like drills.")
    p_audit.add_argument("--fail-on-issues", action="store_true")

    p_audio = sub.add_parser("audio", help="Backfill audio for batches with missing audio paths.")
    p_audio.add_argument("path", nargs="?")
    p_audio.add_argument("--type", choices=["words", "sentences"], default=None)
    p_audio.add_argument("--force", action="store_true", help="Regenerate audio even when all entries already have audio paths.")

    p_tokens = sub.add_parser("tokens", help="Backfill word-only sentence tokens when alignment is unambiguous.")
    p_tokens.add_argument("path", nargs="?")
    p_tokens.add_argument("--type", choices=["sentences"], default="sentences")
    p_tokens.add_argument("--write", action="store_true", help="Write repaired token arrays after validation.")
    p_tokens.add_argument("--force", action="store_true", help="Rebuild existing token arrays when word alignment is possible.")

    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.command == "audit":
        raise SystemExit(cmd_audit(args))
    if args.command == "audio":
        raise SystemExit(cmd_audio(args))
    if args.command == "tokens":
        raise SystemExit(cmd_tokens(args))


if __name__ == "__main__":
    main()
