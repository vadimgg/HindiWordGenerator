#!/usr/bin/env python3
"""Record Hindi evaluator ratings into experiment result JSON files."""

from __future__ import annotations

import argparse
import json
from datetime import UTC, datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]


def resolve_project_path(raw_path: str) -> Path:
    path = Path(raw_path)
    if path.is_absolute():
        return path
    return PROJECT_ROOT / path


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def item_key(item: dict) -> tuple[str, str | int]:
    if item.get("result_file"):
        return ("result_file", item["result_file"])
    return (item["test_name"], int(item["source_index"]))


def evaluation_lookup(evaluation: dict) -> dict[tuple[str, str | int], dict]:
    lookup = {}
    for item in evaluation.get("items", []):
        key = item_key(item)
        if key in lookup:
            raise ValueError(f"Duplicate evaluator item for {key[0]}={key[1]}")
        lookup[key] = item
    return lookup


def normalized_evaluation_items(evaluation: dict) -> list[dict]:
    if "items" in evaluation:
        return evaluation["items"]
    return [evaluation]


def attach_evaluations(summary: dict, evaluation: dict) -> dict:
    lookup = evaluation_lookup({"items": normalized_evaluation_items(evaluation)})
    recorded_count = 0
    missing = []

    for result in summary.get("results", []):
        key = ("result_file", result.get("result_file"))
        rating = lookup.get(key)
        if rating is None:
            key = (result["test_name"], int(result["source"]["index"]))
            rating = lookup.get(key)
        if rating is None:
            missing.append({
                "result_file": result.get("result_file"),
                "test_name": result["test_name"],
                "source_index": result["source"]["index"],
            })
            continue

        result["evaluation"] = rating
        recorded_count += 1

        result_file = result.get("result_file")
        if result_file:
            result_path = resolve_project_path(result_file)
            result_data = load_json(result_path)
            result_data["evaluation"] = rating
            write_json(result_path, result_data)

    summary["evaluation"] = {
        "recorded_at": datetime.now(UTC).isoformat(),
        "verdict": evaluation.get("verdict"),
        "best_experiments": evaluation.get("best_experiments", []),
        "avoid_experiments": evaluation.get("avoid_experiments", []),
        "summary": evaluation.get("summary"),
        "recorded_items": recorded_count,
        "expected_items": len(summary.get("results", [])),
        "missing_items": missing,
    }
    return summary


def attach_single_evaluation(result: dict, evaluation: dict) -> dict:
    result["evaluation"] = evaluation
    return result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result_or_summary_json", help="Experiment result JSON or summary JSON to update in place.")
    parser.add_argument("evaluation_json", help="Raw evaluator JSON output.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    target_path = resolve_project_path(args.result_or_summary_json)
    evaluation_path = resolve_project_path(args.evaluation_json)

    target = load_json(target_path)
    evaluation = load_json(evaluation_path)
    if "results" in target:
        updated = attach_evaluations(target, evaluation)
        write_json(target_path, updated)
        print(json.dumps(updated["evaluation"], ensure_ascii=False, indent=2))
        return

    updated = attach_single_evaluation(target, evaluation)
    write_json(target_path, updated)
    print(json.dumps(updated["evaluation"], ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
