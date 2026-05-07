#!/usr/bin/env python3
"""
Single entrypoint for checking, generating, and backfilling audio.

Usage:
    uv run main.py check
    uv run main.py run --type words --batch-size 5 --max-items 10
    uv run main.py audio
    uv run main.py audio output/words/some_batch.json
"""
# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "langchain-core>=0.3",
#   "langchain-anthropic>=0.3",
#   "langchain-openai>=0.3",
#   "gtts>=2.5.4",
#   "python-dotenv>=1.0",
#   "rich>=13.0",
# ]
# ///

from __future__ import annotations

import argparse
import asyncio
from pathlib import Path

try:
    from rich.console import Console
except ModuleNotFoundError as exc:
    missing = exc.name or "a required dependency"
    raise SystemExit(
        f"Missing dependency: {missing}. Run this script with 'uv run main.py ...' "
        "so uv can install the declared inline dependencies."
    ) from exc

from check_report import build_stem_plans, render_check
import generate
import process


console = Console()


def cmd_check(args: argparse.Namespace) -> int:
    stem_plans, selected_jobs = build_stem_plans(
        pipeline_type=args.type,
        batch_size=args.batch_size,
        force=args.force,
        max_items=args.max_items,
        max_batches=args.max_batches,
    )
    render_check(stem_plans, selected_jobs, args.batch_size, args.max_items, args.max_batches)
    return 0


def cmd_run(args: argparse.Namespace) -> int:
    return asyncio.run(
        generate.run_pipeline(
            model_string=args.model,
            pipeline_type=args.type,
            batch_size=args.batch_size,
            force=args.force,
            concurrency=args.concurrency,
            max_items=args.max_items,
            max_batches=args.max_batches,
            dry_run=args.dry_run,
            fail_fast=not args.no_fail_fast,
        )
    )


def cmd_audio(args: argparse.Namespace) -> int:
    from audio_generator import update_batch_audio

    if args.path:
        paths = [Path(args.path)]
    else:
        base = process.PIPELINES[args.type]["output"] if args.type else Path("output")
        paths = sorted(base.rglob("*.json"))

    if not paths:
        console.print("[yellow]No batch files found for audio generation.[/yellow]")
        return 0

    for path in paths:
        update_batch_audio(path.resolve())
        console.print(f"[green]✓[/green] audio updated for {path}")
    return 0


def add_common_generation_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("--type", choices=["words", "sentences"], default=None)
    parser.add_argument("--batch-size", type=int, default=process.DEFAULT_BATCH_SIZE)
    parser.add_argument("--max-items", type=int, default=None)
    parser.add_argument("--max-batches", type=int, default=None)
    parser.add_argument("--force", action="store_true")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    p_check = sub.add_parser("check", help="Show exactly what would be processed and skipped.")
    add_common_generation_args(p_check)

    p_run = sub.add_parser("run", help="Run generation.")
    add_common_generation_args(p_run)
    p_run.add_argument("--model", default=generate.DEFAULT_MODEL)
    p_run.add_argument("--concurrency", type=int, default=generate.DEFAULT_CONCURRENCY)
    p_run.add_argument("--dry-run", action="store_true")
    p_run.add_argument("--no-fail-fast", action="store_true")
    p_run.add_argument("--verbose", action="store_true")

    p_audio = sub.add_parser("audio", help="Generate or backfill audio for output batches.")
    p_audio.add_argument("path", nargs="?")
    p_audio.add_argument("--type", choices=["words", "sentences"], default=None)

    return parser.parse_args()


def main() -> None:
    args = parse_args()

    if getattr(args, "batch_size", None) is not None and args.batch_size < 1:
        raise SystemExit("--batch-size must be at least 1")
    if getattr(args, "max_items", None) is not None and args.max_items < 1:
        raise SystemExit("--max-items must be at least 1")
    if getattr(args, "max_batches", None) is not None and args.max_batches < 1:
        raise SystemExit("--max-batches must be at least 1")
    if getattr(args, "concurrency", None) is not None and args.concurrency < 1:
        raise SystemExit("--concurrency must be at least 1")

    if getattr(args, "verbose", False):
        generate.setup_logging(True)
    elif args.command == "run":
        generate.setup_logging(False)

    if args.command == "check":
        raise SystemExit(cmd_check(args))
    if args.command == "run":
        raise SystemExit(cmd_run(args))
    if args.command == "audio":
        raise SystemExit(cmd_audio(args))


if __name__ == "__main__":
    main()
