#!/usr/bin/env python3
"""
Generate one MP3 per word/sentence entry for a batch JSON file and write back
relative `audio` paths into the JSON.
"""
# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "gtts>=2.5.4",
# ]
# ///

import json
import re
import sys
from pathlib import Path

from gtts import gTTS


PROJECT_ROOT = Path(__file__).parent
AUDIO_ROOT = PROJECT_ROOT / "audio"


def safe_name(text: str) -> str:
    text = text.strip().lower()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[-\s]+", "_", text)
    return text.strip("_") or "item"


def synthesize(text: str, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    gTTS(text=text, lang="hi", slow=False).save(path)


def update_batch_audio(batch_path: Path) -> Path:
    data = json.loads(batch_path.read_text(encoding="utf-8"))

    if "words" in data:
        entries = data["words"]
        entry_type = "words"
    elif "sentences" in data:
        entries = data["sentences"]
        entry_type = "sentences"
    else:
        raise ValueError(f"Unsupported batch schema: {batch_path}")

    batch_slug = batch_path.stem
    base_dir = AUDIO_ROOT / entry_type / batch_slug

    for index, entry in enumerate(entries, 1):
        stem_hint = entry.get("romanisation") or entry.get("english") or entry.get("hindi") or str(index)
        filename = f"{index:02d}_{safe_name(stem_hint)}.mp3"
        rel_path = Path("audio") / entry_type / batch_slug / filename
        abs_path = PROJECT_ROOT / rel_path
        synthesize(entry["hindi"], abs_path)
        entry["audio"] = rel_path.as_posix()

    batch_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    return base_dir


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("Usage: uv run audio_generator.py <batch.json>")
    batch_path = Path(sys.argv[1]).resolve()
    update_batch_audio(batch_path)


if __name__ == "__main__":
    main()
