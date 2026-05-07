"""Subprocess boundaries for generation planning, output writes, and audio attach."""

from __future__ import annotations

import json
import logging
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

from generation_types import BatchJob, BatchResult

PROJECT_ROOT = Path(__file__).parent
log = logging.getLogger("generate")


def get_pending_batches(
    pipeline_type: Optional[str],
    batch_size: int,
    force: bool,
    default_batch_size: int,
) -> list[BatchJob]:
    """
    Call `process.py check` and deserialise the result into BatchJob list.
    """
    cmd = [sys.executable, str(PROJECT_ROOT / "process.py"), "check"]

    if pipeline_type:
        cmd += ["--type", pipeline_type]
    if force:
        cmd.append("--force")
    if batch_size != default_batch_size:
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
            display_label=item.get("display_label"),
            csv=item["csv"],
            count=item["count"],
        )
        for item in raw
    ]


def write_batch_result(result: BatchResult) -> Path | None:
    """
    Persist a successful batch result by calling `process.py write`.
    """
    job = result.job

    with tempfile.NamedTemporaryFile(
        mode="w",
        suffix=".json",
        prefix="hindi_batch_",
        encoding="utf-8",
        delete=False,
    ) as file:
        json.dump(result.data, file, ensure_ascii=False, indent=2)
        tmp_path = Path(file.name)

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
                job.label,
                proc.returncode,
                proc.stderr.strip(),
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
