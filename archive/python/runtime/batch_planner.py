"""YAML input parsing, existing-output scanning, and pending batch planning."""

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
    """Source title/subtitle metadata carried through planning and batch YAML."""

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

def load_json_file(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def has_top_metadata_heading(path: Path) -> bool:
    """Compatibility check: YAML sources have explicit title/subtitle fields."""
    metadata, _ = parse_yaml_metadata(path)
    return bool(metadata.title or metadata.subtitle)


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


def _parse_yaml_scalar(raw: str) -> str:
    value = raw.strip()
    if value in ("", "null", "~"):
        return ""
    if value.startswith('"') and value.endswith('"'):
        return str(json.loads(value))
    if value.startswith("'") and value.endswith("'"):
        return value[1:-1].replace("''", "'")
    return value


def _format_yaml_scalar(value: str | None) -> str:
    if value is None:
        return "null"
    return json.dumps(value, ensure_ascii=False)


def parse_yaml_metadata(path: Path) -> tuple[BatchMetadata, list[dict]]:
    """Return structured source metadata and YAML item dictionaries."""
    return _parse_yaml_lines(path.read_text(encoding="utf-8").splitlines(), path.stem, str(path))


def parse_yaml_metadata_from_text(text: str, fallback_stem: str) -> tuple[BatchMetadata, list[dict]]:
    """Return structured source metadata and items from a YAML batch string."""
    return _parse_yaml_lines(text.splitlines(), fallback_stem, fallback_stem)


def _parse_yaml_lines(lines: list[str], fallback_stem: str, source_label: str) -> tuple[BatchMetadata, list[dict]]:
    title = None
    subtitle = None
    items: list[dict] = []
    current: dict | None = None
    in_items = False
    list_key: str | None = None

    for raw in lines:
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        stripped = raw.strip()
        indent = len(raw) - len(raw.lstrip(" "))
        if stripped == "items:":
            in_items = True
            continue
        if not in_items:
            if ":" not in stripped:
                raise ValueError(f"Invalid YAML metadata line in {source_label}: {raw}")
            key, value = stripped.split(":", 1)
            if key == "title":
                title = _parse_yaml_scalar(value)
            elif key == "subtitle":
                parsed = _parse_yaml_scalar(value)
                subtitle = parsed or None
            continue

        if current is not None and list_key and stripped.startswith("- ") and indent > 2:
            current[list_key].append(_parse_yaml_scalar(raw.split("-", 1)[1]))
            continue

        if stripped.startswith("- ") and indent <= 2:
            if current:
                items.append(current)
            current = {}
            list_key = None
            remainder = stripped[2:].strip()
            if remainder:
                if ":" not in remainder:
                    raise ValueError(f"Invalid YAML item line in {source_label}: {raw}")
                key, value = remainder.split(":", 1)
                current[key.strip()] = _parse_yaml_scalar(value)
            continue

        if current is None:
            raise ValueError(f"YAML item field before item in {source_label}: {raw}")
        if ":" not in stripped:
            raise ValueError(f"Invalid YAML item field in {source_label}: {raw}")
        key, value = stripped.split(":", 1)
        key = key.strip()
        parsed = _parse_yaml_scalar(value)
        if key == "tags" and not parsed:
            current[key] = []
            list_key = key
        else:
            current[key] = parsed
            list_key = None

    if current:
        items.append(current)

    normalized = []
    for index, item in enumerate(items, 1):
        normalized.append({
            "hindi": str(item.get("hindi", "")).strip(),
            "romanisation": str(item.get("romanisation", "")).strip(),
            "english": str(item.get("english", "")).strip(),
            **({"tags": item["tags"]} if "tags" in item else {}),
        })
        if not normalized[-1]["hindi"]:
            raise ValueError(f"{source_label}: item {index} is missing hindi")

    return BatchMetadata.from_parts(title, subtitle, label_from_stem(fallback_stem)), normalized


def parse_source_file(path: Path) -> tuple[str | None, list[dict]]:
    """Return (source_display_label_or_None, [yaml_item_dicts])."""
    metadata, items = parse_yaml_metadata(path)
    return metadata.display_label, items


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


def build_batch_yaml(display_label: str | None, items: list[dict]) -> str:
    metadata = BatchMetadata.from_label(display_label)
    return build_batch_yaml_from_metadata(metadata, items)


def build_batch_yaml_from_metadata(metadata: BatchMetadata, items: list[dict]) -> str:
    lines = [
        f"title: {_format_yaml_scalar(metadata.title)}",
        f"subtitle: {_format_yaml_scalar(metadata.subtitle)}",
        "items:",
    ]
    for item in items:
        lines.append(f"  - hindi: {_format_yaml_scalar(item.get('hindi', ''))}")
        lines.append(f"    romanisation: {_format_yaml_scalar(item.get('romanisation', ''))}")
        lines.append(f"    english: {_format_yaml_scalar(item.get('english', ''))}")
    return "\n".join(lines)


def batch_file_path(pipeline_type: str, stem: str, batch_num: int) -> Path:
    return PIPELINES[pipeline_type]["output"] / f"{stem}_batch_{batch_num:02d}.json"


def iter_batch_paths(pipeline_type: str, stem: str):
    output_dir = PIPELINES[pipeline_type]["output"]
    for path in sorted(output_dir.glob(f"{stem}_batch_*.json")):
        match = BATCH_RE.match(path.name)
        if match and match.group("stem") == stem:
            yield path, int(match.group("num"))


def parse_input_item(item: dict) -> tuple[str, str, str]:
    """Return the comparable identity tuple for one YAML source item."""
    return (
        str(item.get("hindi", "")).strip(),
        str(item.get("romanisation", "")).strip(),
        str(item.get("english", "")).strip(),
    )


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
    for source_path in sorted(input_dir.glob("*.yaml")):
        stem = source_path.stem
        metadata, items = parse_yaml_metadata(source_path)
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
                "source_path": str(source_path),
                "batch_num": max_batch_num + offset,
                "total_batches": total_batches,
                "display_label": metadata.display_label,
                "title": metadata.title,
                "subtitle": metadata.subtitle,
                "has_metadata_heading": has_top_metadata_heading(source_path),
                "source": build_batch_yaml_from_metadata(metadata, batch),
                "count": len(batch),
            })

    return pending
