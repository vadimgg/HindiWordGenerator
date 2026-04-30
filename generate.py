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
import json
import logging
import os
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from dotenv import load_dotenv

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
MAX_RETRIES = 3
RETRY_BASE_DELAY = 2.0  # seconds

console = Console()
log = logging.getLogger("generate")


# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------

@dataclass
class BatchJob:
    """One unit of work: a single batch slice from one input CSV."""
    pipeline_type: str      # "words" | "sentences"
    stem: str               # input filename without extension
    batch_num: int
    total_batches: int
    chapter: Optional[str]
    csv: str                # CSV content to send to the LLM
    count: int              # number of items in this batch

    @property
    def label(self) -> str:
        return f"{self.stem} [{self.pipeline_type}] {self.batch_num}/{self.total_batches}"


@dataclass
class BatchResult:
    job: BatchJob
    success: bool
    data: Optional[dict] = None
    error: Optional[str] = None
    input_tokens: int = 0
    output_tokens: int = 0
    cache_read_tokens: int = 0
    cache_write_tokens: int = 0


# ---------------------------------------------------------------------------
# Model factory
# ---------------------------------------------------------------------------

def create_llm(model_string: str):
    """
    Build a LangChain chat model from a '<provider>:<model-id>' string.

    Supported providers: anthropic, openai.
    Raises ValueError for bad format or unknown provider.
    Raises EnvironmentError if the required API key is missing.
    """
    if ":" not in model_string:
        raise ValueError(
            f"Invalid model format '{model_string}'. "
            "Expected '<provider>:<model-id>', "
            "e.g. 'openai:gpt-5.4-mini' or 'anthropic:claude-haiku-4-5-20251001'."
        )

    provider, model_id = model_string.split(":", 1)

    if provider == "anthropic":
        _require_env("ANTHROPIC_API_KEY", provider)
        from langchain_anthropic import ChatAnthropic
        return ChatAnthropic(model=model_id, max_tokens=8192)

    elif provider == "openai":
        _require_env("OPENAI_API_KEY", provider)
        from langchain_openai import ChatOpenAI
        return ChatOpenAI(model=model_id, max_tokens=8192)

    else:
        raise ValueError(
            f"Unknown provider '{provider}'. Supported providers: anthropic, openai."
        )


def _require_env(key: str, provider: str) -> None:
    if not os.environ.get(key):
        raise EnvironmentError(
            f"{key} is required for {provider} models. "
            f"Set it with: export {key}=<your-api-key>"
        )


# ---------------------------------------------------------------------------
# Prompt helpers
# ---------------------------------------------------------------------------

def load_prompt(pipeline_type: str) -> str:
    """Load the generation prompt file for the given pipeline type."""
    path = PROJECT_ROOT / f"generation_prompt_{pipeline_type}.txt"
    if not path.exists():
        raise FileNotFoundError(f"Prompt file not found: {path}")
    return path.read_text(encoding="utf-8")


def build_messages(llm, system_prompt: str, csv_content: str) -> list:
    """
    Build the message list for the LLM call.

    For Anthropic models, attaches cache_control to the system prompt so the
    large prompt is cached across all batches of the same type — significantly
    reducing input token costs on repeated calls.
    """
    from langchain_core.messages import HumanMessage, SystemMessage

    is_anthropic = "anthropic" in type(llm).__module__

    if is_anthropic:
        system_content = [
            {
                "type": "text",
                "text": system_prompt,
                "cache_control": {"type": "ephemeral"},
            }
        ]
        return [
            SystemMessage(content=system_content),
            HumanMessage(content=csv_content),
        ]

    return [
        SystemMessage(content=system_prompt),
        HumanMessage(content=csv_content),
    ]


# ---------------------------------------------------------------------------
# LLM interaction
# ---------------------------------------------------------------------------

def extract_json(text: str) -> dict:
    """
    Parse JSON from an LLM response.

    Strips markdown code fences (```json ... ``` or ``` ... ```) if the model
    includes them despite being told not to.
    Raises json.JSONDecodeError on parse failure.
    """
    text = text.strip()

    if text.startswith("```"):
        lines = text.splitlines()
        inner = lines[1:]
        if inner and inner[-1].strip() == "```":
            inner = inner[:-1]
        text = "\n".join(inner).strip()

    return json.loads(text)


def extract_usage(response) -> dict:
    """Extract token usage counts from a LangChain AI message."""
    result = {
        "input_tokens": 0,
        "output_tokens": 0,
        "cache_read_tokens": 0,
        "cache_write_tokens": 0,
    }

    meta = getattr(response, "usage_metadata", None)
    if not meta:
        return result

    if isinstance(meta, dict):
        result["input_tokens"] = meta.get("input_tokens", 0)
        result["output_tokens"] = meta.get("output_tokens", 0)
        details = meta.get("input_token_details", {})
    else:
        result["input_tokens"] = getattr(meta, "input_tokens", 0)
        result["output_tokens"] = getattr(meta, "output_tokens", 0)
        details = getattr(meta, "input_token_details", {}) or {}

    if isinstance(details, dict):
        result["cache_read_tokens"] = details.get("cache_read", 0)
        result["cache_write_tokens"] = details.get("cache_creation", 0)

    return result


