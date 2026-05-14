#!/usr/bin/env python3
"""Migrate legacy Hindi source CSV-like files to structured YAML input."""

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[3]
INPUT_ROOT = PROJECT_ROOT / "input"
ARCHIVE_ROOT = PROJECT_ROOT / "archive" / "python" / "legacy-input"


def yaml_scalar(value: str | None) -> str:
    if value is None:
        return "null"
    return json.dumps(value, ensure_ascii=False)


def split_title_subtitle(label: str | None) -> tuple[str | None, str | None]:
    if not label:
        return None, None
    clean = " ".join(label.replace(",", " ").split())
    match = re.match(r"^(?P<title>.+?)\s+(?P<subtitle>Chapter\s+\d+.*)$", clean, re.IGNORECASE)
    if match:
        return match.group("title").strip(), match.group("subtitle").strip()
    return label.strip(), None


def label_from_stem(stem: str) -> str:
    name = stem
    for suffix in ("_words", "_word", "_sentences", "_sentence"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
            break
    return " ".join(part.capitalize() if not part.isupper() else part for part in name.replace("_", " ").split())


def parse_legacy_line(line: str) -> dict:
    if ";" not in line:
        return {"hindi": line.strip(), "romanisation": "", "english": ""}
    left, english = line.split(";", 1)
    match = re.match(r"^(.*?)\s*\((.*?)\)\s*$", left.strip())
    if not match:
        return {"hindi": left.strip(), "romanisation": "", "english": english.strip()}
    return {
        "hindi": match.group(1).strip(),
        "romanisation": match.group(2).strip(),
        "english": english.strip(),
    }


def parse_legacy_file(path: Path) -> tuple[str | None, str | None, list[dict]]:
    title = None
    subtitle = None
    items = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("##"):
            subtitle = line.lstrip("#").strip()
        elif line.startswith("#"):
            parsed_title, parsed_subtitle = split_title_subtitle(line.lstrip("#").strip())
            title = parsed_title
            if parsed_subtitle:
                subtitle = parsed_subtitle
        else:
            items.append(parse_legacy_line(line))
    if not title and not subtitle:
        title, subtitle = split_title_subtitle(label_from_stem(path.stem))
    return title, subtitle, items


def write_yaml(path: Path, title: str | None, subtitle: str | None, items: list[dict]) -> None:
    lines = [
        f"title: {yaml_scalar(title)}",
        f"subtitle: {yaml_scalar(subtitle)}",
        "items:",
    ]
    for item in items:
        lines.append(f"  - hindi: {yaml_scalar(item['hindi'])}")
        lines.append(f"    romanisation: {yaml_scalar(item['romanisation'])}")
        lines.append(f"    english: {yaml_scalar(item['english'])}")
    path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def migrate_file(path: Path) -> tuple[Path, Path, int]:
    rel = path.relative_to(INPUT_ROOT)
    yaml_path = path.with_suffix(".yaml")
    archive_path = ARCHIVE_ROOT / rel
    if yaml_path.exists():
        raise FileExistsError(f"Refusing to overwrite existing YAML: {yaml_path}")
    title, subtitle, items = parse_legacy_file(path)
    write_yaml(yaml_path, title, subtitle, items)
    archive_path.parent.mkdir(parents=True, exist_ok=True)
    shutil.move(str(path), str(archive_path))
    return yaml_path, archive_path, len(items)


def main() -> None:
    csv_files = sorted(INPUT_ROOT.glob("*/*.csv"))
    if not csv_files:
        print("No legacy CSV-like input files found.")
        return
    for path in csv_files:
        yaml_path, archive_path, count = migrate_file(path)
        print(f"{path} -> {yaml_path} ({count} items)")
        print(f"  archived legacy input: {archive_path}")


if __name__ == "__main__":
    main()
