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

PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))

from llm_client import call_llm, create_llm, load_prompt
from schema_validator import validate_and_fix


DEFAULT_CSV = "# Complete Hindi\n## Ollama Smoke\nक्या बात है? (kyā bāt hai?);What is the matter?\n"


async def run_check(model: str, csv: str) -> dict:
    data, usage = await call_llm(create_llm(model), load_prompt("sentences"), csv)
    return {"usage": usage, "data": validate_and_fix("sentences", data)}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:translategemma:12b")
    parser.add_argument("--csv", default=DEFAULT_CSV)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    result = asyncio.run(run_check(args.model, args.csv))
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
