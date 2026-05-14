"""Check planning and Rich report rendering for `main.py check`."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Optional

from rich.console import Console
from rich.panel import Panel
from rich.table import Table

import generate
import process

console = Console()


@dataclass
class StemPlan:
    pipeline_type: str
    stem: str
    display_label: str
    skipped_items: list[str] = field(default_factory=list)
    planned_jobs: list[generate.BatchJob] = field(default_factory=list)
    deferred_jobs: list[generate.BatchJob] = field(default_factory=list)
    existing_batches: int = 0
    missing_sound_alikes: list[str] = field(default_factory=list)
    missing_audio: list[str] = field(default_factory=list)
    missing_sentence_tokens: list[str] = field(default_factory=list)
    missing_metadata: bool = False


def item_label(item: dict) -> str:
    hindi, roman, english = process.parse_input_item(item)
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
        for source_path in sorted(input_dir.glob("*.yaml")):
            display_label, items = process.parse_source_file(source_path)
            if not items:
                continue

            identities, existing_batches = process.load_existing_output_state(current_type, source_path.stem)
            stem_plan = StemPlan(
                pipeline_type=current_type,
                stem=source_path.stem,
                display_label=display_label or process.label_from_stem(source_path.stem),
                existing_batches=existing_batches,
                missing_metadata=not process.has_top_metadata_heading(source_path),
            )

            pending: list[dict] = []
            for item in items:
                if not force and process.parse_input_item(item) in identities:
                    stem_plan.skipped_items.append(item_label(item))
                else:
                    pending.append(item)

            if current_type == "words":
                for entry in existing_entries(current_type, source_path.stem):
                    if not entry.get("sound_alikes"):
                        stem_plan.missing_sound_alikes.append(entry.get("hindi", ""))
                    if not entry.get("audio"):
                        stem_plan.missing_audio.append(entry.get("hindi", ""))
            else:
                for entry in existing_entries(current_type, source_path.stem):
                    if not entry.get("tokens"):
                        stem_plan.missing_sentence_tokens.append(entry.get("english") or entry.get("hindi", ""))
                    if not entry.get("audio"):
                        stem_plan.missing_audio.append(entry.get("english") or entry.get("hindi", ""))

            batches = process.make_batches(pending, batch_size)
            total_batches = existing_batches + len(batches)
            for offset, batch in enumerate(batches, 1):
                job = generate.BatchJob(
                    pipeline_type=current_type,
                    stem=source_path.stem,
                    batch_num=existing_batches + offset,
                    total_batches=total_batches,
                    display_label=stem_plan.display_label,
                    source=process.build_batch_yaml(stem_plan.display_label, batch),
                    count=len(batch),
                )
                all_jobs.append(job)

            stem_plans.append(stem_plan)
            plan_by_key[(current_type, source_path.stem)] = stem_plan

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
    summary.add_column("Source / Stem")
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
            or stem_plan.missing_metadata
        ):
            continue
        gaps: list[str] = []
        if stem_plan.missing_metadata:
            gaps.append("metadata")
        if stem_plan.missing_sound_alikes:
            gaps.append(f"sound_alikes:{len(stem_plan.missing_sound_alikes)}")
        if stem_plan.missing_sentence_tokens:
            gaps.append(f"tokens:{len(stem_plan.missing_sentence_tokens)}")
        if stem_plan.missing_audio:
            gaps.append(f"audio:{len(stem_plan.missing_audio)}")
        summary.add_row(
            stem_plan.pipeline_type,
            f"{stem_plan.display_label}\n[dim]{stem_plan.stem}[/dim]",
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
            stem_plan.missing_metadata,
        ]):
            continue

        console.print(f"\n[bold]{stem_plan.display_label}[/bold]  [dim]({stem_plan.pipeline_type}/{stem_plan.stem})[/dim]")

        if stem_plan.planned_jobs:
            planned = Table(show_header=True, header_style="green")
            planned.add_column("Planned batch")
            planned.add_column("Items")
            planned.add_column("Preview")
            for job in stem_plan.planned_jobs:
                _, source_items = process.parse_yaml_metadata_from_text(job.source, job.stem)
                preview = "\n".join(item_label(item) for item in source_items[: min(3, len(source_items))])
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

        if stem_plan.missing_metadata:
            console.print(
                "[magenta]Missing source metadata:[/magenta] "
                "add non-empty `title` and/or `subtitle` fields."
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
