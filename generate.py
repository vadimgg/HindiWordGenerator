#!/usr/bin/env python3
"""
Hindi flashcard generator — runs the full pipeline autonomously.

Reads pending batches from process.py, calls an LLM in parallel, and writes
output — no manual steps required.

Usage:
    uv run generate.py
    uv run generate.py --type words
    uv run generate.py --type sentences
    uv run generate.py --model openai:gpt-5.4-mini
    uv run generate.py --model anthropic:claude-sonnet-4-6
    uv run generate.py --force
    uv run generate.py --dry-run
    uv run generate.py --batch-size 5 --concurrency 10
    uv run generate.py --type words --max-items 50
    uv run generate.py --max-batches 3

Model format:  <provider>:<model-id>
  openai:gpt-5.4-mini
  openai:gpt-5-mini
  anthropic:claude-sonnet-4-6
  anthropic:claude-haiku-4-5-20251001

Environment variables:
  MODEL              Default model string (overridden by --model)
  ANTHROPIC_API_KEY  Required for anthropic provider
  OPENAI_API_KEY     Required for openai provider

You can also store these in a local `.env` file in the project root.
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

import argparse
import asyncio
import logging
import os
import sys
from pathlib import Path
from typing import Optional

from dotenv import load_dotenv
from generation_io import attach_audio, get_pending_batches, write_batch_result
from generation_types import BatchJob, BatchResult
from llm_client import call_llm_with_retry, create_llm, load_prompt

try:
    from rich.console import Console
    from rich.logging import RichHandler
    from rich.panel import Panel
    from rich.progress import (
        BarColumn,
        MofNCompleteColumn,
        Progress,
        SpinnerColumn,
        TaskID,
        TextColumn,
        TimeElapsedColumn,
    )
    from rich.table import Table
except ModuleNotFoundError as exc:
    missing = exc.name or "a required dependency"
    raise SystemExit(
        f"Missing dependency: {missing}. Run this script with 'uv run generate.py ...' "
        "so uv can install the declared inline dependencies."
    ) from exc

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

PROJECT_ROOT = Path(__file__).parent
load_dotenv(PROJECT_ROOT / ".env")

DEFAULT_MODEL = os.environ.get("MODEL", "openai:gpt-5.4-mini")
DEFAULT_CONCURRENCY = 15
DEFAULT_BATCH_SIZE = 10

console = Console()
log = logging.getLogger("generate")


# ---------------------------------------------------------------------------
# Batch processing
# ---------------------------------------------------------------------------

async def process_batch(
    llm,
    job: BatchJob,
    system_prompt: str,
    semaphore: asyncio.Semaphore,
) -> BatchResult:
    """
    Process one BatchJob: acquire semaphore slot, call LLM, return result.

    Never raises — failures are captured in BatchResult.success / .error.
    """
    async with semaphore:
        try:
            data, usage = await call_llm_with_retry(
                llm, system_prompt, job.csv, job.label
            )
            return BatchResult(
                job=job,
                success=True,
                data=data,
                input_tokens=usage["input_tokens"],
                output_tokens=usage["output_tokens"],
                cache_read_tokens=usage["cache_read_tokens"],
                cache_write_tokens=usage["cache_write_tokens"],
            )
        except Exception as exc:
            log.error("[%s] Unrecoverable error: %s", job.label, exc)
            return BatchResult(job=job, success=False, error=str(exc))


def limit_jobs(
    jobs: list[BatchJob],
    max_items: Optional[int],
    max_batches: Optional[int],
) -> list[BatchJob]:
    """
    Limit the run to a prefix of pending jobs without splitting batches.

    `max_items` caps the sum of job.count values and only includes whole batches.
    `max_batches` caps the number of batches.
    """
    selected: list[BatchJob] = []
    items_used = 0

    for job in jobs:
        if max_batches is not None and len(selected) >= max_batches:
            break
        if max_items is not None and selected and items_used + job.count > max_items:
            break
        if max_items is not None and not selected and job.count > max_items:
            raise ValueError(
                f"max-items={max_items} is smaller than the first pending batch "
                f"({job.count} items in {job.label}). Increase --max-items or lower --batch-size."
            )
        selected.append(job)
        items_used += job.count

    return selected


# ---------------------------------------------------------------------------
# Rich display helpers
# ---------------------------------------------------------------------------

def make_progress() -> Progress:
    return Progress(
        SpinnerColumn(),
        TextColumn("[bold]{task.description}"),
        BarColumn(bar_width=36),
        MofNCompleteColumn(),
        TimeElapsedColumn(),
        console=console,
    )


def print_dry_run_table(jobs: list[BatchJob]) -> None:
    table = Table(title="Pending batches (dry run)", header_style="bold cyan")
    table.add_column("Type", style="cyan")
    table.add_column("Stem")
    table.add_column("Batch", justify="right")
    table.add_column("Items", justify="right")
    table.add_column("Source")

    for job in jobs:
        table.add_row(
            job.pipeline_type,
            job.stem,
            f"{job.batch_num}/{job.total_batches}",
            str(job.count),
            job.display_label or "—",
        )

    console.print(table)


def print_summary(results: list[BatchResult]) -> None:
    succeeded = [r for r in results if r.success]
    failed = [r for r in results if not r.success]

    total_in = sum(r.input_tokens for r in succeeded)
    total_out = sum(r.output_tokens for r in succeeded)
    cache_read = sum(r.cache_read_tokens for r in succeeded)
    cache_write = sum(r.cache_write_tokens for r in succeeded)

    table = Table(title="Run summary", header_style="bold")
    table.add_column("Metric", style="cyan", min_width=22)
    table.add_column("Value", justify="right")

    table.add_row("Batches completed", f"[green]{len(succeeded)}[/green]")
    table.add_row(
        "Batches failed",
        f"[red]{len(failed)}[/red]" if failed else "[dim]0[/dim]",
    )

    if total_in or total_out:
        table.add_section()
        table.add_row("Input tokens", f"{total_in:,}")
        table.add_row("Output tokens", f"{total_out:,}")
        if cache_read:
            table.add_row("Cache read tokens", f"[green]{cache_read:,}[/green]")
        if cache_write:
            table.add_row("Cache write tokens", f"{cache_write:,}")
        # Estimated savings: cache_read costs ~10% of input_tokens for Anthropic
        if cache_read and total_in:
            saved = int(cache_read * 0.9)
            table.add_row("Est. tokens saved (cache)", f"[green]~{saved:,}[/green]")

    console.print(table)

    if failed:
        console.print("\n[bold red]Failed batches:[/bold red]")
        for r in failed:
            console.print(f"  [red]✗[/red] {r.job.label}: {r.error}")


def _tokens_info(result: BatchResult) -> str:
    if not result.input_tokens:
        return ""
    parts = [f"{result.input_tokens:,} in / {result.output_tokens:,} out"]
    if result.cache_read_tokens:
        parts.append(f"[green]{result.cache_read_tokens:,} cached[/green]")
    return "  (" + ", ".join(parts) + ")"


# ---------------------------------------------------------------------------
# Pipeline orchestrator
# ---------------------------------------------------------------------------

async def run_pipeline(
    model_string: str,
    pipeline_type: Optional[str],
    batch_size: int,
    force: bool,
    concurrency: int,
    max_items: Optional[int],
    max_batches: Optional[int],
    dry_run: bool,
    fail_fast: bool,
) -> int:
    """
    Main pipeline: check → create LLM → generate all batches → write outputs.

    Returns 0 on full success, 1 if any batch failed.
    """
    console.print(
        Panel.fit(
            f"[bold]Hindi Flashcard Generator[/bold]\n"
            f"Model: [cyan]{model_string}[/cyan]"
            f"  |  Concurrency: [cyan]{concurrency}[/cyan]"
            f"  |  Batch size: [cyan]{batch_size}[/cyan]",
            border_style="blue",
        )
    )

    # ── 1. Discover pending work ─────────────────────────────────────────────
    console.print("\n[bold]Scanning for pending batches...[/bold]")
    try:
        jobs = get_pending_batches(pipeline_type, batch_size, force, DEFAULT_BATCH_SIZE)
    except RuntimeError as exc:
        console.print(f"[red]Error:[/red] {exc}")
        return 1

    try:
        jobs = limit_jobs(jobs, max_items, max_batches)
    except ValueError as exc:
        console.print(f"[red]Limit error:[/red] {exc}")
        return 1

    if not jobs:
        console.print("[green]Nothing to process — all files are up to date.[/green]")
        return 0

    words_count = sum(1 for j in jobs if j.pipeline_type == "words")
    sentences_count = sum(1 for j in jobs if j.pipeline_type == "sentences")
    console.print(
        f"Found [bold]{len(jobs)}[/bold] pending batch(es): "
        f"[cyan]{words_count}[/cyan] word, "
        f"[cyan]{sentences_count}[/cyan] sentence"
    )
    console.print(
        f"Planned item total: [bold]{sum(j.count for j in jobs)}[/bold]"
    )

    if dry_run:
        console.print("\n[yellow]Dry run — no LLM calls will be made.[/yellow]")
        print_dry_run_table(jobs)
        return 0

    # ── 2. Initialise LLM and prompts ────────────────────────────────────────
    console.print(f"\n[bold]Initialising model:[/bold] {model_string}")
    try:
        llm = create_llm(model_string)
    except (ValueError, EnvironmentError) as exc:
        console.print(f"[red]Model error:[/red] {exc}")
        return 1

    prompts: dict[str, str] = {}
    for t in sorted({j.pipeline_type for j in jobs}):
        try:
            prompts[t] = load_prompt(t)
            console.print(f"  Loaded prompt: generation_prompt_{t}.txt")
        except FileNotFoundError as exc:
            console.print(f"[red]Error:[/red] {exc}")
            return 1

    # ── 3. Generate and write ────────────────────────────────────────────────
    results: list[BatchResult] = []

    with make_progress() as progress:
        overall_task: TaskID = progress.add_task(
            "[bold green]Overall", total=len(jobs)
        )
        type_tasks: dict[str, TaskID] = {}
        for t in sorted({j.pipeline_type for j in jobs}):
            count = sum(1 for j in jobs if j.pipeline_type == t)
            type_tasks[t] = progress.add_task(
                f"  [cyan]{t.capitalize()}", total=count
            )

        for start in range(0, len(jobs), concurrency):
            wave = jobs[start : start + concurrency]
            semaphore = asyncio.Semaphore(concurrency)
            async_tasks = [
                process_batch(llm, job, prompts[job.pipeline_type], semaphore)
                for job in wave
            ]

            wave_failed = False
            for coro in asyncio.as_completed(async_tasks):
                result: BatchResult = await coro
                results.append(result)

                progress.advance(overall_task)
                progress.advance(type_tasks[result.job.pipeline_type])

                if result.success:
                    batch_path = write_batch_result(result)
                    if not batch_path:
                        result.success = False
                        result.error = "write failed — see log above"
                        wave_failed = True
                        console.log(
                            f"[red]✗[/red] {result.job.label} — write failed"
                        )
                    elif attach_audio(batch_path):
                        console.log(
                            f"[green]✓[/green] {result.job.label}"
                            f"{_tokens_info(result)}"
                        )
                    else:
                        result.success = False
                        result.error = "audio generation failed — see log above"
                        wave_failed = True
                        console.log(
                            f"[red]✗[/red] {result.job.label} — audio generation failed"
                        )
                else:
                    wave_failed = True
                    console.log(
                        f"[red]✗[/red] {result.job.label} — {result.error}"
                    )

            if wave_failed and fail_fast:
                remaining = len(jobs) - (start + len(wave))
                if remaining > 0:
                    console.print(
                        f"[yellow]Stopping early after this wave because fail-fast is enabled. "
                        f"{remaining} pending batch(es) were not started.[/yellow]"
                    )
                break

    # ── 4. Summary ───────────────────────────────────────────────────────────
    console.print()
    print_summary(results)

    failed = [r for r in results if not r.success]
    return 1 if failed else 0


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument(
        "--model",
        default=DEFAULT_MODEL,
        metavar="PROVIDER:MODEL",
        help=(
            "LLM to use. Format: '<provider>:<model-id>'. "
            f"Default: {DEFAULT_MODEL} (set MODEL env var to change the default)."
        ),
    )
    parser.add_argument(
        "--type",
        choices=["words", "sentences"],
        default=None,
        help="Process only this pipeline type. Default: both.",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=DEFAULT_BATCH_SIZE,
        metavar="N",
        help=f"Items per batch. Default: {DEFAULT_BATCH_SIZE}.",
    )
    parser.add_argument(
        "--concurrency",
        type=int,
        default=DEFAULT_CONCURRENCY,
        metavar="N",
        help=f"Max parallel LLM requests. Default: {DEFAULT_CONCURRENCY}.",
    )
    parser.add_argument(
        "--max-items",
        type=int,
        default=None,
        metavar="N",
        help=(
            "Process at most N input items this run, counting full batches only. "
            "Useful for small test runs."
        ),
    )
    parser.add_argument(
        "--max-batches",
        type=int,
        default=None,
        metavar="N",
        help="Process at most N pending batches this run.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Reprocess all files even if already up to date.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show pending batches without making any LLM calls.",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Show debug log messages.",
    )
    parser.add_argument(
        "--no-fail-fast",
        action="store_true",
        help="Keep going after a failed batch instead of stopping after the current wave.",
    )
    return parser.parse_args()


def setup_logging(verbose: bool) -> None:
    logging.basicConfig(
        level=logging.DEBUG if verbose else logging.WARNING,
        format="%(message)s",
        handlers=[RichHandler(console=console, show_path=False, markup=True)],
    )


def main() -> None:
    args = parse_args()
    if args.batch_size < 1:
        raise SystemExit("--batch-size must be at least 1")
    if args.concurrency < 1:
        raise SystemExit("--concurrency must be at least 1")
    if args.max_items is not None and args.max_items < 1:
        raise SystemExit("--max-items must be at least 1")
    if args.max_batches is not None and args.max_batches < 1:
        raise SystemExit("--max-batches must be at least 1")
    setup_logging(args.verbose)

    exit_code = asyncio.run(
        run_pipeline(
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
    sys.exit(exit_code)


if __name__ == "__main__":
    main()
