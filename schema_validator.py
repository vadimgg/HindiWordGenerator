"""Generated batch schema validation and safe normalisation."""

from __future__ import annotations

import re
from pathlib import PurePosixPath


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


def _require_audio_path(value: str, path: str) -> None:
    _require_non_empty_string(value, path)
    _require(not re.match(r"^[a-z][a-z0-9+.-]*:", value, re.IGNORECASE), f"{path} must be a project-relative audio path")
    clean = value.strip().removeprefix("/")
    posix_path = PurePosixPath(clean)
    _require(clean.startswith("audio/"), f"{path} must start with audio/")
    _require(posix_path.suffix == ".mp3", f"{path} must point to an .mp3 file")
    _require(".." not in posix_path.parts, f"{path} must not contain parent-directory segments")


def _require_list(value, path: str) -> None:
    _require_type(value, list, path)


def _require_no_nulls(obj, path: str = "$") -> None:
    if obj is None and path != "$.subtitle":
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
    allowed_top = {"title", "subtitle", "words"}
    _require(set(data.keys()).issubset(allowed_top), "$ contains unexpected top-level keys")
    _require("words" in data, "$.words is required")
    _require("title" in data, "$.title is required")
    _require("subtitle" in data, "$.subtitle is required")
    _require_non_empty_string(data["title"], "$.title")
    _require(data["subtitle"] is None or isinstance(data["subtitle"], str), "$.subtitle must be a string or null")
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
                if key == "audio":
                    _require_audio_path(word[key], f"{path}.{key}")
                else:
                    _require_non_empty_string(word[key], f"{path}.{key}")


def _validate_sentences_batch(data: dict) -> None:
    allowed_top = {"title", "subtitle", "sentences"}
    _require(set(data.keys()).issubset(allowed_top), "$ contains unexpected top-level keys")
    _require("sentences" in data, "$.sentences is required")
    _require("title" in data, "$.title is required")
    _require("subtitle" in data, "$.subtitle is required")
    _require_non_empty_string(data["title"], "$.title")
    _require(data["subtitle"] is None or isinstance(data["subtitle"], str), "$.subtitle must be a string or null")
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
            _require_audio_path(sentence["audio"], f"{path}.audio")
        _validate_tag_list(sentence["anki_tags"], f"{path}.anki_tags", 3)
        _require_list(sentence["tokens"], f"{path}.tokens")
        _require(len(sentence["tokens"]) > 0, f"{path}.tokens must not be empty")
        _require_list(sentence["words"], f"{path}.words")
        _require(len(sentence["words"]) > 0, f"{path}.words must not be empty")

        for token_index, token in enumerate(sentence["tokens"]):
            token_path = f"{path}.tokens[{token_index}]"
            _require_type(token, dict, token_path)
            _require(set(token.keys()).issubset(allowed_token_keys), f"{token_path} has unexpected keys")
            _require({"hindi", "roman", "kind", "word_index"}.issubset(token.keys()), f"{token_path} is missing required keys")
            _require_type(token["hindi"], str, f"{token_path}.hindi")
            _require_type(token["roman"], str, f"{token_path}.roman")
            _require(token["hindi"] != "", f"{token_path}.hindi must not be empty")
            _require(token["roman"] != "", f"{token_path}.roman must not be empty")
            _require(token["kind"] == "word", f"{token_path}.kind must be 'word'")
            _require_type(token["word_index"], int, f"{token_path}.word_index")
            _require(token["word_index"] == token_index, f"{token_path}.word_index must match token position")
            _require(token_index < len(sentence["words"]), f"{token_path}.word_index is out of range")
            linked_word = sentence["words"][token_index]
            _require(
                token["hindi"] == linked_word["hindi"],
                f"{token_path}.hindi must exactly match words[{token_index}].hindi",
            )
            _require(
                token["roman"] == linked_word["roman"],
                f"{token_path}.roman must exactly match words[{token_index}].roman",
            )

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

        _require(len(sentence["tokens"]) == len(sentence["words"]), f"{path}.tokens must contain one word token for each words entry")


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
