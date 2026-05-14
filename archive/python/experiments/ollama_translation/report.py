#!/usr/bin/env python3
"""Build a compact JSON comparison report from Ollama translation experiment summaries."""

from __future__ import annotations

import argparse
import json
from datetime import UTC, datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
RESULTS_ROOT = PROJECT_ROOT / "experiments" / "ollama_translation" / "results"


def summary_files(paths: list[str]) -> list[Path]:
    if not paths:
        return sorted(RESULTS_ROOT.glob("*/*/*_summary.json"))

    files = []
    for raw_path in paths:
        path = Path(raw_path)
        if not path.is_absolute():
            path = PROJECT_ROOT / path
        if path.is_dir():
            files.extend(sorted(path.glob("*_summary.json")))
        else:
            files.append(path)
    return files


def row_from_summary(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    return {
        "experiment_id": data["experiment_id"],
        "model": data["model"],
        "model_folder": path.relative_to(RESULTS_ROOT).parts[0],
        "summary_file": str(path.relative_to(PROJECT_ROOT)),
        "count": data["summary"]["count"],
        "valid_count": data["summary"]["valid_count"],
        "normalized_match_count": data["summary"]["normalized_match_count"],
        "wall_seconds": data["summary"]["wall_seconds"],
        "model_call_seconds": data["summary"]["model_call_seconds"],
        "by_test": data["by_test"],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", nargs="*", help="Summary files or experiment result directories.")
    parser.add_argument("--out", default=None)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    rows = [row_from_summary(path) for path in summary_files(args.paths)]
    report = {
        "generated_at": datetime.now(UTC).isoformat(),
        "count": len(rows),
        "experiments": rows,
    }

    if args.out:
        out_path = Path(args.out)
        if not out_path.is_absolute():
            out_path = PROJECT_ROOT / out_path
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        print(out_path)
        return

    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
