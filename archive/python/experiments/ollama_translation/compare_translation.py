#!/usr/bin/env python3
"""Run adapter-based Ollama translation experiments against approved sentence cards."""
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
EXPERIMENT_ROOT = PROJECT_ROOT / "experiments" / "ollama_translation"
ADAPTER_ROOT = EXPERIMENT_ROOT / "adapters"
sys.path.insert(0, str(PROJECT_ROOT))

from llm_client import call_llm_with_retry, create_llm


@dataclass(frozen=True)
class Adapter:
    name: str
    path: Path
    module: object


def load_sentence_items(source_dir: Path, max_batches: int, batch_size: int) -> list[dict]:
    """Read sampled sentence cards from existing approved output batches."""
    items = []
    for file_index, path in enumerate(sorted(source_dir.glob("*.json"))):
        if file_index >= max_batches:
            break

        data = json.loads(path.read_text(encoding="utf-8"))
        for sentence_index, sentence in enumerate(data.get("sentences", [])[:batch_size]):
            item_number = len(items) + 1
            hindi = sentence.get("hindi", "")
            romanisation = sentence.get("romanisation", "")
            english = sentence.get("english", "")
            items.append({
                "item_id": f"sentence_{item_number:03d}",
                "source_path": str(path.relative_to(PROJECT_ROOT)),
                "source_index": sentence_index,
                "title": data.get("title"),
                "subtitle": data.get("subtitle"),
                "hindi": hindi,
                "romanisation": romanisation,
                "original_english": english,
                "input_line": f"{hindi} ({romanisation});{english}",
                "literal": sentence.get("literal", ""),
                "register": sentence.get("register", ""),
            })
    return items


def ollama_runtime(model_arg: str) -> dict:
    """Inspect locally running Ollama models for the experiment report."""
    runtime = {
        "model_arg": model_arg,
        "ps_command": "ollama ps",
        "ps_stdout": "",
        "ps_stderr": "",
        "running_models": [],
        "selected_running_model": None,
        "requested_ollama_model": model_arg.split(":", 1)[1]
        if model_arg.startswith("ollama:")
        else None,
        "requested_model_is_running": None,
    }

    try:
        completed = subprocess.run(
            ["ollama", "ps"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except Exception as exc:
        runtime["ps_stderr"] = f"{type(exc).__name__}: {exc}"
        return runtime

    runtime["ps_stdout"] = completed.stdout
    runtime["ps_stderr"] = completed.stderr
    runtime["running_models"] = parse_ollama_ps(completed.stdout)

    if runtime["running_models"]:
        runtime["selected_running_model"] = runtime["running_models"][0]["name"]

    if runtime["requested_ollama_model"] is not None:
        runtime["requested_model_is_running"] = any(
            row["name"] == runtime["requested_ollama_model"]
            for row in runtime["running_models"]
        )

    return runtime


def require_requested_ollama_model(runtime: dict) -> None:
    """Fail early when an Ollama experiment targets a model that is not loaded."""
    requested_model = runtime["requested_ollama_model"]
    if requested_model is None:
        return

    if runtime["requested_model_is_running"]:
        return

    running_models = [row["name"] for row in runtime["running_models"]]
    running_text = ", ".join(running_models) if running_models else "none"
    detail = runtime["ps_stderr"].strip()
    if detail:
        detail = f"\nollama ps stderr: {detail}"

    raise RuntimeError(
        "Requested Ollama model is not currently running.\n"
        f"Requested: {requested_model}\n"
        f"Running: {running_text}\n"
        f"Command checked: {runtime['ps_command']}"
        f"{detail}\n"
        f"Start it with: ollama run {requested_model}"
    )


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
            "size": f"{parts[2]} {parts[3]}",
            "processor": f"{parts[4]} {parts[5]}",
            "context": parts[6],
            "until": " ".join(parts[7:]) if len(parts) > 7 else "",
            "raw": line,
        })
    return rows


def normalize_english(text: str) -> str:
    text = text.lower()
    text = text.replace("’", "'")
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


def compare_issue_detection(expected_issue: bool, result: dict) -> dict:
    generated_issue = bool(result.get("has_issue"))
    return {
        "exact_match": expected_issue == generated_issue,
        "normalized_match": expected_issue == generated_issue,
        "original_normalized": str(expected_issue).lower(),
        "generated_normalized": str(generated_issue).lower(),
    }


def compare_result(item: dict, result: dict) -> dict | None:
    if "expected_issue" in item:
        return compare_issue_detection(bool(item["expected_issue"]), result)

    if "register" in result:
        return compare_register(item["register"], str(result.get("register", "")))

    generated_english = str(result.get("english", "")).strip()
    if generated_english:
        return compare_translation(item["original_english"], generated_english)

    return None


def is_valid_result(result: dict) -> bool:
    if "has_issue" in result:
        return isinstance(result.get("has_issue"), bool)
    if "register" in result:
        return bool(str(result.get("register", "")).strip())
    return bool(str(result.get("english", "")).strip())


