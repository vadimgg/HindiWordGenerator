#!/usr/bin/env python3
"""Normalize a no-context agent translation benchmark JSON into summary files."""

from __future__ import annotations

import argparse
import json
import re
from datetime import UTC, datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
RESULTS_ROOT = PROJECT_ROOT / "experiments" / "agent_translation" / "results"


def normalize_english(text: str) -> str:
    text = text.lower().replace("’", "'")
    text = re.sub(r"\bhow're\b", "how are", text)
    text = re.sub(r"[^a-z0-9]+", " ", text)
    return " ".join(text.split())


def compare_translation(original: str, generated: str) -> dict:
    original_normalized = normalize_english(original)
    generated_normalized = normalize_english(generated)
    return {
        "exact_match": original == generated,
        "normalized_match": original_normalized == generated_normalized,
        "original_normalized": original_normalized,
        "generated_normalized": generated_normalized,
    }


def normalize_register(register: str) -> str:
    value = register.strip().lower().replace("_", "-")
    aliases = {
        "casual": "informal",
        "colloquial": "informal",
        "polite": "formal",
        "respectful": "formal",
        "standard": "neutral",
    }
    return aliases.get(value, value)


def compare_register(original: str, generated: str) -> dict:
    original_normalized = normalize_register(original)
    generated_normalized = normalize_register(generated)
    return {
        "exact_match": original == generated,
        "normalized_match": original_normalized == generated_normalized,
        "original_normalized": original_normalized,
        "generated_normalized": generated_normalized,
    }


def compare_result(record: dict) -> dict | None:
    result = record.get("result", {})
    if "register" in result:
        return compare_register(
            record.get("reference", {}).get("register", ""),
            str(result.get("register", "")),
        )

    generated_english = str(result.get("english", "")).strip()
    reference_english = record.get("reference", {}).get("english", "")
    if generated_english:
        return compare_translation(reference_english, generated_english)
    return None


def is_valid_result(record: dict) -> bool:
    result = record.get("result", {})
    if "register" in result:
        return bool(str(result.get("register", "")).strip())
    return bool(str(result.get("english", "")).strip())


def safe_slug(text: str, max_chars: int = 48) -> str:
    words = re.findall(r"[\w\u0900-\u097F]+", text, flags=re.UNICODE)
    slug = "_".join(words)[:max_chars].strip("_")
    return slug or "sentence"


def result_filename(experiment_id: str, test_name: str, source_index: int, sentence: str) -> str:
    return (
        f"{experiment_id}_{safe_slug(test_name)}_sentence_{source_index + 1:03d}_"
        f"{safe_slug(sentence, max_chars=36)}.json"
    )


def model_slug(model: str) -> str:
    return safe_slug(model.replace(":", "_"))


def normalize_result(record: dict, model: str, experiment_id: str) -> dict:
    return {
        "experiment_id": experiment_id,
        "test_name": record["test_name"],
        "adapter": record["test_name"],
        "model": model,
        "valid": is_valid_result(record),
        "input_sentence": record["input_sentence"],
        "input_line": record["input_line"],
        "source": {
            "path": "agent:no-context",
            "index": record["source_index"],
            "title": "Agent Benchmark",
            "subtitle": "No Context",
        },
        "reference": record["reference"],
        "full_prompt": record["full_prompt"],
        "input_payload": record["input_payload"],
        "timing": {
            "started_at": None,
            "finished_at": None,
            "duration_seconds": 0,
        },
        "usage": {},
        "result": record["result"],
        "comparison": compare_result(record),
    }


def build_summary(records: list[dict], model: str, experiment_id: str) -> dict:
    by_test = {}
    for record in records:
        summary = by_test.setdefault(
            record["test_name"],
            {
                "count": 0,
                "valid_count": 0,
                "normalized_match_count": 0,
                "model_call_seconds": 0,
                "avg_model_call_seconds": 0,
            },
        )
        summary["count"] += 1
        if record["valid"]:
            summary["valid_count"] += 1
            if record.get("comparison", {}).get("normalized_match"):
                summary["normalized_match_count"] += 1

    return {
        "experiment_id": experiment_id,
        "model": model,
        "source_dir": "agent:no-context",
        "adapters": sorted(by_test),
        "batch_size": 5,
        "max_batches": 1,
        "concurrency": 1,
        "test_timeout_seconds": None,
        "stop_on_timeout": False,
        "stopped_early": False,
        "stop_reason": None,
        "summary": {
            "count": len(records),
            "valid_count": sum(1 for record in records if record["valid"]),
            "normalized_match_count": sum(
                1 for record in records if record.get("comparison", {}).get("normalized_match")
            ),
            "wall_seconds": 0,
            "model_call_seconds": 0,
            "timed_out_count": 0,
        },
        "timing": {
            "started_at": None,
            "finished_at": datetime.now(UTC).isoformat(),
            "wall_seconds": 0,
            "concurrency": 1,
        },
        "by_test": by_test,
        "results": records,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input_json")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    input_path = Path(args.input_json)
    if not input_path.is_absolute():
        input_path = PROJECT_ROOT / input_path

    data = json.loads(input_path.read_text(encoding="utf-8"))
    experiment_id = data["experiment_id"]
    model = data["model"]
    out_dir = RESULTS_ROOT / model_slug(model) / experiment_id
    out_dir.mkdir(parents=True, exist_ok=True)

    records = [
        normalize_result(record, model=model, experiment_id=experiment_id)
        for record in data["results"]
    ]
    for record in records:
        out_path = out_dir / result_filename(
            experiment_id,
            record["test_name"],
            record["source"]["index"],
            record["input_sentence"],
        )
        record["result_file"] = str(out_path.relative_to(PROJECT_ROOT))
        out_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    summary = build_summary(records, model=model, experiment_id=experiment_id)
    summary_path = out_dir / f"{experiment_id}_summary.json"
    summary_path.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(summary_path)


if __name__ == "__main__":
    main()
