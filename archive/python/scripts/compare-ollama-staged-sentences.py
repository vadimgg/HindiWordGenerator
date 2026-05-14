#!/usr/bin/env python3
"""Compare staged Ollama sentence enrichment against backed-up sentence output."""
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
from datetime import datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))

from llm_client import create_llm
from ollama_staged import enrich_source, source_from_sentence


def load_sentence_batches(source_dir: Path, max_batches: int, batch_size: int) -> list[dict]:
    selected = []
    for file_index, path in enumerate(sorted(source_dir.glob("*.json"))):
        if file_index >= max_batches:
            break
        data = json.loads(path.read_text(encoding="utf-8"))
        for sentence_index, sentence in enumerate(data.get("sentences", [])[:batch_size]):
            selected.append({
                "path": str(path),
                "title": data.get("title") or "Ollama Compare",
                "subtitle": data.get("subtitle") or path.stem,
                "index": sentence_index,
                "sentence": sentence,
            })
    return selected


def compare_sentences(original: dict, generated: dict) -> dict:
    return {
        "same_hindi": original.get("hindi") == generated.get("hindi"),
        "same_romanisation": original.get("romanisation") == generated.get("romanisation"),
        "same_english": original.get("english") == generated.get("english"),
        "same_register": original.get("register") == generated.get("register"),
        "original_word_count": len(original.get("words", [])),
        "generated_word_count": len(generated.get("words", [])),
        "same_word_count": len(original.get("words", [])) == len(generated.get("words", [])),
    }


async def enrich_one(llm, item: dict, semaphore: asyncio.Semaphore) -> dict:
    async with semaphore:
        label = f"{Path(item['path']).name}#{item['index'] + 1}"
        try:
            result = await enrich_source(
                llm,
                source_from_sentence(item["sentence"]),
                title=item["title"],
                subtitle=item["subtitle"],
                tags=["ollama-compare", "staged", "chapter-02"],
                progress_label=label,
            )
            generated = result["data"]["sentences"][0]
            return {
                "source_path": item["path"],
                "source_index": item["index"],
                "valid": True,
                "comparison": compare_sentences(item["sentence"], generated),
                "original": item["sentence"],
                "generated": generated,
                "steps": result["steps"],
            }
        except Exception as exc:
            return {
                "source_path": item["path"],
                "source_index": item["index"],
                "valid": False,
                "error": f"{type(exc).__name__}: {exc}",
                "original": item["sentence"],
            }


async def run_compare(args: argparse.Namespace) -> dict:
    source_dir = Path(args.source_dir)
    if not source_dir.is_absolute():
        source_dir = PROJECT_ROOT / source_dir
    items = load_sentence_batches(source_dir, args.max_batches, args.batch_size)
    if not items:
        raise ValueError(f"No sentence batches found in {source_dir}")

    llm = create_llm(args.model)
    semaphore = asyncio.Semaphore(args.concurrency)
    results = await asyncio.gather(*(enrich_one(llm, item, semaphore) for item in items))
    return {
        "model": args.model,
        "source_dir": str(source_dir),
        "batch_size": args.batch_size,
        "max_batches": args.max_batches,
        "concurrency": args.concurrency,
        "count": len(results),
        "valid_count": sum(1 for result in results if result["valid"]),
        "results": results,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:translategemma:12b")
    parser.add_argument("--type", choices=["sentences"], default="sentences")
    parser.add_argument("--source-dir", default="output_original/sentences")
    parser.add_argument("--batch-size", type=int, default=1, help="Sentences to sample from each source batch file.")
    parser.add_argument("--max-batches", type=int, default=1, help="Source batch files to sample.")
    parser.add_argument("--concurrency", type=int, default=1)
    parser.add_argument("--out", default=None)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    report = asyncio.run(run_compare(args))
    if args.out:
        out_path = Path(args.out)
    else:
        stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        out_path = PROJECT_ROOT / "comparisons" / f"ollama_staged_sentences_{stamp}.json"
    if not out_path.is_absolute():
        out_path = PROJECT_ROOT / out_path
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote comparison: {out_path}")
    print(json.dumps({
        "count": report["count"],
        "valid_count": report["valid_count"],
        "path": str(out_path),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