def load_adapters(adapter_names: list[str]) -> list[Adapter]:
    adapters = []
    available = {
        path.stem: path
        for path in sorted(ADAPTER_ROOT.glob("*.py"))
        if path.name != "__init__.py" and not path.name.startswith("_")
    }
    selected_names = list(available) if adapter_names == ["all"] else adapter_names
    unknown = [name for name in selected_names if name not in available]
    if unknown:
        raise ValueError(f"Unknown adapter(s): {', '.join(unknown)}")

    for name in selected_names:
        path = available[name]
        spec = importlib.util.spec_from_file_location(f"ollama_translation_adapter_{name}", path)
        if spec is None or spec.loader is None:
            raise ValueError(f"Could not load adapter: {path}")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        if not hasattr(module, "build_tests"):
            raise ValueError(f"Adapter {path} does not define build_tests(item).")
        adapters.append(Adapter(name=name, path=path, module=module))
    return adapters


def adapter_names(raw_adapters: str) -> list[str]:
    return [adapter.strip() for adapter in raw_adapters.split(",") if adapter.strip()]


def safe_slug(text: str, max_chars: int = 48) -> str:
    words = re.findall(r"[\w\u0900-\u097F]+", text, flags=re.UNICODE)
    slug = "_".join(words)[:max_chars].strip("_")
    return slug or "sentence"


def model_slug(model: str) -> str:
    return safe_slug(model.replace(":", "_"))


def result_filename(experiment_id: str, test_name: str, item: dict) -> str:
    sentence_slug = safe_slug(item["hindi"], max_chars=36)
    return f"{experiment_id}_{safe_slug(test_name)}_{item['item_id']}_{sentence_slug}.json"


def experiment_output_dir(model: str, experiment_id: str) -> Path:
    return EXPERIMENT_ROOT / "results" / model_slug(model) / experiment_id


async def run_test_case(llm, test_case: dict, item: dict, args: argparse.Namespace, out_dir: Path) -> dict:
    label = f"{test_case['test_name']}:{item['source_path']}#{item['source_index'] + 1}"
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
        duration_seconds = round(time.perf_counter() - started_at, 3)
        comparison = compare_result(item, result)
        record = {
            "experiment_id": args.experiment_id,
            "test_name": test_case["test_name"],
            "adapter": test_case["adapter"],
            "model": args.model,
            "ollama_runtime": args.ollama_runtime,
            "input_sentence": item["hindi"],
            "input_line": item["input_line"],
            "source": {
                "path": item["source_path"],
                "index": item["source_index"],
                "title": item["title"],
                "subtitle": item["subtitle"],
            },
            "reference": {
                "english": item["original_english"],
                "romanisation": item["romanisation"],
                "register": item["register"],
                "expected_issue": item.get("expected_issue"),
                "expected_issue_kind": item.get("expected_issue_kind"),
            },
            "full_prompt": test_case["prompt"],
            "input_payload": test_case["input_payload"],
            "timing": {
                "started_at": started_at_iso,
                "finished_at": datetime.now(UTC).isoformat(),
                "duration_seconds": duration_seconds,
            },
            "usage": usage,
            "result": result,
            "comparison": comparison,
            "valid": is_valid_result(result),
        }
    except Exception as exc:
        duration_seconds = round(time.perf_counter() - started_at, 3)
        error_type = "TimeoutError" if isinstance(exc, TimeoutError) else type(exc).__name__
        record = {
            "experiment_id": args.experiment_id,
            "test_name": test_case["test_name"],
            "adapter": test_case["adapter"],
            "model": args.model,
            "ollama_runtime": args.ollama_runtime,
            "valid": False,
            "input_sentence": item["hindi"],
            "input_line": item["input_line"],
            "source": {
                "path": item["source_path"],
                "index": item["source_index"],
                "title": item["title"],
                "subtitle": item["subtitle"],
            },
            "reference": {
                "english": item["original_english"],
                "romanisation": item["romanisation"],
                "register": item["register"],
                "expected_issue": item.get("expected_issue"),
                "expected_issue_kind": item.get("expected_issue_kind"),
            },
            "full_prompt": test_case["prompt"],
            "input_payload": test_case["input_payload"],
            "timing": {
                "started_at": started_at_iso,
                "finished_at": datetime.now(UTC).isoformat(),
                "duration_seconds": duration_seconds,
            },
            "error": f"{error_type}: {exc}",
            "timed_out": isinstance(exc, TimeoutError),
        }

    record_path = out_dir / result_filename(args.experiment_id, test_case["test_name"], item)
    record["result_file"] = str(record_path.relative_to(PROJECT_ROOT))
    record_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return record


