"""Input parsing, existing-output scanning, and pending batch planning."""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path

from pipeline_config import PIPELINES

BATCH_RE = re.compile(r"^(?P<stem>.+)_batch_(?P<num>\d+)\.json$")


@dataclass(frozen=True)
class BatchMetadata:
    """Source title/subtitle metadata carried through planning and batch CSVs."""

    title: str | None
    subtitle: str | None
    display_label: str

    @classmethod
    def from_parts(cls, title: str | None, subtitle: str | None, fallback_label: str) -> "BatchMetadata":
        if not title and not subtitle and fallback_label:
            title, subtitle = split_title_subtitle(fallback_label)
        display_label = metadata_label(title, subtitle) or fallback_label
        return cls(title=title, subtitle=subtitle, display_label=display_label)

    @classmethod
    def from_label(cls, label: str | None, fallback_label: str | None = None) -> "BatchMetadata":
        title, subtitle = split_title_subtitle(label)
        return cls.from_parts(title, subtitle, fallback_label or label or "")

    def heading_lines(self) -> list[str]:
        lines = []
        if self.title:
            lines.append(f"# {self.title}")
        if self.subtitle:
            lines.append(f"## {self.subtitle}")
        return lines


def load_json_file(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def has_top_metadata_heading(path: Path) -> bool:
    """Return True when the first non-empty line is a non-empty metadata heading."""
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        return line.startswith("#") and bool(line.lstrip("#").strip())
    return False


def split_title_subtitle(label: str | None) -> tuple[str | None, str | None]:
    """Split a display label like 'Complete Hindi Chapter 02' into title/subtitle."""
    if not label:
        return None, None
    clean = " ".join(label.replace(",", " ").split())
    match = re.match(r"^(?P<title>.+?)\s+(?P<subtitle>Chapter\s+\d+.*)$", clean, re.IGNORECASE)
    if match:
        return match.group("title").strip(), match.group("subtitle").strip()
    return label.strip(), None


def metadata_label(title: str | None, subtitle: str | None) -> str | None:
    """Build the display label used for planning and CLI output."""
    if title and subtitle:
        return f"{title} {subtitle}"
    return title or subtitle


def parse_csv_metadata(path: Path) -> tuple[BatchMetadata, list[str]]:
    """Return structured source metadata and content lines."""
    title = None
    subtitle = None
    lines = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("##"):
            subtitle = line.lstrip("#").strip()
        elif line.startswith("#"):
            heading = line.lstrip("#").strip()
            parsed_title, parsed_subtitle = split_title_subtitle(heading)
            title = parsed_title
            if parsed_subtitle:
                subtitle = parsed_subtitle
        else:
            lines.append(line)
    return BatchMetadata.from_parts(title, subtitle, label_from_stem(path.stem)), lines


def parse_csv(path: Path) -> tuple[str | None, list[str]]:
    """Return (source_display_label_or_None, [content_lines])."""
    metadata, lines = parse_csv_metadata(path)
    return metadata.display_label, lines


def label_from_stem(stem: str) -> str:
    """Build a readable fallback label from an input filename."""
    name = stem
    for suffix in ("_words", "_word", "_sentences", "_sentence"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
            break
    name = name.replace("_", " ").replace("-", " ").strip()
    parts = [part.capitalize() if not part.isupper() else part for part in name.split()]
    return " ".join(parts)


def make_batches(items: list[str], size: int) -> list[list[str]]:
    return [items[i : i + size] for i in range(0, len(items), size)]


def build_batch_csv(display_label: str | None, items: list[str]) -> str:
    metadata = BatchMetadata.from_label(display_label)
    lines = metadata.heading_lines() + items
    return "\n".join(lines)


def build_batch_csv_from_metadata(metadata: BatchMetadata, items: list[str]) -> str:
    lines = metadata.heading_lines() + items
    return "\n".join(lines)


def batch_file_path(pipeline_type: str, stem: str, batch_num: int) -> Path:
    return PIPELINES[pipeline_type]["output"] / f"{stem}_batch_{batch_num:02d}.json"


def iter_batch_paths(pipeline_type: str, stem: str):
    output_dir = PIPELINES[pipeline_type]["output"]
    for path in sorted(output_dir.glob(f"{stem}_batch_*.json")):
        match = BATCH_RE.match(path.name)
        if match and match.group("stem") == stem:
            yield path, int(match.group("num"))


def parse_input_item(line: str) -> tuple[str, str, str]:
    """Parse one input CSV content line into a comparable identity tuple."""
    if ";" not in line:
        return line.strip(), "", ""
    left, english = line.split(";", 1)
    left = left.strip()
    english = english.strip()

    match = re.match(r"^(.*?)\s*\((.*?)\)\s*$", left)
    if not match:
        return left, "", english
    hindi = match.group(1).strip()
    roman = match.group(2).strip()
    return hindi, roman, english


def load_existing_entries(pipeline_type: str, stem: str) -> list[dict]:
    key = "words" if pipeline_type == "words" else "sentences"
    entries: list[dict] = []
    for path, _ in iter_batch_paths(pipeline_type, stem):
        try:
            data = load_json_file(path)
        except json.JSONDecodeError:
            continue
        entries.extend(data.get(key, []))
    return entries


def load_existing_output_state(pipeline_type: str, stem: str) -> tuple[set[tuple[str, str, str]], int]:
    """
    Return (processed_identities, max_batch_num) for existing output batches.

    Identity is compared using (hindi, romanisation/roman, english).
    """
    identities: set[tuple[str, str, str]] = set()
    batch_nums: list[int] = []

    for path, batch_num in iter_batch_paths(pipeline_type, stem):
        batch_nums.append(batch_num)
        try:
            data = load_json_file(path)
        except json.JSONDecodeError:
            continue
        key = "words" if pipeline_type == "words" else "sentences"
        for item in data.get(key, []):
            identities.add((
                str(item.get("hindi", "")).strip(),
                str(item.get("romanisation", "") or item.get("roman", "")).strip(),
                str(item.get("english", "")).strip(),
            ))

    if batch_nums:
        expected = list(range(1, max(batch_nums) + 1))
        if sorted(batch_nums) != expected:
            raise ValueError(
                f"Output batches for {pipeline_type}/{stem} are not contiguous. "
                f"Found {sorted(batch_nums)}, expected {expected}."
            )

    return identities, (max(batch_nums) if batch_nums else 0)


def pending_batches_for(pipeline_type: str, batch_size: int, force: bool) -> list[dict]:
    cfg = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir = cfg["input"]

    if not prompt_file.exists():
        print(f"Warning: prompt file not found: {prompt_file}", file=sys.stderr)
        return []

    pending = []
    for csv_path in sorted(input_dir.glob("*.csv")):
        stem = csv_path.stem
        metadata, items = parse_csv_metadata(csv_path)
        if not items:
            continue

        try:
            existing_identities, max_batch_num = load_existing_output_state(pipeline_type, stem)
        except ValueError as exc:
            print(f"Error: {exc}", file=sys.stderr)
            sys.exit(1)

        pending_items = items if force else [
            line for line in items
            if parse_input_item(line) not in existing_identities
        ]
        if not pending_items:
            continue

        batches = make_batches(pending_items, batch_size)
        total_batches = max_batch_num + len(batches)

        for offset, batch in enumerate(batches, 1):
            pending.append({
                "type": pipeline_type,
                "stem": stem,
                "csv_path": str(csv_path),
                "batch_num": max_batch_num + offset,
                "total_batches": total_batches,
                "display_label": metadata.display_label,
                "title": metadata.title,
                "subtitle": metadata.subtitle,
                "has_metadata_heading": has_top_metadata_heading(csv_path),
                "csv": build_batch_csv_from_metadata(metadata, batch),
                "count": len(batch),
            })

    return pending
