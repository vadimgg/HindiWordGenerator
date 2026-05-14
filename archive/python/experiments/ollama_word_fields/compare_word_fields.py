#!/usr/bin/env python3
"""Run field-level Ollama experiments for Hindi word-card generation."""
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
import importlib.util
import json
import re
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[2]
EXPERIMENT_ROOT = PROJECT_ROOT / "experiments" / "ollama_word_fields"
ADAPTER_ROOT = EXPERIMENT_ROOT / "adapters"
sys.path.insert(0, str(PROJECT_ROOT))

from llm_client import call_llm_with_retry, create_llm


@dataclass(frozen=True)
class Adapter:
    name: str
    path: Path
    module: object


WORD_FIXTURES = [
    {
        "item_id": "word_001",
        "source_path": "input/words/complete_hindi_chapter_01_words.csv",
        "source_index": 0,
        "title": "Complete Hindi",
        "subtitle": "Chapter 01",
        "hindi": "अच्छा",
        "romanisation": "acchā",
        "english": "good, nice; really?, Oh I see!",
    },
    {
        "item_id": "word_002",
        "source_path": "input/words/complete_hindi_chapter_01_words.csv",
        "source_index": 52,
        "title": "Complete Hindi",
        "subtitle": "Chapter 01",
        "hindi": "लड़की",
        "romanisation": "laṛkī",
        "english": "girl",
    },
    {
        "item_id": "word_003",
        "source_path": "input/words/complete_hindi_chapter_07_words.csv",
        "source_index": 6,
        "title": "Complete Hindi",
        "subtitle": "Chapter 07",
        "hindi": "जवाब देना",
        "romanisation": "javāb denā",
        "english": "to reply",
    },
]


def word_items() -> list[dict]:
    items = []
    for item in WORD_FIXTURES:
        row = dict(item)
        row["input_line"] = f"{row['hindi']} ({row['romanisation']});{row['english']}"
        items.append(row)
    return items


def parse_ollama_ps(output: str) -> list[dict]:
    rows = []
    lines = [line for line in output.splitlines() if line.strip()]
    if len(lines) < 2:
        return rows
    for line in lines[1:]:
        parts = line.split()
        if len(parts) < 7:
            continue
        rows.append({
            "name": parts[0],
            "id": parts[1],
            "raw": line,
        })
    return rows


def ollama_runtime(model_arg: str) -> dict:
    runtime = {
        "model_arg": model_arg,
        "ps_command": "ollama ps",
        "ps_stdout": "",
        "ps_stderr": "",
        "running_models": [],
        "requested_ollama_model": model_arg.split(":", 1)[1] if model_arg.startswith("ollama:") else None,
        "requested_model_is_running": None,
    }
    try:
        completed = subprocess.run(["ollama", "ps"], check=False, capture_output=True, text=True, timeout=10)
    except Exception as exc:
        runtime["ps_stderr"] = f"{type(exc).__name__}: {exc}"
        return runtime
    runtime["ps_stdout"] = completed.stdout
    runtime["ps_stderr"] = completed.stderr
    runtime["running_models"] = parse_ollama_ps(completed.stdout)
    if runtime["requested_ollama_model"] is not None:
        runtime["requested_model_is_running"] = any(
            row["name"] == runtime["requested_ollama_model"]
            for row in runtime["running_models"]
        )
    return runtime


def require_requested_ollama_model(runtime: dict) -> None:
    requested_model = runtime["requested_ollama_model"]
    if requested_model is None or runtime["requested_model_is_running"]:
        return
    running = ", ".join(row["name"] for row in runtime["running_models"]) or "none"
    raise RuntimeError(
        "Requested Ollama model is not currently running.\n"
        f"Requested: {requested_model}\n"
        f"Running: {running}\n"
        f"Start it with: ollama run {requested_model}"
    )


def adapter_names(raw_adapters: str) -> list[str]:
    return [adapter.strip() for adapter in raw_adapters.split(",") if adapter.strip()]


