"""Manifest persistence helpers for generated batch outputs."""

from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent
MANIFEST = PROJECT_ROOT / "manifest.json"


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_manifest() -> dict:
    if not MANIFEST.exists():
        return {"words": {}, "sentences": {}}
    data = json.loads(MANIFEST.read_text())
    if "words" not in data and "sentences" not in data:
        return {"words": data, "sentences": {}}
    return data


def save_manifest(manifest: dict) -> None:
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False))


def record_processed(
    pipeline_type: str,
    stem: str,
    csv_path: Path,
    prompt_path: Path,
    batches: int,
    count: int,
) -> None:
    manifest = load_manifest()
    manifest.setdefault(pipeline_type, {})[stem] = {
        "csv_hash": sha256(csv_path.read_bytes()),
        "prompt_hash": sha256(prompt_path.read_bytes()),
        "processed_at": datetime.now(timezone.utc).isoformat(),
        "batches": batches,
        "count": count,
    }
    save_manifest(manifest)
