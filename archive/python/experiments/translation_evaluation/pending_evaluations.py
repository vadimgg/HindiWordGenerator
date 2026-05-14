#!/usr/bin/env python3
"""List experiment result files that do not yet have evaluator verdicts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from build_evaluator_packet import packet_from_model_run, packet_from_result

PROJECT_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_RESULTS_ROOT = PROJECT_ROOT / "experiments" / "ollama_translation" / "results"


def resolve_path(raw_path: str) -> Path:
    path = Path(raw_path)
    if path.is_absolute():
        return path
    return PROJECT_ROOT / path


def is_result_file(path: Path, data: dict) -> bool:
    if path.name.endswith(("_summary.json", "_evaluator_packet.json", "_evaluation.json")):
        return False
    return all(key in data for key in ("experiment_id", "test_name", "input_sentence", "result_file"))


def has_verdict(data: dict) -> bool:
    return bool(data.get("evaluation", {}).get("verdict"))


def iter_json_files(paths: list[Path]) -> list[Path]:
    files = []
    for path in paths:
        if path.is_dir():
            files.extend(sorted(path.glob("**/*.json")))
        else:
            files.append(path)
    return files


def pending_files(paths: list[Path]) -> list[Path]:
    pending = []
    for path in iter_json_files(paths):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if is_result_file(path, data) and not has_verdict(data):
            pending.append(path)
    return pending


def packet_path_for_result(path: Path) -> Path:
    return path.with_name(f"{path.stem}_evaluator_packet.json")


def write_packets(paths: list[Path]) -> None:
    for path in paths:
        result = json.loads(path.read_text(encoding="utf-8"))
        packet = packet_from_result(result, path)
        packet_path = packet_path_for_result(path)
        packet_path.write_text(json.dumps(packet, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def write_model_run_packet(root: Path, include_evaluated: bool) -> Path:
    if not root.is_dir():
        raise ValueError("--write-model-run-packet requires a single experiment result directory.")
    packet = packet_from_model_run(root, include_evaluated=include_evaluated)
    out_path = root / f"{root.name}_model_run_evaluator_packet.json"
    out_path.write_text(json.dumps(packet, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return out_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="*",
        help="Result files or result directories. Defaults to all Ollama experiment results.",
    )
    parser.add_argument("--write-packets", action="store_true")
    parser.add_argument("--write-model-run-packet", action="store_true")
    parser.add_argument("--include-evaluated", action="store_true")
    parser.add_argument("--json", action="store_true", help="Print machine-readable JSON.")
    parser.add_argument("--limit", type=int, default=None)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    roots = [resolve_path(path) for path in args.paths] if args.paths else [DEFAULT_RESULTS_ROOT]
    paths = pending_files(roots)
    if args.limit is not None:
        paths = paths[: args.limit]

    model_run_packet = None
    if args.write_model_run_packet:
        if len(roots) != 1:
            raise SystemExit("--write-model-run-packet requires exactly one result directory path.")
        model_run_packet = write_model_run_packet(roots[0], include_evaluated=args.include_evaluated)

    if args.write_packets:
        write_packets(paths)

    rows = [
        {
            "result_file": str(path.relative_to(PROJECT_ROOT)),
            "packet_file": str(packet_path_for_result(path).relative_to(PROJECT_ROOT)),
        }
        for path in paths
    ]

    if args.json:
        print(json.dumps({
            "count": len(rows),
            "pending": rows,
            "model_run_packet": str(model_run_packet.relative_to(PROJECT_ROOT)) if model_run_packet else None,
        }, ensure_ascii=False, indent=2))
        return

    print(f"{len(rows)} result file(s) missing evaluation.verdict")
    if model_run_packet:
        print(f"Model-run packet: {model_run_packet.relative_to(PROJECT_ROOT)}")
    for row in rows:
        print(row["result_file"])


if __name__ == "__main__":
    main()
