#!/usr/bin/env python3
"""
File management utility for the Hindi word generator pipeline.

Claude Code orchestrates the processing; this script handles the
manifest and reports what work needs to be done.

Commands
--------
check
    Scan input/words/ and input/sentences/ and print a JSON array of
    batches that need processing (new files, changed CSVs, or changed prompt).

mark-done <type> <stem> <batches> <count>
    Record a completed file in the manifest.
    type is "words" or "sentences".

Examples
--------
    uv run process.py check
    uv run process.py check --force
    uv run process.py mark-done words hindi_01 3 28
    uv run process.py mark-done sentences hindi_01 2 15
"""
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import argparse
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent
MANIFEST     = PROJECT_ROOT / "manifest.json"

PIPELINES = {
    "words": {
        "prompt":  PROJECT_ROOT / "generation_prompt_words.txt",
        "input":   PROJECT_ROOT / "input"  / "words",
        "output":  PROJECT_ROOT / "output" / "words",
    },
    "sentences": {
        "prompt":  PROJECT_ROOT / "generation_prompt_sentences.txt",
        "input":   PROJECT_ROOT / "input"  / "sentences",
        "output":  PROJECT_ROOT / "output" / "sentences",
    },
}

DEFAULT_BATCH_SIZE = 10


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_manifest() -> dict:
    if not MANIFEST.exists():
        return {"words": {}, "sentences": {}}
    data = json.loads(MANIFEST.read_text())
    # Migrate flat manifest from earlier version
    if "words" not in data and "sentences" not in data:
        return {"words": data, "sentences": {}}
    return data


def save_manifest(manifest: dict):
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False))


def parse_csv(path: Path) -> tuple[str | None, list[str]]:
    """Return (chapter_title_or_None, [content_lines])."""
    chapter = None
    lines = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            chapter = line.lstrip("#").strip()
        else:
            lines.append(line)
    return chapter, lines


def make_batches(items: list[str], size: int) -> list[list[str]]:
    return [items[i : i + size] for i in range(0, len(items), size)]


def build_batch_csv(chapter: str | None, items: list[str]) -> str:
    lines = ([f"# {chapter}"] if chapter else []) + items
    return "\n".join(lines)


def pending_batches_for(pipeline_type: str, batch_size: int, force: bool, manifest: dict) -> list[dict]:
    cfg         = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir   = cfg["input"]

    if not prompt_file.exists():
        print(f"Warning: prompt file not found: {prompt_file}", file=sys.stderr)
        return []

    prompt_hash = sha256(prompt_file.read_bytes())
    section     = manifest.get(pipeline_type, {})
    csv_files   = sorted(input_dir.glob("*.csv"))
    pending     = []

    for csv_path in csv_files:
        stem     = csv_path.stem
        csv_hash = sha256(csv_path.read_bytes())
        entry    = section.get(stem, {})

        needs_processing = (
            force
            or entry.get("csv_hash")    != csv_hash
            or entry.get("prompt_hash") != prompt_hash
        )

        if not needs_processing:
            continue

        chapter, items = parse_csv(csv_path)
        if not items:
            continue

        batches = make_batches(items, batch_size)

        for i, batch in enumerate(batches, 1):
            pending.append({
                "type":          pipeline_type,
                "stem":          stem,
                "csv_path":      str(csv_path),
                "batch_num":     i,
                "total_batches": len(batches),
                "chapter":       chapter,
                "csv":           build_batch_csv(chapter, batch),
                "count":         len(batch),
            })

    return pending


# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_check(batch_size: int, force: bool, pipeline_type: str | None):
    manifest = load_manifest()
    pending  = []

    types = [pipeline_type] if pipeline_type else list(PIPELINES.keys())
    for t in types:
        pending.extend(pending_batches_for(t, batch_size, force, manifest))

    print(json.dumps(pending, indent=2, ensure_ascii=False))


def cmd_mark_done(pipeline_type: str, stem: str, batches: int, count: int):
    cfg         = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir   = cfg["input"]
    csv_path    = input_dir / f"{stem}.csv"

    if not csv_path.exists():
        print(f"Error: {csv_path} not found", file=sys.stderr)
        sys.exit(1)

    csv_hash    = sha256(csv_path.read_bytes())
    prompt_hash = sha256(prompt_file.read_bytes())
    manifest    = load_manifest()

    manifest.setdefault(pipeline_type, {})[stem] = {
        "csv_hash":     csv_hash,
        "prompt_hash":  prompt_hash,
        "processed_at": datetime.now(timezone.utc).isoformat(),
        "batches":      batches,
        "count":        count,
    }

    save_manifest(manifest)
    label = "words" if pipeline_type == "words" else "sentences"
    print(f"Manifest updated: {pipeline_type}/{stem} ({count} {label}, {batches} batch(es))")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="File management utility for the Hindi word generator pipeline.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    sub = parser.add_subparsers(dest="command", required=True)

    p_check = sub.add_parser("check", help="List batches that need processing.")
    p_check.add_argument("--batch-size", type=int, default=DEFAULT_BATCH_SIZE)
    p_check.add_argument("--force", action="store_true",
                         help="Include all files even if unchanged.")
    p_check.add_argument("--type", choices=["words", "sentences"],
                         help="Check only one pipeline type (default: both).")

    p_done = sub.add_parser("mark-done", help="Record a completed file in the manifest.")
    p_done.add_argument("type",    choices=["words", "sentences"])
    p_done.add_argument("stem",    help="CSV filename without extension, e.g. hindi_01")
    p_done.add_argument("batches", type=int, help="Number of batch files written.")
    p_done.add_argument("count",   type=int, help="Total items processed.")

    args = parser.parse_args()

    # Ensure output dirs exist
    for cfg in PIPELINES.values():
        cfg["output"].mkdir(parents=True, exist_ok=True)

    if args.command == "check":
        cmd_check(args.batch_size, args.force, args.type)
    elif args.command == "mark-done":
        cmd_mark_done(args.type, args.stem, args.batches, args.count)


if __name__ == "__main__":
    main()
