#!/usr/bin/env python3
"""
File management utility for the Hindi word generator pipeline.

The main runner orchestrates generation; this script owns batch planning,
schema validation, output writing, and manifest updates.

Commands
--------
check
    Scan input/words/ and input/sentences/ and print a JSON array of
    batches that still need processing.

write <type> <stem> <batch_num> <total_batches> <expected_count> <json_file>
    Validate a generated batch JSON file, write it to the output directory,
    and auto-run mark-done when all batches for the stem are present.
    type is "words" or "sentences".

mark-done <type> <stem> <batches> <count>
    Record a completed file in the manifest.
    type is "words" or "sentences".

Examples
--------
    uv run process.py check
    uv run process.py check --force
    uv run process.py write words hindi_01 3 7 10 /tmp/batch.json
    uv run process.py mark-done words hindi_01 3 28
    uv run process.py mark-done sentences hindi_01 2 15
"""
# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///

import argparse
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent
MANIFEST     = PROJECT_ROOT / "manifest.json"

PIPELINES = {
    "words": {
        "prompt":  PROJECT_ROOT / "generation_prompt_words.txt",
        "input":   PROJECT_ROOT / "input"  / "words",
        "output":  PROJECT_ROOT / "output" / "words",
    },
    "sentences": {
        "prompt":  PROJECT_ROOT / "generation_prompt_sentences.txt",
        "input":   PROJECT_ROOT / "input"  / "sentences",
        "output":  PROJECT_ROOT / "output" / "sentences",
    },
}

DEFAULT_BATCH_SIZE = 10
BATCH_RE = re.compile(r"^(?P<stem>.+)_batch_(?P<num>\d+)\.json$")


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_manifest() -> dict:
    if not MANIFEST.exists():
        return {"words": {}, "sentences": {}}
    data = json.loads(MANIFEST.read_text())
    # Migrate flat manifest from earlier version
    if "words" not in data and "sentences" not in data:
        return {"words": data, "sentences": {}}
    return data


def save_manifest(manifest: dict):
    MANIFEST.write_text(json.dumps(manifest, indent=2, ensure_ascii=False))


