#!/usr/bin/env python3
"""Static checks for local agent workflow documentation."""

from __future__ import annotations

from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (PROJECT_ROOT / path).read_text(encoding="utf-8")


def assert_contains(path: str, required: list[str]) -> None:
    text = read(path)
    missing = [item for item in required if item not in text]
    if missing:
        joined = ", ".join(missing)
        raise AssertionError(f"{path} is missing required workflow text: {joined}")


def main() -> None:
    shared_requirements = [
        "process.py check",
        "process.py write",
        "without external provider API keys",
        "Do not manually choose or split source lines",
        "Do not place manually generated JSON",
    ]
    assert_contains("agents/packs/sentence-batch-generator/AGENT.md", shared_requirements)
    assert_contains("agents/packs/word-batch-generator/AGENT.md", shared_requirements)
    assert_contains("NO_API_AGENT_WORKFLOW.md", [
        "python3 process.py check",
        "python3 process.py write",
        "Do not manually split CSV input",
        "Do not write directly to `output/`",
        "Do not use `main.py run` unless API-backed generation was requested",
    ])
    print("Agent workflow documentation checks passed.")


if __name__ == "__main__":
    main()
