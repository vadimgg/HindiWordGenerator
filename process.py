#!/usr/bin/env python3
"""
File management utility for the Hindi word generator pipeline.

The main runner orchestrates generation; this script owns batch planning,
schema validation, output writing, and manifest updates.

Commands
--------
check
    Scan input/words/ and input/sentences/ and print a JSON array of
    batches that still need processing.

write <type> <stem> <batch_num> <total_batches> <expected_count> <json_file>
    Validate a generated batch JSON file, write it to the output directory,
    and auto-run mark-done when all batches for the stem are present.
    type is "words" or "sentences".

mark-done <type> <stem> <batches> <count>
    Record a completed file in the manifest.
    type is "words" or "sentences".

Examples
--------
    uv run process.py check
    uv run process.py check --force
    uv run process.py write words hindi_01 3 7 10 /tmp/batch.json
    uv run process.py mark-done words hindi_01 3 28
    uv run process.py mark-done sentences hindi_01 2 15
"""
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import argparse
import json
import sys
from pathlib import Path

from batch_planner import (
    BatchMetadata,
    batch_file_path,
    build_batch_csv,
    build_batch_csv_from_metadata,
    has_top_metadata_heading,
    iter_batch_paths,
    label_from_stem,
    load_existing_entries,
    load_existing_output_state,
    load_json_file,
    make_batches,
    metadata_label,
    parse_csv,
    parse_csv_metadata,
    parse_input_item,
    pending_batches_for,
    split_title_subtitle,
)
from manifest_store import record_processed
from pipeline_config import DEFAULT_BATCH_SIZE, PIPELINES
from schema_validator import ValidationError, validate_and_fix

# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_check(batch_size: int, force: bool, pipeline_type: str | None):
    pending  = []

    types = [pipeline_type] if pipeline_type else list(PIPELINES.keys())
    for t in types:
        pending.extend(pending_batches_for(t, batch_size, force))

    print(json.dumps(pending, indent=2, ensure_ascii=False))


def cmd_write(
    pipeline_type: str,
    stem: str,
    batch_num: int,
    total_batches: int,
    expected_count: int,
    json_path: str,
):
    src        = Path(json_path)

    if not src.exists():
        print(f"Error: {src} not found", file=sys.stderr)
        sys.exit(1)

    try:
        data = load_json_file(src)
    except json.JSONDecodeError as e:
        print(f"Error: invalid JSON in {src}: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        data = validate_and_fix(pipeline_type, data)
    except ValidationError as e:
        print(f"Error: schema validation failed for {src}: {e}", file=sys.stderr)
        sys.exit(1)

    key = "words" if pipeline_type == "words" else "sentences"
    count = len(data.get(key, []))
    if count != expected_count:
        print(
            f"Error: expected {expected_count} {key} in this batch, got {count}",
            file=sys.stderr,
        )
        sys.exit(1)

    out_path = batch_file_path(pipeline_type, stem, batch_num)
    if out_path.exists():
        print(
            f"Error: refusing to overwrite existing output batch: {out_path}",
            file=sys.stderr,
        )
        sys.exit(1)

    out_path.write_text(json.dumps(data, ensure_ascii=False, indent=2))
    print(f"Written: {out_path}")

    # Count items in this batch for reporting
    print(f"  {count} {key} in this batch")

    # Auto mark-done when all batch files are present
    all_present = all(
        batch_file_path(pipeline_type, stem, n).exists()
        for n in range(1, total_batches + 1)
    )
    if all_present:
        total_items = sum(
            len(load_json_file(batch_file_path(pipeline_type, stem, n)).get(key, []))
            for n in range(1, total_batches + 1)
        )
        cmd_mark_done(pipeline_type, stem, total_batches, total_items)
    else:
        present = sum(
            1 for n in range(1, total_batches + 1)
            if batch_file_path(pipeline_type, stem, n).exists()
        )
        print(f"  {present}/{total_batches} batches present — mark-done deferred")


def cmd_mark_done(pipeline_type: str, stem: str, batches: int, count: int):
    cfg         = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir   = cfg["input"]
    csv_path    = input_dir / f"{stem}.csv"

    if not csv_path.exists():
        print(f"Error: {csv_path} not found", file=sys.stderr)
        sys.exit(1)

    record_processed(pipeline_type, stem, csv_path, prompt_file, batches, count)
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

    p_write = sub.add_parser("write", help="Validate a batch JSON and write it to output.")
    p_write.add_argument("type",         choices=["words", "sentences"])
    p_write.add_argument("stem",         help="CSV filename without extension, e.g. hindi_01")
    p_write.add_argument("batch_num",    type=int, help="Batch number (1-based).")
    p_write.add_argument("total_batches",type=int, help="Total number of batches for this stem.")
    p_write.add_argument("expected_count", type=int, help="Expected number of items in the batch.")
    p_write.add_argument("json_file",    help="Path to the raw JSON file from the agent.")

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
    elif args.command == "write":
        cmd_write(
            args.type,
            args.stem,
            args.batch_num,
            args.total_batches,
            args.expected_count,
            args.json_file,
        )
    elif args.command == "mark-done":
        cmd_mark_done(args.type, args.stem, args.batches, args.count)


if __name__ == "__main__":
    main()
