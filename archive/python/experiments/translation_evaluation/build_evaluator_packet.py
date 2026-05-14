#!/usr/bin/env python3
"""Build Hindi translation evaluator packets from experiment result JSON."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]

EVALUATOR_PROMPT = """You are the Hindi language teacher reviewer for this flashcard project.

Evaluate Hindi benchmark results for practical learner quality, not just exact string matching.
Assume the learner speaks English, Russian, and Hebrew, lives in India, and wants practical Hindi.

For each result, rate:
- english_accuracy: 1-5, whether English preserves the Hindi meaning.
- natural_english: 1-5, whether English sounds natural for a learner-facing card.
- romanisation_accuracy: 1-5, whether romanisation is readable and faithful.
- word_breakdown_accuracy: 1-5 or null when no word breakdown exists.
- register_accuracy: 1-5 or null when the test is not register detection.
- learner_usefulness: 1-5.

Also flag:
- meaning drift
- dropped honorifics like जी when they matter
- bad romanisation
- Devanagari placed in romanisation fields
- word-by-word entries that are transliteration instead of meaning
- register mistakes
- source-row QA mistakes, especially failing to catch bad English, romanisation mismatches, or non-sentence drills

When evaluating one result, return raw JSON only:
{
  "verdict":"good|usable|weak|bad",
  "test_name":"...",
  "result_file":"...",
  "english_accuracy":1,
  "natural_english":1,
  "romanisation_accuracy":1,
  "word_breakdown_accuracy":null,
  "register_accuracy":null,
  "learner_usefulness":1,
  "issues":["..."],
  "bullet_points":["..."],
  "comment":"..."
}

When evaluating a model-run batch, return raw JSON only:
{
  "verdict":"good|usable|weak|bad",
  "summary":"...",
  "items":[
    {
      "verdict":"good|usable|weak|bad",
      "test_name":"...",
      "result_file":"...",
      "english_accuracy":1,
      "natural_english":1,
      "romanisation_accuracy":1,
      "word_breakdown_accuracy":null,
      "register_accuracy":null,
      "learner_usefulness":1,
      "issues":["..."],
      "bullet_points":["..."],
      "comment":"..."
    }
  ]
}
"""


def slim_result(result: dict) -> dict:
    return {
        "test_name": result["test_name"],
        "result_file": result.get("result_file"),
        "source_index": result["source"]["index"],
        "source_path": result["source"]["path"],
        "input_sentence": result["input_sentence"],
        "reference": result["reference"],
        "result": result.get("result"),
        "valid_json": result.get("valid", False),
        "timed_out": result.get("timed_out", False),
        "error": result.get("error"),
        "comparison": result.get("comparison"),
    }


def packet_from_result(result: dict, source_path: Path) -> dict:
    return {
        "model": result["model"],
        "experiment_id": result["experiment_id"],
        "result_file": str(source_path.relative_to(PROJECT_ROOT)),
        "evaluator_prompt": EVALUATOR_PROMPT,
        "result": slim_result(result),
    }


def packet_from_summary(summary: dict, source_path: Path) -> dict:
    return {
        "model": summary["model"],
        "experiment_id": summary["experiment_id"],
        "summary_path": str(source_path.relative_to(PROJECT_ROOT)),
        "evaluator_prompt": EVALUATOR_PROMPT,
        "results": [slim_result(result) for result in summary.get("results", [])],
    }


def is_result_file(path: Path, data: dict) -> bool:
    if path.name.endswith(("_summary.json", "_evaluator_packet.json", "_evaluation.json")):
        return False
    return all(key in data for key in ("experiment_id", "test_name", "input_sentence", "result_file"))


def needs_evaluation(data: dict) -> bool:
    return not bool(data.get("evaluation", {}).get("verdict"))


def model_run_results(run_dir: Path, include_evaluated: bool) -> list[tuple[Path, dict]]:
    rows = []
    for path in sorted(run_dir.glob("*.json")):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        if not is_result_file(path, data):
            continue
        if include_evaluated or needs_evaluation(data):
            rows.append((path, data))
    return rows


def packet_from_model_run(run_dir: Path, include_evaluated: bool) -> dict:
    rows = model_run_results(run_dir, include_evaluated=include_evaluated)
    if not rows:
        return {
            "model": None,
            "experiment_id": run_dir.name,
            "run_dir": str(run_dir.relative_to(PROJECT_ROOT)),
            "evaluator_prompt": EVALUATOR_PROMPT,
            "results": [],
        }

    first = rows[0][1]
    return {
        "model": first["model"],
        "experiment_id": first["experiment_id"],
        "run_dir": str(run_dir.relative_to(PROJECT_ROOT)),
        "evaluation_scope": "model_run",
        "includes_evaluated_results": include_evaluated,
        "evaluator_prompt": EVALUATOR_PROMPT,
        "results": [slim_result(result) for _path, result in rows],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result_summary_or_run_dir")
    parser.add_argument("--out", default=None)
    parser.add_argument(
        "--include-evaluated",
        action="store_true",
        help="For run directories, include results that already have evaluation.verdict.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    source_path = Path(args.result_summary_or_run_dir)
    if not source_path.is_absolute():
        source_path = PROJECT_ROOT / source_path

    if source_path.is_dir():
        packet = packet_from_model_run(source_path, include_evaluated=args.include_evaluated)
    else:
        source = json.loads(source_path.read_text(encoding="utf-8"))
        if "results" in source:
            packet = packet_from_summary(source, source_path)
        else:
            packet = packet_from_result(source, source_path)

    if source_path.is_dir():
        default_out = source_path / f"{source_path.name}_model_run_evaluator_packet.json"
    else:
        default_out = source_path.with_name(f"{source_path.stem}_evaluator_packet.json")

    if args.out:
        out_path = Path(args.out)
        if not out_path.is_absolute():
            out_path = PROJECT_ROOT / out_path
    else:
        out_path = default_out

    out_path.write_text(json.dumps(packet, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(out_path)


if __name__ == "__main__":
    main()