def load_adapters(names: list[str]) -> list[Adapter]:
    available = {
        path.stem: path
        for path in sorted(ADAPTER_ROOT.glob("*.py"))
        if path.name != "__init__.py" and not path.name.startswith("_")
    }
    selected_names = list(available) if names == ["all"] else names
    unknown = [name for name in selected_names if name not in available]
    if unknown:
        raise ValueError(f"Unknown adapter(s): {', '.join(unknown)}")
    adapters = []
    for name in selected_names:
        path = available[name]
        spec = importlib.util.spec_from_file_location(f"word_field_adapter_{name}", path)
        if spec is None or spec.loader is None:
            raise ValueError(f"Could not load adapter: {path}")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        adapters.append(Adapter(name=name, path=path, module=module))
    return adapters


def safe_slug(text: str, max_chars: int = 48) -> str:
    words = re.findall(r"[\w\u0900-\u097F]+", text, flags=re.UNICODE)
    slug = "_".join(words)[:max_chars].strip("_")
    return slug or "word"


def model_slug(model: str) -> str:
    return safe_slug(model.replace(":", "_").replace("/", "_"))


def experiment_output_dir(model: str, experiment_id: str) -> Path:
    return EXPERIMENT_ROOT / "results" / model_slug(model) / experiment_id


def result_filename(experiment_id: str, test_name: str, item: dict) -> str:
    return f"{experiment_id}_{safe_slug(test_name)}_{item['item_id']}_{safe_slug(item['hindi'])}.json"


def required_keys(test_name: str) -> set[str]:
    return {
        "word_raw_translation": {"english", "romanisation"},
        "word_source_row_synthesis": {"hindi", "romanisation", "english", "pos"},
        "word_grammar_core": {"pos"},
        "word_syllables_related": {"syllables", "related_words"},
        "word_example_sentence": {"example_sentence"},
        "word_delhi_usage": set(),
        "word_sound_alikes": set(),
        "word_etymology": set(),
    }.get(test_name, set())


def is_valid_result(test_name: str, result: dict) -> bool:
    if not isinstance(result, dict):
        return False
    return all(key in result for key in required_keys(test_name))


async def run_test_case(llm, test_case: dict, item: dict, args: argparse.Namespace, out_dir: Path) -> dict:
    label = f"{test_case['test_name']}:{item['hindi']}"
    print(f"Running {label}", file=sys.stderr, flush=True)
    started_at = time.perf_counter()
    started_at_iso = datetime.now(UTC).isoformat()
    try:
        result, usage = await asyncio.wait_for(
            call_llm_with_retry(
                llm,
                test_case["prompt"],
                json.dumps(test_case["input_payload"], ensure_ascii=False, indent=2),
                label,
            ),
            timeout=args.test_timeout_seconds,
        )
        duration = round(time.perf_counter() - started_at, 3)
        valid = is_valid_result(test_case["test_name"], result)
        record = {
            "experiment_id": args.experiment_id,
            "test_name": test_case["test_name"],
            "adapter": test_case["adapter"],
            "model": args.model,
            "ollama_runtime": args.ollama_runtime,
            "input_word": item["hindi"],
            "input_line": item["input_line"],
            "source": {
                "path": item["source_path"],
                "index": item["source_index"],
                "title": item["title"],
                "subtitle": item["subtitle"],
            },
            "reference": {
                "english": item["english"],
                "romanisation": item["romanisation"],
            },
            "full_prompt": test_case["prompt"],
            "input_payload": test_case["input_payload"],
            "timing": {
                "started_at": started_at_iso,
                "finished_at": datetime.now(UTC).isoformat(),
                "duration_seconds": duration,
            },
            "usage": usage,
            "result": result,
            "valid": valid,
        }
    except Exception as exc:
        duration = round(time.perf_counter() - started_at, 3)
        error_type = "TimeoutError" if isinstance(exc, TimeoutError) else type(exc).__name__
        record = {
            "experiment_id": args.experiment_id,
            "test_name": test_case["test_name"],
            "adapter": test_case["adapter"],
            "model": args.model,
            "ollama_runtime": args.ollama_runtime,
            "valid": False,
            "input_word": item["hindi"],
            "input_line": item["input_line"],
            "source": {
                "path": item["source_path"],
                "index": item["source_index"],
                "title": item["title"],
                "subtitle": item["subtitle"],
            },
            "reference": {
                "english": item["english"],
                "romanisation": item["romanisation"],
            },
            "full_prompt": test_case["prompt"],
            "input_payload": test_case["input_payload"],
            "timing": {
                "started_at": started_at_iso,
                "finished_at": datetime.now(UTC).isoformat(),
                "duration_seconds": duration,
            },
            "error": f"{error_type}: {exc}",
            "timed_out": isinstance(exc, TimeoutError),
        }
    record_path = out_dir / result_filename(args.experiment_id, test_case["test_name"], item)
    record["result_file"] = str(record_path.relative_to(PROJECT_ROOT))
    record_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return record


