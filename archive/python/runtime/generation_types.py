"""Shared generation pipeline data structures."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass
class BatchJob:
    """One unit of work: a single batch slice from one YAML input."""

    pipeline_type: str
    stem: str
    batch_num: int
    total_batches: int
    display_label: Optional[str]
    source: str
    count: int

    @property
    def label(self) -> str:
        return f"{self.stem} [{self.pipeline_type}] {self.batch_num}/{self.total_batches}"


@dataclass
class BatchResult:
    """Result of processing one generated batch."""

    job: BatchJob
    success: bool
    data: Optional[dict] = None
    error: Optional[str] = None
    input_tokens: int = 0
    output_tokens: int = 0
    cache_read_tokens: int = 0
    cache_write_tokens: int = 0