def load_json_file(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def has_top_chapter_heading(path: Path) -> bool:
    """Return True when the first non-empty line is a non-empty chapter/topic heading."""
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        return line.startswith("#") and bool(line.lstrip("#").strip())
    return False


def parse_csv(path: Path) -> tuple[str | None, list[str]]:
    """Return (chapter_title_or_None, [content_lines])."""
    chapter = None
    lines = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line:
            continue
        if line.startswith("#"):
            chapter = line.lstrip("#").strip()
        else:
            lines.append(line)
    return chapter or chapter_from_stem(path.stem), lines


def chapter_from_stem(stem: str) -> str:
    """Build a readable chapter title from an input filename."""
    name = stem
    for suffix in ("_words", "_word", "_sentences", "_sentence"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
            break
    name = name.replace("_", " ").replace("-", " ").strip()
    parts = [part.capitalize() if not part.isupper() else part for part in name.split()]
    return " ".join(parts)


def make_batches(items: list[str], size: int) -> list[list[str]]:
    return [items[i : i + size] for i in range(0, len(items), size)]


def build_batch_csv(chapter: str | None, items: list[str]) -> str:
    lines = ([f"# {chapter}"] if chapter else []) + items
    return "\n".join(lines)


def batch_file_path(pipeline_type: str, stem: str, batch_num: int) -> Path:
    return PIPELINES[pipeline_type]["output"] / f"{stem}_batch_{batch_num:02d}.json"


def iter_batch_paths(pipeline_type: str, stem: str):
    output_dir = PIPELINES[pipeline_type]["output"]
    for path in sorted(output_dir.glob(f"{stem}_batch_*.json")):
        match = BATCH_RE.match(path.name)
        if match and match.group("stem") == stem:
            yield path, int(match.group("num"))


def parse_input_item(line: str) -> tuple[str, str, str]:
    """Parse one input CSV content line into a comparable identity tuple."""
    if ";" not in line:
        return line.strip(), "", ""
    left, english = line.split(";", 1)
    left = left.strip()
    english = english.strip()

    match = re.match(r"^(.*?)\s*\((.*?)\)\s*$", left)
    if not match:
        return left, "", english
    hindi = match.group(1).strip()
    roman = match.group(2).strip()
    return hindi, roman, english


def load_existing_entries(pipeline_type: str, stem: str) -> list[dict]:
    key = "words" if pipeline_type == "words" else "sentences"
    entries: list[dict] = []
    for path, _ in iter_batch_paths(pipeline_type, stem):
        try:
            data = load_json_file(path)
        except json.JSONDecodeError:
            continue
        entries.extend(data.get(key, []))
    return entries


def load_existing_output_state(pipeline_type: str, stem: str) -> tuple[set[tuple[str, str, str]], int]:
    """
    Return (processed_identities, max_batch_num) for existing output batches.

    Identity is compared using (hindi, romanisation/roman, english).
    """
    identities: set[tuple[str, str, str]] = set()
    batch_nums: list[int] = []

    for path, batch_num in iter_batch_paths(pipeline_type, stem):
        batch_nums.append(batch_num)
        try:
            data = load_json_file(path)
        except json.JSONDecodeError:
            continue
        key = "words" if pipeline_type == "words" else "sentences"
        for item in data.get(key, []):
            identities.add((
                str(item.get("hindi", "")).strip(),
                str(item.get("romanisation", "") or item.get("roman", "")).strip(),
                str(item.get("english", "")).strip(),
            ))

    if batch_nums:
        expected = list(range(1, max(batch_nums) + 1))
        if sorted(batch_nums) != expected:
            raise ValueError(
                f"Output batches for {pipeline_type}/{stem} are not contiguous. "
                f"Found {sorted(batch_nums)}, expected {expected}."
            )

    return identities, (max(batch_nums) if batch_nums else 0)


def pending_batches_for(pipeline_type: str, batch_size: int, force: bool) -> list[dict]:
    cfg         = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir   = cfg["input"]

    if not prompt_file.exists():
        print(f"Warning: prompt file not found: {prompt_file}", file=sys.stderr)
        return []

    csv_files   = sorted(input_dir.glob("*.csv"))
    pending     = []

    for csv_path in csv_files:
        stem     = csv_path.stem
        chapter, items = parse_csv(csv_path)
        if not items:
            continue

        has_heading = has_top_chapter_heading(csv_path)

        try:
            existing_identities, max_batch_num = load_existing_output_state(pipeline_type, stem)
        except ValueError as exc:
            print(f"Error: {exc}", file=sys.stderr)
            sys.exit(1)

        pending_items = items if force else [
            line for line in items
            if parse_input_item(line) not in existing_identities
        ]
        if not pending_items:
            continue

        batches = make_batches(pending_items, batch_size)
        total_batches = max_batch_num + len(batches)

        for offset, batch in enumerate(batches, 1):
            pending.append({
                "type":          pipeline_type,
                "stem":          stem,
                "csv_path":      str(csv_path),
                "batch_num":     max_batch_num + offset,
                "total_batches": total_batches,
                "chapter":       chapter,
                "has_chapter_heading": has_heading,
                "csv":           build_batch_csv(chapter, batch),
                "count":         len(batch),
            })

    return pending


# ---------------------------------------------------------------------------
# Schema validation
# ---------------------------------------------------------------------------

class ValidationError(ValueError):
    """Raised when generated batch JSON does not match the expected schema."""


def _fix_word(word: dict) -> dict:
    """Remove forms entries whose Devanagari spelling matches the base word."""
    base = word.get("hindi", "")
    forms = word.get("forms")
    if not forms:
        return word
    fixed = [f for f in forms if f.get("hindi") != base]
    if not fixed:
        del word["forms"]
    elif len(fixed) < len(forms):
        word["forms"] = fixed
    return word


def validate_and_fix(pipeline_type: str, data: dict) -> dict:
    """Validate the batch schema, then apply safe normalisations."""
    _validate_batch(pipeline_type, data)
    if pipeline_type == "words":
        data["words"] = [_fix_word(w) for w in data.get("words", [])]
    return data


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise ValidationError(message)


def _require_type(value, expected_type, path: str) -> None:
    _require(isinstance(value, expected_type), f"{path} must be {expected_type.__name__}")


def _require_non_empty_string(value, path: str) -> None:
    _require_type(value, str, path)
    _require(bool(value.strip()), f"{path} must be a non-empty string")


def _require_list(value, path: str) -> None:
    _require_type(value, list, path)


def _require_no_nulls(obj, path: str = "$") -> None:
    if obj is None and path != "$.chapter":
        raise ValidationError(f"{path} must not be null")
    if isinstance(obj, dict):
        for key, value in obj.items():
            _require_no_nulls(value, f"{path}.{key}")
    elif isinstance(obj, list):
        for index, value in enumerate(obj):
            _require_no_nulls(value, f"{path}[{index}]")


def _require_no_date_added(obj, path: str = "$") -> None:
    if isinstance(obj, dict):
        _require("date_added" not in obj, f"{path}.date_added is not allowed")
        for key, value in obj.items():
            _require_no_date_added(value, f"{path}.{key}")
    elif isinstance(obj, list):
        for index, value in enumerate(obj):
            _require_no_date_added(value, f"{path}[{index}]")


def _require_no_empty_optionals(obj: dict, optional_keys: set[str], path: str) -> None:
    for key in optional_keys.intersection(obj.keys()):
        value = obj[key]
        if isinstance(value, (list, dict, str)):
            _require(bool(value), f"{path}.{key} must be omitted instead of empty")


def _validate_tag_list(tags: list, path: str, minimum: int) -> None:
    _require_list(tags, path)
    _require(len(tags) >= minimum, f"{path} must contain at least {minimum} item(s)")
    for index, tag in enumerate(tags):
        _require_non_empty_string(tag, f"{path}[{index}]")


def _validate_words_batch(data: dict) -> None:
    allowed_top = {"chapter", "words"}
    _require(set(data.keys()).issubset(allowed_top), "$ contains unexpected top-level keys")
    _require("words" in data, "$.words is required")
    _require("chapter" in data, "$.chapter is required")
    _require(data["chapter"] is None or isinstance(data["chapter"], str), "$.chapter must be a string or null")
    _require_list(data["words"], "$.words")

    required = {
        "hindi", "romanisation", "english", "pos", "anki_tags",
        "syllables", "related_words", "example_sentence",
    }
    optional = {
        "gender", "transitivity", "forms", "morphemes", "usage_notes",
        "delhi_note", "sound_alikes", "etymology_journey", "origin_note", "audio",
    }
    allowed = required | optional

    for index, word in enumerate(data["words"]):
        path = f"$.words[{index}]"
        _require_type(word, dict, path)
        missing = required - word.keys()
        _require(not missing, f"{path} is missing required keys: {', '.join(sorted(missing))}")
        extra = set(word.keys()) - allowed
        _require(not extra, f"{path} has unexpected keys: {', '.join(sorted(extra))}")
        _require_no_empty_optionals(word, optional, path)
        _require_non_empty_string(word["hindi"], f"{path}.hindi")
        _require_non_empty_string(word["romanisation"], f"{path}.romanisation")
        _require_non_empty_string(word["english"], f"{path}.english")
        _require_non_empty_string(word["pos"], f"{path}.pos")
        _require_non_empty_string(word["syllables"], f"{path}.syllables")
        _validate_tag_list(word["anki_tags"], f"{path}.anki_tags", 2)
        _require_list(word["related_words"], f"{path}.related_words")
        _require(len(word["related_words"]) > 0, f"{path}.related_words must not be empty")

        for rel_index, rel in enumerate(word["related_words"]):
            rel_path = f"{path}.related_words[{rel_index}]"
            _require_type(rel, dict, rel_path)
            for key in ("hindi", "roman", "english"):
                _require_non_empty_string(rel.get(key), f"{rel_path}.{key}")

        example = word["example_sentence"]
        ex_path = f"{path}.example_sentence"
        _require_type(example, dict, ex_path)
        _require(set(example.keys()) == {"hindi", "roman", "english", "breakdown"}, f"{ex_path} must contain exactly hindi, roman, english, breakdown")
        for key in ("hindi", "roman", "english"):
            _require_non_empty_string(example[key], f"{ex_path}.{key}")
        _require_list(example["breakdown"], f"{ex_path}.breakdown")
        _require(len(example["breakdown"]) > 0, f"{ex_path}.breakdown must not be empty")
        for tok_index, token in enumerate(example["breakdown"]):
            tok_path = f"{ex_path}.breakdown[{tok_index}]"
            _require_type(token, dict, tok_path)
            _require(set(token.keys()) == {"hindi", "roman", "meaning"}, f"{tok_path} must contain exactly hindi, roman, meaning")
            for key in ("hindi", "roman", "meaning"):
                _require_non_empty_string(token[key], f"{tok_path}.{key}")

        if "forms" in word:
            _require_list(word["forms"], f"{path}.forms")
            for form_index, form in enumerate(word["forms"]):
                form_path = f"{path}.forms[{form_index}]"
                _require_type(form, dict, form_path)
                _require(set(form.keys()) == {"label", "hindi", "roman"}, f"{form_path} must contain exactly label, hindi, roman")
                for key in ("label", "hindi", "roman"):
                    _require_non_empty_string(form[key], f"{form_path}.{key}")

        if "morphemes" in word:
            _require_list(word["morphemes"], f"{path}.morphemes")
            for morph_index, morph in enumerate(word["morphemes"]):
                morph_path = f"{path}.morphemes[{morph_index}]"
                _require_type(morph, dict, morph_path)
                _require(set(morph.keys()) == {"part", "roman", "meaning", "origin"}, f"{morph_path} must contain exactly part, roman, meaning, origin")
                for key in ("part", "roman", "meaning", "origin"):
                    _require_non_empty_string(morph[key], f"{morph_path}.{key}")

        if "sound_alikes" in word:
            _require_list(word["sound_alikes"], f"{path}.sound_alikes")
            for sound_index, sound in enumerate(word["sound_alikes"]):
                sound_path = f"{path}.sound_alikes[{sound_index}]"
                _require_type(sound, dict, sound_path)
                _require(set(sound.keys()) == {"part", "association", "roman", "language", "note"}, f"{sound_path} must contain exactly part, association, roman, language, note")
                for key in ("part", "association", "roman", "language", "note"):
                    _require_non_empty_string(sound[key], f"{sound_path}.{key}")

        if "etymology_journey" in word:
            _require_list(word["etymology_journey"], f"{path}.etymology_journey")
            for stage_index, stage in enumerate(word["etymology_journey"]):
                stage_path = f"{path}.etymology_journey[{stage_index}]"
                _require_type(stage, dict, stage_path)
                _require(set(stage.keys()) == {"stage", "form", "roman", "meaning"}, f"{stage_path} must contain exactly stage, form, roman, meaning")
                for key in ("stage", "form", "roman", "meaning"):
                    _require_non_empty_string(stage[key], f"{stage_path}.{key}")

        for key in ("usage_notes", "delhi_note", "origin_note", "gender", "transitivity", "audio"):
            if key in word:
                _require_non_empty_string(word[key], f"{path}.{key}")


def _validate_sentences_batch(data: dict) -> None:
    allowed_top = {"chapter", "sentences"}
    _require(set(data.keys()).issubset(allowed_top), "$ contains unexpected top-level keys")
    _require("sentences" in data, "$.sentences is required")
    _require("chapter" in data, "$.chapter is required")
    _require(data["chapter"] is None or isinstance(data["chapter"], str), "$.chapter must be a string or null")
    _require_list(data["sentences"], "$.sentences")

    required = {
        "hindi",
        "romanisation",
        "english",
        "literal",
        "register",
        "tokens",
        "words",
        "anki_tags",
    }
    allowed_sentence_keys = required | {"audio"}
    allowed_word_keys = {"hindi", "roman", "meaning", "gender", "number", "note"}
    allowed_token_keys = {"hindi", "roman", "kind", "word_index"}

    for index, sentence in enumerate(data["sentences"]):
        path = f"$.sentences[{index}]"
        _require_type(sentence, dict, path)
        missing = required - sentence.keys()
        _require(not missing, f"{path} is missing required keys: {', '.join(sorted(missing))}")
        extra = set(sentence.keys()) - allowed_sentence_keys
        _require(not extra, f"{path} has unexpected keys: {', '.join(sorted(extra))}")
        for key in ("hindi", "romanisation", "english", "literal", "register"):
            _require_non_empty_string(sentence[key], f"{path}.{key}")
        if "audio" in sentence:
            _require_non_empty_string(sentence["audio"], f"{path}.audio")
        _validate_tag_list(sentence["anki_tags"], f"{path}.anki_tags", 3)
        _require_list(sentence["tokens"], f"{path}.tokens")
        _require(len(sentence["tokens"]) > 0, f"{path}.tokens must not be empty")
        _require_list(sentence["words"], f"{path}.words")
        _require(len(sentence["words"]) > 0, f"{path}.words must not be empty")

        reconstructed_hindi: list[str] = []
        reconstructed_roman: list[str] = []
        used_word_indexes: set[int] = set()

        for token_index, token in enumerate(sentence["tokens"]):
            token_path = f"{path}.tokens[{token_index}]"
            _require_type(token, dict, token_path)
            _require(set(token.keys()).issubset(allowed_token_keys), f"{token_path} has unexpected keys")
            _require({"hindi", "roman", "kind"}.issubset(token.keys()), f"{token_path} is missing required keys")
            _require_type(token["hindi"], str, f"{token_path}.hindi")
            _require_type(token["roman"], str, f"{token_path}.roman")
            _require(token["hindi"] != "", f"{token_path}.hindi must not be empty")
            _require(token["roman"] != "", f"{token_path}.roman must not be empty")
            _require(token["kind"] in {"word", "punct", "space"}, f"{token_path}.kind must be 'word', 'punct', or 'space'")

            reconstructed_hindi.append(token["hindi"])
            reconstructed_roman.append(token["roman"])

            if token["kind"] == "word":
                _require("word_index" in token, f"{token_path}.word_index is required for word tokens")
                _require_type(token["word_index"], int, f"{token_path}.word_index")
                _require(0 <= token["word_index"] < len(sentence["words"]), f"{token_path}.word_index is out of range")
                linked_word = sentence["words"][token["word_index"]]
                _require(
                    token["hindi"] == linked_word["hindi"],
                    f"{token_path}.hindi must exactly match words[{token['word_index']}].hindi",
                )
                _require(
                    token["roman"] == linked_word["roman"],
                    f"{token_path}.roman must exactly match words[{token['word_index']}].roman",
                )
                used_word_indexes.add(token["word_index"])
            elif token["kind"] == "space":
                _require(token["hindi"].isspace(), f"{token_path}.hindi must contain only whitespace for space tokens")
                _require(token["roman"].isspace(), f"{token_path}.roman must contain only whitespace for space tokens")
                _require("word_index" not in token, f"{token_path}.word_index must be omitted for space tokens")
            else:
                _require("word_index" not in token, f"{token_path}.word_index must be omitted for punct tokens")

        for word_index, word in enumerate(sentence["words"]):
            word_path = f"{path}.words[{word_index}]"
            _require_type(word, dict, word_path)
            _require({"hindi", "roman", "meaning"}.issubset(word.keys()), f"{word_path} is missing required keys")
            extra = set(word.keys()) - allowed_word_keys
            _require(not extra, f"{word_path} has unexpected keys: {', '.join(sorted(extra))}")
            for key in ("hindi", "roman", "meaning"):
                _require_non_empty_string(word[key], f"{word_path}.{key}")
            for key in ("gender", "number", "note"):
                if key in word:
                    _require_non_empty_string(word[key], f"{word_path}.{key}")

        _require(
            used_word_indexes == set(range(len(sentence["words"]))),
            f"{path}.tokens must reference every entry in words exactly at least once",
        )
        _require(
            "".join(reconstructed_hindi) == sentence["hindi"],
            f"{path}.tokens do not reconstruct sentence.hindi exactly",
        )
        _require(
            "".join(reconstructed_roman) == sentence["romanisation"],
            f"{path}.tokens do not reconstruct sentence.romanisation exactly",
        )


def _validate_batch(pipeline_type: str, data: dict) -> None:
    _require_type(data, dict, "$")
    _require_no_nulls(data)
    _require_no_date_added(data)

    if pipeline_type == "words":
        _validate_words_batch(data)
    elif pipeline_type == "sentences":
        _validate_sentences_batch(data)
    else:
        raise ValidationError(f"Unknown pipeline type: {pipeline_type}")



# ---------------------------------------------------------------------------
# Commands
# ---------------------------------------------------------------------------

def cmd_check(batch_size: int, force: bool, pipeline_type: str | None):
    pending  = []

    types = [pipeline_type] if pipeline_type else list(PIPELINES.keys())
    for t in types:
        pending.extend(pending_batches_for(t, batch_size, force))

    print(json.dumps(pending, indent=2, ensure_ascii=False))


def cmd_write(
    pipeline_type: str,
    stem: str,
    batch_num: int,
    total_batches: int,
    expected_count: int,
    json_path: str,
):
    src        = Path(json_path)

    if not src.exists():
        print(f"Error: {src} not found", file=sys.stderr)
        sys.exit(1)

    try:
        data = load_json_file(src)
    except json.JSONDecodeError as e:
        print(f"Error: invalid JSON in {src}: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        data = validate_and_fix(pipeline_type, data)
    except ValidationError as e:
        print(f"Error: schema validation failed for {src}: {e}", file=sys.stderr)
        sys.exit(1)

    key = "words" if pipeline_type == "words" else "sentences"
    count = len(data.get(key, []))
    if count != expected_count:
        print(
            f"Error: expected {expected_count} {key} in this batch, got {count}",
            file=sys.stderr,
        )
        sys.exit(1)

    out_path = batch_file_path(pipeline_type, stem, batch_num)
    out_path.write_text(json.dumps(data, ensure_ascii=False, indent=2))
    print(f"Written: {out_path}")

    # Count items in this batch for reporting
    print(f"  {count} {key} in this batch")

    # Auto mark-done when all batch files are present
    all_present = all(
        batch_file_path(pipeline_type, stem, n).exists()
        for n in range(1, total_batches + 1)
    )
    if all_present:
        total_items = sum(
            len(load_json_file(batch_file_path(pipeline_type, stem, n)).get(key, []))
            for n in range(1, total_batches + 1)
        )
        cmd_mark_done(pipeline_type, stem, total_batches, total_items)
    else:
        present = sum(
            1 for n in range(1, total_batches + 1)
            if batch_file_path(pipeline_type, stem, n).exists()
        )
        print(f"  {present}/{total_batches} batches present — mark-done deferred")


def cmd_mark_done(pipeline_type: str, stem: str, batches: int, count: int):
    cfg         = PIPELINES[pipeline_type]
    prompt_file = cfg["prompt"]
    input_dir   = cfg["input"]
    csv_path    = input_dir / f"{stem}.csv"

    if not csv_path.exists():
        print(f"Error: {csv_path} not found", file=sys.stderr)
        sys.exit(1)

    csv_hash    = sha256(csv_path.read_bytes())
    prompt_hash = sha256(prompt_file.read_bytes())
    manifest    = load_manifest()

    manifest.setdefault(pipeline_type, {})[stem] = {
        "csv_hash":     csv_hash,
        "prompt_hash":  prompt_hash,
        "processed_at": datetime.now(timezone.utc).isoformat(),
        "batches":      batches,
        "count":        count,
    }

    save_manifest(manifest)
    label = "words" if pipeline_type == "words" else "sentences"
    print(f"Manifest updated: {pipeline_type}/{stem} ({count} {label}, {batches} batch(es))")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    parser = argparse.ArgumentParser(
        description="File management utility for the Hindi word generator pipeline.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__,
    )
    sub = parser.add_subparsers(dest="command", required=True)

    p_check = sub.add_parser("check", help="List batches that need processing.")
    p_check.add_argument("--batch-size", type=int, default=DEFAULT_BATCH_SIZE)
    p_check.add_argument("--force", action="store_true",
                         help="Include all files even if unchanged.")
    p_check.add_argument("--type", choices=["words", "sentences"],
                         help="Check only one pipeline type (default: both).")

    p_write = sub.add_parser("write", help="Validate a batch JSON and write it to output.")
    p_write.add_argument("type",         choices=["words", "sentences"])
    p_write.add_argument("stem",         help="CSV filename without extension, e.g. hindi_01")
    p_write.add_argument("batch_num",    type=int, help="Batch number (1-based).")
    p_write.add_argument("total_batches",type=int, help="Total number of batches for this stem.")
    p_write.add_argument("expected_count", type=int, help="Expected number of items in the batch.")
    p_write.add_argument("json_file",    help="Path to the raw JSON file from the agent.")

    p_done = sub.add_parser("mark-done", help="Record a completed file in the manifest.")
    p_done.add_argument("type",    choices=["words", "sentences"])
    p_done.add_argument("stem",    help="CSV filename without extension, e.g. hindi_01")
    p_done.add_argument("batches", type=int, help="Number of batch files written.")
    p_done.add_argument("count",   type=int, help="Total items processed.")

    args = parser.parse_args()

    # Ensure output dirs exist
    for cfg in PIPELINES.values():
        cfg["output"].mkdir(parents=True, exist_ok=True)

    if args.command == "check":
        cmd_check(args.batch_size, args.force, args.type)
    elif args.command == "write":
        cmd_write(
            args.type,
            args.stem,
            args.batch_num,
            args.total_batches,
            args.expected_count,
            args.json_file,
        )
    elif args.command == "mark-done":
        cmd_mark_done(args.type, args.stem, args.batches, args.count)


if __name__ == "__main__":
    main()