async def run_experiment(args: argparse.Namespace) -> dict:
    items = word_items()
    adapters = load_adapters(adapter_names(args.adapters))
    test_cases = []
    for adapter in adapters:
        for test_case in adapter.module.build_tests(items):
            test_cases.append({
                **test_case,
                "adapter": adapter.name,
                "adapter_path": str(adapter.path.relative_to(PROJECT_ROOT)),
            })
    args.ollama_runtime = ollama_runtime(args.model)
    require_requested_ollama_model(args.ollama_runtime)
    out_dir = experiment_output_dir(args.model, args.experiment_id)
    out_dir.mkdir(parents=True, exist_ok=True)
    llm = create_llm(args.model)
    started_at = time.perf_counter()
    started_at_iso = datetime.now(UTC).isoformat()
    results = []
    for test_case in test_cases:
        result = await run_test_case(llm, test_case, test_case["item"], args, out_dir)
        results.append(result)
        if args.stop_on_timeout and result.get("timed_out"):
            break

    by_test = {}
    for result in results:
        row = by_test.setdefault(result["test_name"], {"count": 0, "valid_count": 0, "model_call_seconds": 0})
        row["count"] += 1
        row["model_call_seconds"] = round(row["model_call_seconds"] + result["timing"]["duration_seconds"], 3)
        if result["valid"]:
            row["valid_count"] += 1
    for row in by_test.values():
        row["avg_model_call_seconds"] = round(row["model_call_seconds"] / row["count"], 3)

    wall_seconds = round(time.perf_counter() - started_at, 3)
    return {
        "experiment_id": args.experiment_id,
        "model": args.model,
        "ollama_runtime": args.ollama_runtime,
        "source_dir": "input/words",
        "adapters": [adapter.name for adapter in adapters],
        "word_count": len(items),
        "test_timeout_seconds": args.test_timeout_seconds,
        "stop_on_timeout": args.stop_on_timeout,
        "summary": {
            "count": len(results),
            "valid_count": sum(1 for result in results if result["valid"]),
            "wall_seconds": wall_seconds,
            "model_call_seconds": round(sum(result["timing"]["duration_seconds"] for result in results), 3),
            "timed_out_count": sum(1 for result in results if result.get("timed_out")),
        },
        "timing": {
            "started_at": started_at_iso,
            "finished_at": datetime.now(UTC).isoformat(),
            "wall_seconds": wall_seconds,
            "concurrency": 1,
        },
        "by_test": by_test,
        "results": results,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:gemma4:latest")
    parser.add_argument("--adapters", default="all")
    parser.add_argument("--test-timeout-seconds", type=float, default=180)
    parser.add_argument("--keep-going-on-timeout", action="store_true")
    parser.add_argument("--experiment-id", default=None)
    args = parser.parse_args()
    if args.experiment_id is None:
        stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        args.experiment_id = f"{model_slug(args.model)}_word_fields_{stamp}"
    args.stop_on_timeout = not args.keep_going_on_timeout
    return args


def main() -> None:
    args = parse_args()
    try:
        report = asyncio.run(run_experiment(args))
    except (RuntimeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc
    out_path = experiment_output_dir(report["model"], report["experiment_id"]) / f"{report['experiment_id']}_summary.json"
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        **report["summary"],
        "experiment_id": report["experiment_id"],
        "by_test": report["by_test"],
        "path": str(out_path),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
