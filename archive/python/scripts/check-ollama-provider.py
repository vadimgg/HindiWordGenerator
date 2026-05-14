#!/usr/bin/env python3
"""Smoke-check an Ollama model against the sentence generation contract."""
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

from llm_client import call_llm, create_llm, load_prompt
from schema_validator import validate_and_fix


DEFAULT_SOURCE = """title: "Complete Hindi"
subtitle: "Ollama Smoke"
items:
  - hindi: "क्या बात है?"
    romanisation: "kyā bāt hai?"
    english: "What is the matter?"
"""


async def run_check(model: str, source: str) -> dict:
    data, usage = await call_llm(create_llm(model), load_prompt("sentences"), source)
    return {"usage": usage, "data": validate_and_fix("sentences", data)}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:translategemma:12b")
    parser.add_argument("--source", default=DEFAULT_SOURCE)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    result = asyncio.run(run_check(args.model, args.source))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
