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
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

try:
    from rich.console import Console
    from rich.panel import Panel
    from rich.table import Table
except ModuleNotFoundError as exc:
    missing = exc.name or "a required dependency"
    raise SystemExit(
        f"Missing dependency: {missing}. Run this script with 'uv run main.py ...' "
        "so uv can install the declared inline dependencies."
    ) from exc

import generate
import process


console = Console()


@dataclass
class StemPlan:
    pipeline_type: str
    stem: str
    chapter: str
    skipped_items: list[str] = field(default_factory=list)
    planned_jobs: list[generate.BatchJob] = field(default_factory=list)
    deferred_jobs: list[generate.BatchJob] = field(default_factory=list)
    existing_batches: int = 0
    missing_sound_alikes: list[str] = field(default_factory=list)
    missing_audio: list[str] = field(default_factory=list)
    missing_sentence_tokens: list[str] = field(default_factory=list)
    missing_chapter_heading: bool = False


def item_label(line: str) -> str:
    hindi, roman, english = process.parse_input_item(line)
    left = hindi
    if roman:
        left += f" ({roman})"
    if english:
        left += f" — {english}"
    return left


def existing_entries(pipeline_type: str, stem: str) -> list[dict]:
    return process.load_existing_entries(pipeline_type, stem)


def build_stem_plans(
    pipeline_type: Optional[str],
    batch_size: int,
    force: bool,
    max_items: Optional[int],
    max_batches: Optional[int],
) -> tuple[list[StemPlan], list[generate.BatchJob]]:
    types = [pipeline_type] if pipeline_type else list(process.PIPELINES.keys())
    stem_plans: list[StemPlan] = []
    all_jobs: list[generate.BatchJob] = []
    plan_by_key: dict[tuple[str, str], StemPlan] = {}

    for current_type in types:
        input_dir = process.PIPELINES[current_type]["input"]
        for csv_path in sorted(input_dir.glob("*.csv")):
            chapter, items = process.parse_csv(csv_path)
            if not items:
                continue

            identities, existing_batches = process.load_existing_output_state(current_type, csv_path.stem)
            stem_plan = StemPlan(
                pipeline_type=current_type,
                stem=csv_path.stem,
                chapter=chapter or process.chapter_from_stem(csv_path.stem),
                existing_batches=existing_batches,
                missing_chapter_heading=not process.has_top_chapter_heading(csv_path),
            )

            pending: list[str] = []
            for line in items:
                if not force and process.parse_input_item(line) in identities:
                    stem_plan.skipped_items.append(item_label(line))
                else:
                    pending.append(line)

            if current_type == "words":
                for entry in existing_entries(current_type, csv_path.stem):
                    if not entry.get("sound_alikes"):
                        stem_plan.missing_sound_alikes.append(entry.get("hindi", ""))
                    if not entry.get("audio"):
                        stem_plan.missing_audio.append(entry.get("hindi", ""))
            else:
                for entry in existing_entries(current_type, csv_path.stem):
                    if not entry.get("tokens"):
                        stem_plan.missing_sentence_tokens.append(entry.get("english") or entry.get("hindi", ""))
                    if not entry.get("audio"):
                        stem_plan.missing_audio.append(entry.get("english") or entry.get("hindi", ""))

            batches = process.make_batches(pending, batch_size)
            total_batches = existing_batches + len(batches)
            for offset, batch in enumerate(batches, 1):
                job = generate.BatchJob(
                    pipeline_type=current_type,
                    stem=csv_path.stem,
                    batch_num=existing_batches + offset,
                    total_batches=total_batches,
                    chapter=stem_plan.chapter,
                    csv=process.build_batch_csv(stem_plan.chapter, batch),
                    count=len(batch),
                )
                all_jobs.append(job)

            stem_plans.append(stem_plan)
            plan_by_key[(current_type, csv_path.stem)] = stem_plan

    selected_jobs = generate.limit_jobs(all_jobs, max_items, max_batches)
    selected_keys = {(job.pipeline_type, job.stem, job.batch_num) for job in selected_jobs}

    for job in all_jobs:
        stem_plan = plan_by_key[(job.pipeline_type, job.stem)]
        if (job.pipeline_type, job.stem, job.batch_num) in selected_keys:
            stem_plan.planned_jobs.append(job)
        else:
            stem_plan.deferred_jobs.append(job)

    return stem_plans, selected_jobs


