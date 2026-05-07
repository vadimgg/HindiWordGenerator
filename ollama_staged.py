"""Ollama-only staged sentence enrichment helpers."""

from __future__ import annotations

import json
import sys

from batch_planner import parse_input_item
from llm_client import call_llm
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
- If source_romanisation is provided, preserve it unless it is clearly wrong.
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


def source_from_csv_line(csv_line: str) -> dict:
    hindi, source_romanisation, source_english = parse_input_item(csv_line)
    if not hindi:
        raise ValueError("Could not parse Hindi from input line.")
    return {
        "hindi": hindi,
        "source_romanisation": source_romanisation,
        "source_english": source_english,
    }


def source_from_sentence(sentence: dict) -> dict:
    hindi = str(sentence.get("hindi", "")).strip()
    if not hindi:
        raise ValueError("Sentence is missing hindi.")
    return {
        "hindi": hindi,
        "source_romanisation": str(sentence.get("romanisation", "")).strip(),
        "source_english": str(sentence.get("english", "")).strip(),
    }


async def ask(llm, prompt: str, payload: dict) -> dict:
    data, _ = await call_llm(
        llm,
        prompt,
        json.dumps(payload, ensure_ascii=False, indent=2),
    )
    return data


async def enrich_source(
    llm,
    source: dict,
    title: str = "Ollama Smoke",
    subtitle: str = "Staged Sentence",
    tags: list[str] | None = None,
    progress_label: str | None = None,
) -> dict:
    prefix = f"{progress_label}: " if progress_label else ""
    print(f"{prefix}→ English/literal/register", file=sys.stderr, flush=True)
    english = await ask(llm, ENGLISH_PROMPT, source)
    print(f"{prefix}→ Romanisation", file=sys.stderr, flush=True)
    roman = await ask(llm, ROMAN_PROMPT, source)
    print(f"{prefix}→ Word breakdown", file=sys.stderr, flush=True)
    words = await ask(
        llm,
        WORDS_PROMPT,
        {
            "hindi": source["hindi"],
            "source_romanisation": source.get("source_romanisation", ""),
            "romanisation": roman["romanisation"],
            "english": english["english"],
        },
    )

    sentence = {
        "hindi": source["hindi"],
        "romanisation": roman["romanisation"],
        "english": english["english"],
        "literal": english["literal"],
        "register": english["register"],
        "tokens": build_tokens(words["words"]),
        "words": words["words"],
        "anki_tags": tags or ["ollama-smoke", english["register"], "staged"],
    }
    batch = {
        "title": title,
        "subtitle": subtitle,
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
