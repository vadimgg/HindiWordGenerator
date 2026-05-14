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

PYTHON_ARCHIVE_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PYTHON_ARCHIVE_ROOT / "runtime"))

from llm_client import create_llm
from ollama_staged import DEFAULT_SOURCE, enrich_source, source_from_yaml_item


async def run_check(model: str, source_json: str) -> dict:
    llm = create_llm(model)
    return await enrich_source(llm, source_from_yaml_item(json.loads(source_json)))


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:translategemma:12b")
    parser.add_argument("--source-json", default=json.dumps(DEFAULT_SOURCE, ensure_ascii=False))
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    result = asyncio.run(run_check(args.model, args.source_json))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
