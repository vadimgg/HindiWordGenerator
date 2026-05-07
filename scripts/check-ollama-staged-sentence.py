#!/usr/bin/env python3
"""Smoke-check a staged Ollama-only sentence enrichment workflow."""
# /// script
# requires-python = ">=3.12"
# dependencies = [
#   "langchain-core>=0.3",
#   "langchain-openai>=0.3",
# ]
# ///

from __future__ import annotations

import argparse
import asyncio
import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))

from llm_client import create_llm
from ollama_staged import DEFAULT_CSV_LINE, enrich_source, source_from_csv_line


async def run_check(model: str, csv_line: str) -> dict:
    llm = create_llm(model)
    return await enrich_source(llm, source_from_csv_line(csv_line))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:translategemma:12b")
    parser.add_argument("--line", default=DEFAULT_CSV_LINE)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    result = asyncio.run(run_check(args.model, args.line))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
