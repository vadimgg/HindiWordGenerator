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

from batch_planner import parse_input_item
from llm_client import call_llm, create_llm
from schema_validator import validate_and_fix

DEFAULT_CSV_LINE = "क्या बात है? (kyā bāt hai?);What's the matter?"

ENGLISH_PROMPT = """You translate one Hindi sentence for a flashcard.
Return JSON only with exactly these keys:
english, literal, register

Rules:
- Use the supplied English as a hint, but improve it if needed.
- literal follows Hindi word order in plain English.
- register is one of: casual, neutral, formal.
"""

ROMAN_PROMPT = """You romanise one Hindi sentence for a learner.
Return JSON only with exactly this key:
romanisation

Rules:
- Use Latin letters with Hindi diacritics.
- Use tilde nasalisation, e.g. maĩ, yahā̃, haĩ.
- Do not include English.
- Do not include the Hindi sentence.
"""

WORDS_PROMPT = """You make a word-by-word Hindi sentence breakdown.
Return JSON only with exactly this key:
words

Each words item has:
hindi, roman, meaning

Optional keys only when useful:
gender, number, note

Rules:
- Include actual words only.
- Do not include spaces or punctuation.
- Keep the words in sentence order.
- Use the supplied romanisation to romanise each word.
"""


def build_tokens(words: list[dict]) -> list[dict]:
    return [
        {
            "hindi": word["hindi"],
            "roman": word["roman"],
            "kind": "word",
            "word_index": index,
        }
        for index, word in enumerate(words)
    ]


async def ask(llm, prompt: str, payload: dict) -> dict:
    data, _ = await call_llm(
        llm,
        prompt,
        json.dumps(payload, ensure_ascii=False, indent=2),
    )
    return data


async def run_check(model: str, csv_line: str) -> dict:
    hindi, source_romanisation, source_english = parse_input_item(csv_line)
    if not hindi:
        raise ValueError("Could not parse Hindi from input line.")

    llm = create_llm(model)
    source = {
        "hindi": hindi,
        "source_romanisation": source_romanisation,
        "source_english": source_english,
    }

    print("→ English/literal/register", file=sys.stderr)
    english = await ask(llm, ENGLISH_PROMPT, source)
    print("→ Romanisation", file=sys.stderr)
    roman = await ask(llm, ROMAN_PROMPT, source)
    print("→ Word breakdown", file=sys.stderr)
    words = await ask(
        llm,
        WORDS_PROMPT,
        {
            "hindi": hindi,
            "romanisation": roman["romanisation"],
            "english": english["english"],
        },
    )

    sentence = {
        "hindi": hindi,
        "romanisation": roman["romanisation"],
        "english": english["english"],
        "literal": english["literal"],
        "register": english["register"],
        "tokens": build_tokens(words["words"]),
        "words": words["words"],
        "anki_tags": ["ollama-smoke", english["register"], "staged"],
    }
    batch = {
        "title": "Ollama Smoke",
        "subtitle": "Staged Sentence",
        "sentences": [sentence],
    }
    return {
        "source": source,
        "steps": {
            "english": english,
            "romanisation": roman,
            "words": words,
        },
        "data": validate_and_fix("sentences", batch),
    }


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
