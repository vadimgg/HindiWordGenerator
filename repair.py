#!/usr/bin/env python3
"""Repair and backfill helpers for generated Hindi card data."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

from batch_planner import parse_csv_metadata
from pipeline_config import PIPELINES, PROJECT_ROOT

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

    return parser.parse_args()


def main() -> None:
    args = parse_args()
    if args.command == "audit":
        raise SystemExit(cmd_audit(args))
    if args.command == "audio":
        raise SystemExit(cmd_audio(args))


if __name__ == "__main__":
    main()