def render_check(
    stem_plans: list[StemPlan],
    selected_jobs: list[generate.BatchJob],
    batch_size: int,
    max_items: Optional[int],
    max_batches: Optional[int],
) -> None:
    console.print(
        Panel.fit(
            f"[bold]Hindi Generator Check[/bold]\n"
            f"Batch size: [cyan]{batch_size}[/cyan]"
            f"  |  Max items: [cyan]{max_items if max_items is not None else '∞'}[/cyan]"
            f"  |  Max batches: [cyan]{max_batches if max_batches is not None else '∞'}[/cyan]",
            border_style="blue",
        )
    )

    summary = Table(title="Plan Summary", header_style="bold cyan")
    summary.add_column("Type")
    summary.add_column("Chapter / Stem")
    summary.add_column("Existing", justify="right")
    summary.add_column("Skip", justify="right")
    summary.add_column("Plan", justify="right")
    summary.add_column("Defer", justify="right")
    summary.add_column("Gaps")

    for stem_plan in stem_plans:
        if not (
            stem_plan.skipped_items
            or stem_plan.planned_jobs
            or stem_plan.deferred_jobs
            or stem_plan.missing_sound_alikes
            or stem_plan.missing_audio
            or stem_plan.missing_sentence_tokens
            or stem_plan.missing_chapter_heading
        ):
            continue
        gaps: list[str] = []
        if stem_plan.missing_chapter_heading:
            gaps.append("chapter_heading")
        if stem_plan.missing_sound_alikes:
            gaps.append(f"sound_alikes:{len(stem_plan.missing_sound_alikes)}")
        if stem_plan.missing_sentence_tokens:
            gaps.append(f"tokens:{len(stem_plan.missing_sentence_tokens)}")
        if stem_plan.missing_audio:
            gaps.append(f"audio:{len(stem_plan.missing_audio)}")
        summary.add_row(
            stem_plan.pipeline_type,
            f"{stem_plan.chapter}\n[dim]{stem_plan.stem}[/dim]",
            str(stem_plan.existing_batches),
            str(len(stem_plan.skipped_items)),
            str(sum(job.count for job in stem_plan.planned_jobs)),
            str(sum(job.count for job in stem_plan.deferred_jobs)),
            ", ".join(gaps) if gaps else "—",
        )

    console.print(summary)
    console.print(f"\nPlanned this run: [bold]{len(selected_jobs)}[/bold] batch(es), [bold]{sum(job.count for job in selected_jobs)}[/bold] item(s)")

    for stem_plan in stem_plans:
        if not any([
            stem_plan.planned_jobs,
            stem_plan.skipped_items,
            stem_plan.deferred_jobs,
            stem_plan.missing_sound_alikes,
            stem_plan.missing_audio,
            stem_plan.missing_sentence_tokens,
            stem_plan.missing_chapter_heading,
        ]):
            continue

        console.print(f"\n[bold]{stem_plan.chapter}[/bold]  [dim]({stem_plan.pipeline_type}/{stem_plan.stem})[/dim]")

        if stem_plan.planned_jobs:
            planned = Table(show_header=True, header_style="green")
            planned.add_column("Planned batch")
            planned.add_column("Items")
            planned.add_column("Preview")
            for job in stem_plan.planned_jobs:
                lines = [line for line in job.csv.splitlines() if not line.startswith("#")]
                preview = "\n".join(item_label(line) for line in lines[: min(3, len(lines))])
                planned.add_row(
                    f"{job.batch_num}/{job.total_batches}",
                    str(job.count),
                    preview,
                )
            console.print(planned)

        if stem_plan.deferred_jobs:
            deferred = ", ".join(
                f"{job.batch_num}/{job.total_batches} ({job.count})" for job in stem_plan.deferred_jobs
            )
            console.print(f"[yellow]Deferred by limits:[/yellow] {deferred}")

        if stem_plan.skipped_items:
            console.print("[cyan]Skip existing:[/cyan]")
            for item in stem_plan.skipped_items:
                console.print(f"  - {item}")

        if stem_plan.missing_chapter_heading:
            console.print(
                "[magenta]Missing chapter/topic heading:[/magenta] "
                "add a non-empty '# ...' line as the first non-empty line."
            )

        if stem_plan.missing_sound_alikes:
            console.print("[magenta]Existing words missing sound_alikes:[/magenta]")
            for hindi in stem_plan.missing_sound_alikes:
                console.print(f"  - {hindi}")

        if stem_plan.missing_sentence_tokens:
            console.print("[magenta]Existing sentences missing exact tokens:[/magenta]")
            for label in stem_plan.missing_sentence_tokens:
                console.print(f"  - {label}")

        if stem_plan.missing_audio:
            console.print("[magenta]Existing entries missing audio:[/magenta]")
            for label in stem_plan.missing_audio:
                console.print(f"  - {label}")


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
