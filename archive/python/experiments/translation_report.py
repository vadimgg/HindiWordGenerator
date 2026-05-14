#!/usr/bin/env python3
"""Print a colorful CLI comparison table for translation experiment summaries."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import textwrap
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parents[1]
OLLAMA_RESULTS = PROJECT_ROOT / "experiments" / "ollama_translation" / "results"
AGENT_RESULTS = PROJECT_ROOT / "experiments" / "agent_translation" / "results"

RESET = "\033[0m"
BOLD = "\033[1m"
DIM = "\033[2m"
GREEN = "\033[32m"
YELLOW = "\033[33m"
RED = "\033[31m"
CYAN = "\033[36m"
MAGENTA = "\033[35m"
BLUE = "\033[34m"


def color(text: str, code: str, enabled: bool) -> str:
    return f"{code}{text}{RESET}" if enabled else text


def all_summary_files() -> list[Path]:
    return [
        *sorted(OLLAMA_RESULTS.glob("*/*/*_summary.json")),
        *sorted(AGENT_RESULTS.glob("*/*/*_summary.json")),
    ]


def summary_sort_time(path: Path) -> str:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return ""
    return data.get("timing", {}).get("finished_at") or data.get("evaluation", {}).get("recorded_at") or ""


def latest_evaluated_summary_files() -> list[Path]:
    by_model: dict[str, list[Path]] = {}
    for path in all_summary_files():
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError):
            continue
        if not data.get("evaluation", {}).get("recorded_items"):
            continue
        by_model.setdefault(data["model"], []).append(path)

    selected = []
    for paths in by_model.values():
        selected.append(max(paths, key=summary_sort_time))
    return sorted(selected)


def summary_files(paths: list[str], include_all_runs: bool = False) -> list[Path]:
    if paths:
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

    if include_all_runs:
        return all_summary_files()
    return latest_evaluated_summary_files()


def load_summary(path: Path) -> dict:
    data = json.loads(path.read_text(encoding="utf-8"))
    return {
        "path": path,
        "experiment_id": data["experiment_id"],
        "model": data["model"],
        "count": data["summary"]["count"],
        "valid_count": data["summary"]["valid_count"],
        "match_count": data["summary"].get("normalized_match_count", 0),
        "wall_seconds": data["summary"].get("wall_seconds", 0),
        "timed_out_count": data["summary"].get("timed_out_count", 0),
        "test_timeout_seconds": data.get("test_timeout_seconds"),
        "by_test": data.get("by_test", {}),
        "evaluation": data.get("evaluation", {}),
        "results": data.get("results", []),
    }


def fmt_seconds(seconds: float) -> str:
    if seconds >= 60:
        return f"{seconds / 60:.1f}m"
    return f"{seconds:.1f}s"


def fmt_model(model: str) -> str:
    aliases = {
        "codex_agent_no_context": "codex_agent",
    }
    if model in aliases:
        return aliases[model]
    if model.startswith("ollama:"):
        return model.removeprefix("ollama:")
    return model


def fmt_experiment(test_name: str) -> str:
    aliases = {
        "hindi_gloss_guided_translation": "gloss-guided",
        "hindi_simple_translation_romanisation": "simple + romanisation",
        "hindi_strict_translation_romanisation": "strict + romanisation",
        "hindi_word_breakdown_translation": "word breakdown",
        "register_detection": "register detection",
        "source_row_issue_detection": "source-row issue detection",
        "source_row_simple_translation_romanisation": "source-row simple",
        "source_row_word_breakdown_translation": "source-row word breakdown",
    }
    return aliases.get(test_name, test_name.replace("_", " "))


def score_bar(score: float | None, width: int = 20) -> str:
    if score is None:
        return "░" * width
    filled = max(0, min(width, round((score / 5) * width)))
    return "█" * filled + "░" * (width - filled)


def score_color(match_count: int, count: int) -> str:
    if count == 0:
        return RED
    ratio = match_count / count
    if ratio >= 0.9:
        return GREEN
    if ratio >= 0.5:
        return YELLOW
    return RED


def timeout_color(timeout_count: int) -> str:
    return RED if timeout_count else GREEN


def verdict_color(verdict: str) -> str:
    return {
        "good": GREEN,
        "usable": CYAN,
        "weak": YELLOW,
        "bad": RED,
    }.get(verdict, DIM)


def json_status(row: dict) -> str:
    if row["timed_out_count"] and row["valid_count"] < row["count"]:
        return "-"
    if row["valid_count"] == row["count"]:
        return "✓"
    return "✗"


def timeout_label(row: dict) -> str:
    if row["timed_out_count"]:
        return fmt_seconds(row["time_seconds"])
    return ""


def rows_from_summary(summary: dict) -> list[dict]:
    rows = []
    for test_name, by_test in sorted(summary["by_test"].items()):
        count = by_test["count"]
        match_count = by_test.get("normalized_match_count", 0)
        avg_seconds = by_test.get("avg_model_call_seconds", 0)
        rows.append({
            "model": summary["model"],
            "run_id": summary["experiment_id"],
            "experiment": test_name,
            "match_count": match_count,
            "count": count,
            "valid_count": by_test.get("valid_count", 0),
            "time_seconds": avg_seconds,
            "run_seconds": summary["wall_seconds"],
            "timed_out_count": summary["timed_out_count"],
            "timeout_seconds": summary.get("test_timeout_seconds"),
        })
    return rows


def row_cells(row: dict) -> list[str]:
    return [
        row["model"],
        row["experiment"],
        f"{row['match_count']}/{row['count']}",
        json_status(row),
        fmt_seconds(row["time_seconds"]),
        timeout_label(row),
    ]


def row_sort_key(row: dict, sort_mode: str) -> tuple:
    match_ratio = row["match_count"] / row["count"] if row["count"] else 0
    if sort_mode == "name":
        return (row["model"], row["experiment"], row["run_id"])
    return (
        row["experiment"],
        -match_ratio,
        row["time_seconds"],
        row["timed_out_count"],
        row["model"],
        row["run_id"],
    )


def experiment_sort_key(summary: dict, sort_mode: str) -> tuple:
    match_ratio = summary["match_count"] / summary["count"] if summary["count"] else 0
    if sort_mode == "name":
        return (summary["model"], summary["experiment_id"])
    return (
        -match_ratio,
        summary["wall_seconds"],
        summary["timed_out_count"],
        summary["model"],
        summary["experiment_id"],
    )


def table_widths(rows: list[dict], headers: list[str]) -> list[int]:
    widths = [len(header) for header in headers]
    max_width = max(92, shutil.get_terminal_size((120, 24)).columns)
    for row in rows:
        for index, cell in enumerate(row_cells(row)):
            widths[index] = max(widths[index], min(len(cell), 36))
    overflow = sum(widths) + (len(widths) * 3) - max_width
    if overflow > 0:
        for index in (1, 2):
            shrink = min(overflow, max(0, widths[index] - 22))
            widths[index] -= shrink
            overflow -= shrink
            if overflow <= 0:
                break
    return widths


def widths_for_cells(rows: list[list[str]], headers: list[str]) -> list[int]:
    widths = [len(header) for header in headers]
    max_width = max(92, shutil.get_terminal_size((120, 24)).columns)
    for row in rows:
        for index, cell in enumerate(row):
            widths[index] = max(widths[index], min(len(cell), 36))
    overflow = sum(widths) + (len(widths) * 3) - max_width
    if overflow > 0:
        for index in range(len(widths)):
            shrink = min(overflow, max(0, widths[index] - 14))
            widths[index] -= shrink
            overflow -= shrink
            if overflow <= 0:
                break
    return widths


def clip(text: str, width: int) -> str:
    if len(text) <= width:
        return text
    return f"{text[: max(0, width - 1)]}…"


def sorted_summaries(summaries: list[dict], sort_mode: str) -> list[dict]:
    return sorted(summaries, key=lambda item: experiment_sort_key(item, sort_mode))


def print_table(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    headers = ["Model", "Experiment", "Exact", "JSON", "Time", "Timeout"]
    rows = [row for summary in summaries for row in rows_from_summary(summary)]
    rows.sort(key=lambda row: row_sort_key(row, sort_mode))
    widths = table_widths(rows, headers)

    title = "Translation Experiment Comparison"
    print(color(f"\n{title}", BOLD + CYAN, use_color))
    print(color(
        "Grouped by prompt experiment; rows rank by best match rate, then fastest time. "
        "Exact = count of generated target values that match the reference after normalization "
        "(English for translation tests, register for register tests).\n",
        DIM,
        use_color,
    ))

    header_line = "  ".join(header.ljust(widths[i]) for i, header in enumerate(headers))
    print(color(header_line, BOLD, use_color))
    print(color("-" * len(header_line), DIM, use_color))

    for row in rows:
        row_color = score_color(row["match_count"], row["count"])
        timeout_count = row["timed_out_count"]
        cells = [clip(cell, widths[i]).ljust(widths[i]) for i, cell in enumerate(row_cells(row))]
        cells[2] = color(cells[2], row_color, use_color)
        cells[5] = color(cells[5], timeout_color(timeout_count), use_color)
        print("  ".join(cells))

    print()


def evaluation_counts(results: list[dict]) -> dict[str, int]:
    counts = {"good": 0, "usable": 0, "weak": 0, "bad": 0, "missing": 0}
    for result in results:
        verdict = result.get("evaluation", {}).get("verdict")
        if verdict in counts:
            counts[verdict] += 1
        else:
            counts["missing"] += 1
    return counts


def evaluation_rows_from_summary(summary: dict) -> list[dict]:
    by_test: dict[str, list[dict]] = {}
    for result in summary["results"]:
        by_test.setdefault(result.get("test_name", "unknown"), []).append(result)

    rows = []
    for test_name, results in sorted(by_test.items()):
        counts = evaluation_counts(results)
        scored = [
            result.get("evaluation", {}).get("learner_usefulness")
            for result in results
            if isinstance(result.get("evaluation", {}).get("learner_usefulness"), int)
        ]
        avg_usefulness = sum(scored) / len(scored) if scored else None
        rows.append({
            "model": summary["model"],
            "run_id": summary["experiment_id"],
            "experiment": test_name,
            "count": len(results),
            "counts": counts,
            "avg_usefulness": avg_usefulness,
        })
    return rows


def evaluation_row_cells(row: dict) -> list[str]:
    counts = row["counts"]
    avg = "-" if row["avg_usefulness"] is None else f"{row['avg_usefulness']:.1f}"
    return [
        row["model"],
        row["experiment"],
        str(row["count"]),
        str(counts["good"]),
        str(counts["usable"]),
        str(counts["weak"]),
        str(counts["bad"]),
        str(counts["missing"]),
        avg,
    ]


def evaluation_row_sort_key(row: dict, sort_mode: str) -> tuple:
    counts = row["counts"]
    quality_score = (
        counts["good"] * 3
        + counts["usable"] * 2
        + counts["weak"]
        - counts["bad"] * 2
        - counts["missing"] * 3
    )
    if sort_mode == "name":
        return (row["model"], row["experiment"], row["run_id"])
    return (
        row["experiment"],
        -quality_score,
        row["model"],
        row["run_id"],
    )


def print_evaluations(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    headers = ["Model", "Experiment", "N", "Good", "Usable", "Weak", "Bad", "Missing", "Use"]
    rows = [row for summary in summaries for row in evaluation_rows_from_summary(summary)]
    rows.sort(key=lambda row: evaluation_row_sort_key(row, sort_mode))
    cell_rows = [evaluation_row_cells(row) for row in rows]
    widths = widths_for_cells(cell_rows, headers)

    print(color("\nTranslation Quality Evaluation", BOLD + CYAN, use_color))
    print(color(
        "Grouped by prompt experiment; counts come from each result file's evaluation.verdict. "
        "Use = average learner_usefulness.\n",
        DIM,
        use_color,
    ))

    for summary in sorted_summaries(summaries, sort_mode):
        evaluation = summary.get("evaluation", {})
        verdict = evaluation.get("verdict", "missing")
        recorded = evaluation.get("recorded_items", 0)
        expected = evaluation.get("expected_items", summary["count"])
        summary_text = evaluation.get("summary", "")
        print(
            f"- {summary['model']}  "
            f"{color(summary['experiment_id'], BLUE, use_color)}  "
            f"{color(verdict, verdict_color(verdict), use_color)}  "
            f"{recorded}/{expected} evaluated"
        )
        if summary_text:
            print(color(f"  {summary_text}", DIM, use_color))
    print()

    header_line = "  ".join(header.ljust(widths[i]) for i, header in enumerate(headers))
    print(color(header_line, BOLD, use_color))
    print(color("-" * len(header_line), DIM, use_color))

    for row, cells in zip(rows, cell_rows, strict=True):
        cells = [clip(cell, widths[i]).ljust(widths[i]) for i, cell in enumerate(cells)]
        cells[3] = color(cells[3], GREEN, use_color)
        cells[4] = color(cells[4], CYAN, use_color)
        cells[5] = color(cells[5], YELLOW, use_color)
        cells[6] = color(cells[6], RED, use_color)
        cells[7] = color(cells[7], RED if row["counts"]["missing"] else GREEN, use_color)
        print("  ".join(cells))

    print()


def print_evaluation_glossary(use_color: bool) -> None:
    print(color("\nEvaluation Glossary", BOLD + CYAN, use_color))
    rows = [
        ("Good", "Safe/useful result. Minor polish may remain, but it basically works."),
        ("Usable", "Meaning is mostly right, but needs validation or cleanup before real cards."),
        ("Weak", "Not reliable enough for production cards; useful mainly as experiment signal."),
        ("Bad", "Actively wrong or misleading for learner-facing data."),
        ("Missing", "No evaluator verdict has been recorded for those result files yet."),
        ("Use", "Average learner_usefulness score from 1-5; 5 is very useful, 1 is misleading or not useful."),
    ]
    width = max(len(label) for label, _description in rows)
    for label, description in rows:
        print(f"- {color(label.ljust(width), verdict_color(label.lower()), use_color)}  {description}")
    print()


def detail_sort_key(result: dict, sort_mode: str) -> tuple:
    evaluation = result.get("evaluation", {})
    verdict_rank = {"bad": 0, "weak": 1, "usable": 2, "good": 3}
    if sort_mode == "name":
        return (result.get("test_name", ""), result.get("source", {}).get("index", 0))
    return (
        verdict_rank.get(evaluation.get("verdict"), -1),
        result.get("test_name", ""),
        result.get("source", {}).get("index", 0),
    )


def print_evaluation_comments(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    print(color("\nEvaluator Comments", BOLD + CYAN, use_color))
    for summary in sorted_summaries(summaries, sort_mode):
        print(
            f"{summary['model']}  "
            f"{color(summary['experiment_id'], BLUE, use_color)}"
        )
        results = sorted(summary["results"], key=lambda result: detail_sort_key(result, sort_mode))
        for result in results:
            evaluation = result.get("evaluation", {})
            verdict = evaluation.get("verdict", "missing")
            source_index = result.get("source", {}).get("index", 0) + 1
            print(
                f"- {color(verdict, verdict_color(verdict), use_color)}  "
                f"{result.get('test_name', 'unknown')}  "
                f"#{source_index}: {result.get('input_sentence', '')}"
            )
            for bullet in evaluation.get("bullet_points", []):
                print(f"  - {bullet}")
            comment = evaluation.get("comment")
            if comment:
                print(color(f"  comment: {comment}", DIM, use_color))
        print()


def model_summary_rows(summaries: list[dict]) -> list[dict]:
    by_model: dict[str, list[dict]] = {}
    for summary in summaries:
        by_model.setdefault(summary["model"], []).append(summary)

    rows = []
    for model, model_summaries in sorted(by_model.items()):
        total_count = sum(summary["count"] for summary in model_summaries)
        match_count = sum(summary["match_count"] for summary in model_summaries)
        valid_count = sum(summary["valid_count"] for summary in model_summaries)
        wall_seconds = sum(summary["wall_seconds"] for summary in model_summaries)
        timed_out_count = sum(summary["timed_out_count"] for summary in model_summaries)
        all_results = [result for summary in model_summaries for result in summary["results"]]
        counts = evaluation_counts(all_results)
        scored = [
            result.get("evaluation", {}).get("learner_usefulness")
            for result in all_results
            if isinstance(result.get("evaluation", {}).get("learner_usefulness"), int)
        ]
        avg_usefulness = sum(scored) / len(scored) if scored else None
        rows.append({
            "model": model,
            "runs": len(model_summaries),
            "count": total_count,
            "valid_count": valid_count,
            "match_count": match_count,
            "wall_seconds": wall_seconds,
            "timed_out_count": timed_out_count,
            "counts": counts,
            "avg_usefulness": avg_usefulness,
        })
    return rows


def model_summary_cells(row: dict) -> list[str]:
    counts = row["counts"]
    exact = f"{row['match_count']}/{row['count']}"
    json_cell = f"{row['valid_count']}/{row['count']}"
    score = "-" if row["avg_usefulness"] is None else f"{row['avg_usefulness']:.1f}"
    return [
        row["model"],
        str(row["runs"]),
        exact,
        json_cell,
        fmt_seconds(row["wall_seconds"]),
        str(row["timed_out_count"]),
        str(counts["good"]),
        str(counts["usable"]),
        str(counts["weak"]),
        str(counts["bad"]),
        str(counts["missing"]),
        score,
    ]


def model_summary_sort_key(row: dict, sort_mode: str) -> tuple:
    exact_ratio = row["match_count"] / row["count"] if row["count"] else 0
    score = row["avg_usefulness"] or 0
    if sort_mode == "name":
        return (row["model"],)
    return (-score, -exact_ratio, row["timed_out_count"], row["model"])


def print_model_summary(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    headers = ["Model", "Runs", "Exact", "JSON", "Wall", "TO", "Good", "Usable", "Weak", "Bad", "Miss", "Score"]
    rows = sorted(model_summary_rows(summaries), key=lambda row: model_summary_sort_key(row, sort_mode))
    cell_rows = [model_summary_cells(row) for row in rows]
    widths = widths_for_cells(cell_rows, headers)

    print(color("\nModel Summary", BOLD + CYAN, use_color))
    print(color(
        "Aggregated across selected runs. Score = average evaluator learner_usefulness from 1-5.\n",
        DIM,
        use_color,
    ))

    header_line = "  ".join(header.ljust(widths[i]) for i, header in enumerate(headers))
    print(color(header_line, BOLD, use_color))
    print(color("-" * len(header_line), DIM, use_color))
    for row, cells in zip(rows, cell_rows, strict=True):
        cells = [clip(cell, widths[i]).ljust(widths[i]) for i, cell in enumerate(cells)]
        cells[2] = color(cells[2], score_color(row["match_count"], row["count"]), use_color)
        cells[5] = color(cells[5], RED if row["timed_out_count"] else GREEN, use_color)
        cells[6] = color(cells[6], GREEN, use_color)
        cells[7] = color(cells[7], CYAN, use_color)
        cells[8] = color(cells[8], YELLOW, use_color)
        cells[9] = color(cells[9], RED, use_color)
        cells[10] = color(cells[10], RED if row["counts"]["missing"] else GREEN, use_color)
        print("  ".join(cells))
    print()


def score_rows(summaries: list[dict]) -> list[dict]:
    rows = []
    models = sorted({summary["model"] for summary in summaries})
    experiments = sorted({
        row["experiment"]
        for summary in summaries
        for row in evaluation_rows_from_summary(summary)
    })
    seen = set()
    for summary in summaries:
        for row in evaluation_rows_from_summary(summary):
            seen.add((row["experiment"], row["model"]))
            rows.append({
                "model": row["model"],
                "run_id": row["run_id"],
                "experiment": row["experiment"],
                "score": row["avg_usefulness"],
                "count": row["count"],
                "counts": row["counts"],
                "status": "evaluated",
            })
    for experiment in experiments:
        for model in models:
            if (experiment, model) in seen:
                continue
            rows.append({
                "model": model,
                "run_id": "",
                "experiment": experiment,
                "score": None,
                "count": 0,
                "counts": {"good": 0, "usable": 0, "weak": 0, "bad": 0, "missing": 0},
                "status": "not run",
            })
    return rows


def score_cells(row: dict) -> list[str]:
    score = "-" if row["score"] is None else f"{row['score']:.1f}"
    counts = row["counts"]
    if row.get("status") == "not run":
        verdict_mix = "not run"
    else:
        verdict_mix = (
            f"G{counts['good']} "
            f"U{counts['usable']} "
            f"W{counts['weak']} "
            f"B{counts['bad']} "
            f"M{counts['missing']}"
        )
    return [row["experiment"], row["model"], score, str(row["count"]), verdict_mix]


def score_sort_key(row: dict, sort_mode: str) -> tuple:
    score = row["score"] or 0
    if sort_mode == "name":
        return (row["experiment"], row.get("status") == "not run", row["model"], row["run_id"])
    return (row["experiment"], row.get("status") == "not run", -score, row["model"], row["run_id"])


def print_score_comparison(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    headers = ["Experiment", "Model", "Score", "N", "Verdicts"]
    rows = sorted(score_rows(summaries), key=lambda row: score_sort_key(row, sort_mode))
    cell_rows = [score_cells(row) for row in rows]
    widths = widths_for_cells(cell_rows, headers)

    print(color("\nComplete Score Comparison", BOLD + CYAN, use_color))
    print(color(
        "Grouped by prompt experiment. Score = average learner_usefulness from 1-5; "
        "Verdicts show Good/Usable/Weak/Bad/Missing counts.\n",
        DIM,
        use_color,
    ))

    header_line = "  ".join(header.ljust(widths[i]) for i, header in enumerate(headers))
    print(color(header_line, BOLD, use_color))
    print(color("-" * len(header_line), DIM, use_color))
    for row, cells in zip(rows, cell_rows, strict=True):
        cells = [clip(cell, widths[i]).ljust(widths[i]) for i, cell in enumerate(cells)]
        score = row["score"] or 0
        if row.get("status") == "not run":
            cells[2] = color(cells[2], DIM, use_color)
            cells[4] = color(cells[4], DIM, use_color)
        else:
            cells[2] = color(cells[2], GREEN if score >= 4 else YELLOW if score >= 3 else RED, use_color)
        print("  ".join(cells))
    print()


def print_experiments(summaries: list[dict], use_color: bool, sort_mode: str) -> None:
    print(color("Experiments", BOLD + MAGENTA, use_color))
    for summary in sorted_summaries(summaries, sort_mode):
        rel_path = summary["path"].relative_to(PROJECT_ROOT)
        line = (
            f"- {summary['model']}  "
            f"{color(summary['experiment_id'], BLUE, use_color)}  "
            f"{summary['match_count']}/{summary['count']} exact  "
            f"{fmt_seconds(summary['wall_seconds'])}  "
            f"{rel_path}"
        )
        print(line)
    print()


def report_label(summaries: list[dict]) -> str:
    for summary in summaries:
        match = re.search(r"batch\d+_\d+sent", summary["experiment_id"])
        if match:
            return match.group(0)
    return "selected runs"


def verdict_mix(counts: dict[str, int], include_missing: bool = False) -> str:
    parts = []
    for label, key in (("G", "good"), ("U", "usable"), ("W", "weak"), ("B", "bad")):
        value = counts.get(key, 0)
        if value:
            parts.append(f"{label}:{value}")
    if include_missing and counts.get("missing", 0):
        parts.append(f"M:{counts['missing']}")
    return " ".join(parts) if parts else "no verdicts"


def result_score(result: dict) -> int | None:
    value = result.get("evaluation", {}).get("learner_usefulness")
    return value if isinstance(value, int) else None


def per_experiment_rows(summaries: list[dict]) -> list[dict]:
    rows = []
    for summary in summaries:
        by_test = summary.get("by_test", {})
        for row in evaluation_rows_from_summary(summary):
            test = row["experiment"]
            exact = by_test.get(test, {}).get("normalized_match_count", 0)
            timing = by_test.get(test, {}).get("avg_model_call_seconds", 0)
            timeout = by_test.get(test, {}).get("timed_out_count", 0)
            rows.append({
                **row,
                "exact_count": exact,
                "time_seconds": timing,
                "timed_out_count": timeout,
            })
    return rows


def experiment_note(test_name: str, rows: list[dict]) -> str:
    evaluated = [row for row in rows if row["avg_usefulness"] is not None]
    if not evaluated:
        return "No evaluator verdicts recorded yet."

    best = evaluated[0]
    weak_models = [
        fmt_model(row["model"])
        for row in evaluated
        if row["counts"].get("weak", 0) or row["counts"].get("bad", 0)
    ]
    fastest = min(evaluated, key=lambda row: row["time_seconds"])
    exact_best = max(evaluated, key=lambda row: row["exact_count"])

    if test_name == "source_row_issue_detection":
        return (
            f"{fmt_model(best['model'])} leads on source-row QA. "
            f"Exact means the model matched the expected has_issue flag; inspect comments for missed issue types."
        )
    if test_name.startswith("source_row"):
        return (
            f"{fmt_model(best['model'])} leads on quality. "
            f"{fmt_model(exact_best['model'])} leads exact matching; "
            f"{fmt_model(fastest['model'])} is fastest."
        )
    if test_name == "register_detection":
        return (
            f"{fmt_model(best['model'])} leads on register quality. "
            f"Exact labels are useful here, but evaluator score catches rationale quality."
        )
    if weak_models:
        return (
            f"{fmt_model(best['model'])} leads on learner quality. "
            f"Weak/bad verdicts appear for {', '.join(weak_models)}; inspect comments for drift."
        )
    return (
        f"{fmt_model(best['model'])} leads on learner quality. "
        f"All compared models stayed usable or better."
    )


def print_box(lines: list[str], use_color: bool, width: int = 105) -> None:
    inner = width - 4
    print(f"  ┌{'─' * (width - 2)}┐")
    for line in lines:
        wrapped = textwrap.wrap(line, width=inner) if line else [""]
        for text in wrapped:
            print(f"  │ {text.ljust(inner)} │")
    print(f"  └{'─' * (width - 2)}┘")


def print_pretty_model_summary(summaries: list[dict], use_color: bool) -> None:
    print(color("MODEL SUMMARY", BOLD, use_color))
    print("─" * 65)
    rows = sorted(model_summary_rows(summaries), key=lambda row: model_summary_sort_key(row, "rank"))
    for row in rows:
        score = row["avg_usefulness"]
        score_text = "-" if score is None else f"{score:.1f}"
        exact = f"{row['match_count']}/{row['count']} exact"
        timing = f"{fmt_seconds(row['wall_seconds'])}"
        timeouts = f" TO:{row['timed_out_count']}" if row["timed_out_count"] else ""
        mix = verdict_mix(row["counts"], include_missing=True)
        line = (
            f"  {fmt_model(row['model']).ljust(17)} "
            f"{score_text.rjust(3)}  "
            f"{score_bar(score)}  "
            f"{exact.ljust(11)}  "
            f"{timing.rjust(6)}{timeouts}  "
            f"{mix}"
        )
        score_code = GREEN if (score or 0) >= 4 else YELLOW if (score or 0) >= 3 else RED
        print(color(line, score_code, use_color))
    print()
    print("  G = good  U = usable  W = weak  B = bad  TO = timeouts")
    print()


def print_pretty_breakdown(summaries: list[dict], use_color: bool) -> None:
    print(color("PER-EXPERIMENT BREAKDOWN", BOLD, use_color))
    print("─" * 107)
    rows = per_experiment_rows(summaries)
    by_experiment: dict[str, list[dict]] = {}
    for row in rows:
        by_experiment.setdefault(row["experiment"], []).append(row)

    for test_name in sorted(by_experiment):
        ranked = sorted(
            by_experiment[test_name],
            key=lambda row: (
                -(row["avg_usefulness"] or 0),
                -row["exact_count"],
                row["time_seconds"],
                fmt_model(row["model"]),
            ),
        )
        print()
        print(color(f"  {fmt_experiment(test_name)}", BOLD + BLUE, use_color))
        lines = []
        previous_score = None
        previous_rank = 0
        for index, row in enumerate(ranked, start=1):
            score = row["avg_usefulness"]
            rank = previous_rank if score == previous_score else index
            previous_rank = rank
            previous_score = score
            score_text = "-" if score is None else f"{score:.1f}"
            exact = f"{row['exact_count']}/{row['count']} exact"
            timing = fmt_seconds(row["time_seconds"])
            mix = verdict_mix(row["counts"], include_missing=True)
            lines.append(
                f"#{rank:<2} {fmt_model(row['model']).ljust(17)} "
                f"{score_text.rjust(3)}  {score_bar(score)}  "
                f"{exact.ljust(10)}  {timing.rjust(6)}  {mix}"
            )
        lines.append("")
        lines.append(experiment_note(test_name, ranked))
        print_box(lines, use_color)
    print()


def compact_issue_lines(summary: dict) -> list[str]:
    all_results = summary["results"]
    counts = evaluation_counts(all_results)
    lines = [
        f"{fmt_model(summary['model'])} — "
        f"{counts['usable']} usable, {counts['weak']} weak, {counts['bad']} bad"
    ]

    grouped: dict[str, list[dict]] = {}
    for result in all_results:
        verdict = result.get("evaluation", {}).get("verdict")
        if verdict in {"usable", "weak", "bad"}:
            grouped.setdefault(result.get("test_name", "unknown"), []).append(result)

    if not grouped:
        lines.append("· No recurring evaluator issues in this run.")
        return lines

    for test_name, results in sorted(
        grouped.items(),
        key=lambda item: (
            -sum(1 for result in item[1] if result.get("evaluation", {}).get("verdict") == "bad"),
            -len(item[1]),
            item[0],
        ),
    )[:4]:
        verdicts = evaluation_counts(results)
        sample = results[0].get("evaluation", {})
        issue = "; ".join(sample.get("issues", [])[:2])
        if not issue:
            issue = sample.get("comment", "Needs review.")
        lines.append(
            f"· {fmt_experiment(test_name)} — {verdict_mix(verdicts)}. {issue}"
        )
    return lines


def print_pretty_issues(summaries: list[dict], use_color: bool) -> None:
    print(color("ISSUES BY MODEL", BOLD, use_color))
    print("─" * 107)
    ordered = sorted(summaries, key=lambda summary: model_summary_sort_key(model_summary_rows([summary])[0], "rank"))
    for index, summary in enumerate(ordered):
        for line in compact_issue_lines(summary):
            print(f"  {line}")
        if index != len(ordered) - 1:
            print("  " + "·" * 100)
    print()


def print_pretty_report(summaries: list[dict], use_color: bool, include_comments: bool, sort_mode: str) -> None:
    title = f"Translation Report — {report_label(summaries)}"
    print(color(title, BOLD + CYAN, use_color))
    print("═" * 65)
    print()
    print_pretty_model_summary(summaries, use_color)
    print_pretty_breakdown(summaries, use_color)
    print_pretty_issues(summaries, use_color)
    print("─" * 107)
    print("  run with `comments` or `--comments` for full per-sentence evaluator notes")
    print("═" * 65)
    if include_comments:
        print_evaluation_comments(summaries, use_color, sort_mode)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "mode",
        nargs="?",
        choices=["compare", "evaluations", "comments", "all"],
        default="all",
        help="Report view to print. Defaults to all.",
    )
    parser.add_argument("paths", nargs="*", help="Summary files or experiment result directories.")
    parser.add_argument("--sort", choices=["rank", "name"], default="rank")
    parser.add_argument(
        "--all-runs",
        action="store_true",
        help="With no paths, include every saved summary instead of latest evaluated run per model.",
    )
    parser.add_argument("--glossary", action="store_true", help="Print evaluation terminology.")
    parser.add_argument("--comments", action="store_true", help="Include evaluator comments in all mode.")
    parser.add_argument("--no-color", action="store_true")
    modes = {"compare", "evaluations", "comments", "all"}
    if len(sys.argv) > 1 and sys.argv[1] not in modes and not sys.argv[1].startswith("-"):
        return parser.parse_args(["all", *sys.argv[1:]])
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    summaries = [load_summary(path) for path in summary_files(args.paths, include_all_runs=args.all_runs)]
    use_color = not args.no_color
    if args.mode == "all":
        print_pretty_report(summaries, use_color, args.comments, args.sort)
        return
    if args.mode in {"compare", "all"}:
        print_experiments(summaries, use_color, args.sort)
        print_table(summaries, use_color, args.sort)
    if args.mode in {"evaluations", "comments", "all"} or args.glossary:
        print_evaluation_glossary(use_color)
    if args.mode in {"evaluations", "all"}:
        print_evaluations(summaries, use_color, args.sort)
    if args.mode == "comments" or (args.mode == "all" and args.comments):
        print_evaluation_comments(summaries, use_color, args.sort)


if __name__ == "__main__":
    main()
