"""Shared filesystem configuration for the generation pipeline."""

from __future__ import annotations

from pathlib import Path

PROJECT_ROOT = Path(__file__).parent

PIPELINES = {
    "words": {
        "prompt": PROJECT_ROOT / "generation_prompt_words.txt",
        "input": PROJECT_ROOT / "input" / "words",
        "output": PROJECT_ROOT / "output" / "words",
    },
    "sentences": {
        "prompt": PROJECT_ROOT / "generation_prompt_sentences.txt",
        "input": PROJECT_ROOT / "input" / "sentences",
        "output": PROJECT_ROOT / "output" / "sentences",
    },
}

DEFAULT_BATCH_SIZE = 10