async def call_llm(llm, system_prompt: str, csv_content: str) -> tuple[dict, dict]:
    """
    Make a single LLM call and return (parsed_json, usage_dict).

    Raises json.JSONDecodeError if the response is not valid JSON.
    Raises any LangChain/API exception on network or provider errors.
    """
    messages = build_messages(llm, system_prompt, csv_content)
    response = await llm.ainvoke(messages)
    data = extract_json(response.content)
    usage = extract_usage(response)
    return data, usage


async def call_llm_with_retry(
    llm,
    system_prompt: str,
    csv_content: str,
    label: str,
) -> tuple[dict, dict]:
    """
    Retry the LLM call up to MAX_RETRIES times with exponential backoff.

    Retries on JSON parse errors and transient API errors.
    Raises on the final attempt if all retries are exhausted.
    """
    last_error: Exception | None = None

    for attempt in range(1, MAX_RETRIES + 1):
        try:
            return await call_llm(llm, system_prompt, csv_content)

        except json.JSONDecodeError as exc:
            last_error = exc
            if attempt < MAX_RETRIES:
                delay = RETRY_BASE_DELAY ** (attempt - 1)
                log.warning(
                    "[%s] JSON parse error (attempt %d/%d), retrying in %.0fs: %s",
                    label, attempt, MAX_RETRIES, delay, exc,
                )
                await asyncio.sleep(delay)

        except Exception as exc:
            last_error = exc
            if attempt < MAX_RETRIES:
                delay = RETRY_BASE_DELAY ** (attempt - 1)
                log.warning(
                    "[%s] API error (attempt %d/%d), retrying in %.0fs: %s: %s",
                    label, attempt, MAX_RETRIES, delay, type(exc).__name__, exc,
                )
                await asyncio.sleep(delay)
            else:
                raise

    raise ValueError(
        f"[{label}] Failed after {MAX_RETRIES} attempts. Last error: {last_error}"
    )


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


def write_batch_result(result: BatchResult) -> Path | None:
    """
    Persist a successful batch result by calling `process.py write`.

    Writes the JSON to a temp file, delegates validation and file placement
    to process.py, then cleans up the temp file.
    Returns the written batch path on success, or None on failure.
    """
    job = result.job

    with tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".json",
        prefix="hindi_batch_",
        encoding="utf-8",
        delete=False,
    ) as f:
        json.dump(result.data, f, ensure_ascii=False, indent=2)
        tmp_path = Path(f.name)

    try:
        cmd = [
            sys.executable,
            str(PROJECT_ROOT / "process.py"),
            "write",
            job.pipeline_type,
            job.stem,
            str(job.batch_num),
            str(job.total_batches),
            str(job.count),
            str(tmp_path),
        ]
        proc = subprocess.run(cmd, capture_output=True, text=True, check=False)

        if proc.returncode != 0:
            log.error(
                "[%s] process.py write failed (exit %d):\n%s",
                job.label, proc.returncode, proc.stderr.strip(),
            )
            return None

        log.debug("[%s] write output: %s", job.label, proc.stdout.strip())
        return PROJECT_ROOT / "output" / job.pipeline_type / f"{job.stem}_batch_{job.batch_num:02d}.json"

    finally:
        tmp_path.unlink(missing_ok=True)


def attach_audio(batch_path: Path) -> bool:
    """Generate per-entry audio for a written batch and update JSON audio paths."""
    cmd = [sys.executable, str(PROJECT_ROOT / "audio_generator.py"), str(batch_path)]
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        log.error(
            "[%s] audio generation failed (exit %d):\n%s",
            batch_path.name,
            proc.returncode,
            proc.stderr.strip() or proc.stdout.strip(),
        )
        return False
    return True


# ---------------------------------------------------------------------------
# Manifest / check
# ---------------------------------------------------------------------------

def get_pending_batches(
    pipeline_type: Optional[str],
    batch_size: int,
    force: bool,
) -> list[BatchJob]:
    """
    Call `process.py check` and deserialise the result into BatchJob list.

    Raises RuntimeError if process.py exits non-zero.
    """
    cmd = [sys.executable, str(PROJECT_ROOT / "process.py"), "check"]

    if pipeline_type:
        cmd += ["--type", pipeline_type]
    if force:
        cmd.append("--force")
    if batch_size != DEFAULT_BATCH_SIZE:
        cmd += ["--batch-size", str(batch_size)]

    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)

    if proc.returncode != 0:
        raise RuntimeError(
            f"process.py check failed (exit {proc.returncode}):\n"
            f"{proc.stderr.strip()}"
        )

    raw: list[dict] = json.loads(proc.stdout)
    return [
        BatchJob(
            pipeline_type=item["type"],
            stem=item["stem"],
            batch_num=item["batch_num"],
            total_batches=item["total_batches"],
            chapter=item.get("chapter"),
            csv=item["csv"],
            count=item["count"],
        )
        for item in raw
    ]


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
    table.add_column("Chapter")

    for job in jobs:
        table.add_row(
            job.pipeline_type,
            job.stem,
            f"{job.batch_num}/{job.total_batches}",
            str(job.count),
            job.chapter or "—",
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
        jobs = get_pending_batches(pipeline_type, batch_size, force)
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