async def run_experiment(args: argparse.Namespace) -> dict:
    source_dir = Path(args.source_dir)
    if not source_dir.is_absolute():
        source_dir = PROJECT_ROOT / source_dir

    items = load_sentence_items(source_dir, args.max_batches, args.batch_size)
    if not items:
        raise ValueError(f"No sentence cards found in {source_dir}")

    adapters = load_adapters(adapter_names(args.adapters))
    test_cases = []
    for adapter in adapters:
        for test_case in adapter.module.build_tests(items):
            test_cases.append({
                **test_case,
                "adapter": adapter.name,
                "adapter_path": str(adapter.path.relative_to(PROJECT_ROOT)),
            })

    if not test_cases:
        raise ValueError("No test cases were produced by the selected adapters.")

    args.ollama_runtime = ollama_runtime(args.model)
    require_requested_ollama_model(args.ollama_runtime)

    out_dir = experiment_output_dir(args.model, args.experiment_id)
    out_dir.mkdir(parents=True, exist_ok=True)

    llm = create_llm(args.model)
    started_at = time.perf_counter()
    started_at_iso = datetime.now(UTC).isoformat()
    results = []
    stopped_early = False
    stop_reason = None

    for test_case in test_cases:
        result = await run_test_case(llm, test_case, test_case["item"], args, out_dir)
        results.append(result)
        if args.stop_on_timeout and result.get("timed_out"):
            stopped_early = True
            stop_reason = f"Stopped after timeout in {result['test_name']}."
            break
    wall_seconds = round(time.perf_counter() - started_at, 3)
    valid_results = [result for result in results if result["valid"]]

    by_test = {}
    for result in results:
        summary = by_test.setdefault(
            result["test_name"],
            {
                "count": 0,
                "valid_count": 0,
                "normalized_match_count": 0,
                "model_call_seconds": 0,
            },
        )
        summary["count"] += 1
        summary["model_call_seconds"] = round(
            summary["model_call_seconds"] + result["timing"]["duration_seconds"], 3
        )
        if result["valid"]:
            summary["valid_count"] += 1
            if result.get("comparison", {}).get("normalized_match"):
                summary["normalized_match_count"] += 1

    for summary in by_test.values():
        summary["avg_model_call_seconds"] = round(
            summary["model_call_seconds"] / summary["count"], 3
        )

    return {
        "experiment_id": args.experiment_id,
        "model": args.model,
        "ollama_runtime": args.ollama_runtime,
        "source_dir": str(source_dir.relative_to(PROJECT_ROOT)),
        "adapters": [adapter.name for adapter in adapters],
        "batch_size": args.batch_size,
        "max_batches": args.max_batches,
        "concurrency": args.concurrency,
        "test_timeout_seconds": args.test_timeout_seconds,
        "stop_on_timeout": args.stop_on_timeout,
        "stopped_early": stopped_early,
        "stop_reason": stop_reason,
        "summary": {
            "count": len(results),
            "valid_count": len(valid_results),
            "normalized_match_count": sum(
                1 for result in valid_results if result.get("comparison", {}).get("normalized_match")
            ),
            "wall_seconds": wall_seconds,
            "model_call_seconds": round(
                sum(result["timing"]["duration_seconds"] for result in results), 3
            ),
            "timed_out_count": sum(1 for result in results if result.get("timed_out")),
        },
        "timing": {
            "started_at": started_at_iso,
            "finished_at": datetime.now(UTC).isoformat(),
            "wall_seconds": wall_seconds,
            "concurrency": args.concurrency,
        },
        "by_test": by_test,
        "results": results,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--model", default="ollama:gemma4:latest")
    parser.add_argument("--type", choices=["sentences"], default="sentences")
    parser.add_argument("--source-dir", default="output/sentences")
    parser.add_argument("--batch-size", type=int, default=1, help="Sentences sampled from each batch file.")
    parser.add_argument("--max-batches", type=int, default=1, help="Sentence batch files to sample.")
    parser.add_argument("--concurrency", type=int, default=1)
    parser.add_argument("--adapters", default="all", help="Comma-separated adapter names, or all.")
    parser.add_argument("--test-timeout-seconds", type=float, default=180)
    parser.add_argument("--keep-going-on-timeout", action="store_true")
    parser.add_argument("--experiment-id", default=None)
    parser.add_argument("--out", default=None)
    args = parser.parse_args()
    if args.experiment_id is None:
        stamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        args.experiment_id = f"{model_slug(args.model)}_{stamp}"
    args.stop_on_timeout = not args.keep_going_on_timeout
    return args


def main() -> None:
    args = parse_args()
    try:
        report = asyncio.run(run_experiment(args))
    except (RuntimeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc

    if args.out:
        out_path = Path(args.out)
    else:
        out_path = (
            experiment_output_dir(report["model"], report["experiment_id"])
            / f"{report['experiment_id']}_summary.json"
        )
    if not out_path.is_absolute():
        out_path = PROJECT_ROOT / out_path

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({
        **report["summary"],
        "experiment_id": report["experiment_id"],
        "by_test": report["by_test"],
        "path": str(out_path),
    }, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
