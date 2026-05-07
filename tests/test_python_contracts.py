from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

PROJECT_ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(PROJECT_ROOT))

import batch_planner
from schema_validator import ValidationError, validate_and_fix


def write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def minimal_sentence(audio: str | None = None) -> dict:
    sentence = {
        "hindi": "क्या?",
        "romanisation": "kyā?",
        "english": "What?",
        "literal": "what",
        "register": "standard",
        "tokens": [
            {"hindi": "क्या", "roman": "kyā", "kind": "word", "word_index": 0},
            {"hindi": "?", "roman": "?", "kind": "punct"},
        ],
        "words": [{"hindi": "क्या", "roman": "kyā", "meaning": "what"}],
        "anki_tags": ["test", "contract", "sentence"],
    }
    if audio is not None:
        sentence["audio"] = audio
    return sentence


def minimal_word() -> dict:
    return {
        "hindi": "घर",
        "romanisation": "ghar",
        "english": "house",
        "pos": "noun",
        "anki_tags": ["test", "contract"],
        "syllables": "ghar",
        "related_words": [{"hindi": "कमरा", "roman": "kamrā", "english": "room"}],
        "example_sentence": {
            "hindi": "यह घर है।",
            "roman": "yah ghar hai.",
            "english": "This is a house.",
            "breakdown": [
                {"hindi": "यह", "roman": "yah", "meaning": "this"},
                {"hindi": "घर", "roman": "ghar", "meaning": "house"},
                {"hindi": "है", "roman": "hai", "meaning": "is"},
            ],
        },
        "forms": [
            {"label": "base", "hindi": "घर", "roman": "ghar"},
            {"label": "plural", "hindi": "घरें", "roman": "gharẽ"},
        ],
    }


class PythonContractTests(unittest.TestCase):
    def test_metadata_parsing_uses_headings_and_filename_fallback(self) -> None:
        with TemporaryDirectory() as tmp:
            csv_path = Path(tmp) / "complete_hindi_chapter_09_sentences.csv"
            write(
                csv_path,
                "# Complete Hindi\n## Chapter 09\nक्या? (kyā?);What?\n",
            )
            metadata, lines = batch_planner.parse_csv_metadata(csv_path)

            self.assertEqual(metadata.title, "Complete Hindi")
            self.assertEqual(metadata.subtitle, "Chapter 09")
            self.assertEqual(metadata.display_label, "Complete Hindi Chapter 09")
            self.assertEqual(lines, ["क्या? (kyā?);What?"])
            self.assertTrue(
                batch_planner.build_batch_csv_from_metadata(metadata, lines).startswith(
                    "# Complete Hindi\n## Chapter 09\n"
                )
            )

            fallback_path = Path(tmp) / "complete_hindi_chapter_10_sentences.csv"
            write(fallback_path, "कौन? (kaun?);Who?\n")
            fallback, fallback_lines = batch_planner.parse_csv_metadata(fallback_path)

            self.assertEqual(fallback.display_label, "Complete Hindi Chapter 10")
            self.assertEqual(fallback.title, "Complete Hindi")
            self.assertEqual(fallback.subtitle, "Chapter 10")
            self.assertEqual(fallback_lines, ["कौन? (kaun?);Who?"])

    def test_pending_planning_skips_existing_items_and_rejects_gaps(self) -> None:
        with TemporaryDirectory() as tmp:
            root = Path(tmp)
            prompt = root / "generation_prompt_sentences.txt"
            input_dir = root / "input" / "sentences"
            output_dir = root / "output" / "sentences"
            write(prompt, "prompt")
            write(
                input_dir / "sample_sentences.csv",
                "# Complete Hindi\n## Chapter 01\n"
                "क्या? (kyā?);What?\n"
                "कौन? (kaun?);Who?\n"
                "कहाँ? (kahā̃?);Where?\n",
            )
            write(
                output_dir / "sample_sentences_batch_01.json",
                json.dumps(
                    {
                        "title": "Complete Hindi",
                        "subtitle": "Chapter 01",
                        "sentences": [minimal_sentence()],
                    },
                    ensure_ascii=False,
                ),
            )

            old_pipeline = batch_planner.PIPELINES["sentences"]
            batch_planner.PIPELINES["sentences"] = {
                "prompt": prompt,
                "input": input_dir,
                "output": output_dir,
            }
            try:
                pending = batch_planner.pending_batches_for(
                    "sentences", batch_size=2, force=False
                )
                self.assertEqual(len(pending), 1)
                self.assertEqual(pending[0]["batch_num"], 2)
                self.assertEqual(pending[0]["total_batches"], 2)
                self.assertEqual(pending[0]["count"], 2)
                self.assertEqual(pending[0]["title"], "Complete Hindi")
                self.assertEqual(pending[0]["subtitle"], "Chapter 01")

                write(output_dir / "sample_sentences_batch_03.json", "{}")
                with self.assertRaisesRegex(ValueError, "not contiguous"):
                    batch_planner.load_existing_output_state(
                        "sentences", "sample_sentences"
                    )
            finally:
                batch_planner.PIPELINES["sentences"] = old_pipeline

    def test_schema_validation_rejects_unsafe_audio_and_fixes_word_forms(self) -> None:
        validate_and_fix(
            "sentences",
            {
                "title": "Complete Hindi",
                "subtitle": "Chapter 01",
                "sentences": [minimal_sentence("audio/sentences/sample/01_kyā.mp3")],
            },
        )

        with self.assertRaisesRegex(ValidationError, "must start with audio/"):
            validate_and_fix(
                "sentences",
                {
                    "title": "Complete Hindi",
                    "subtitle": "Chapter 01",
                    "sentences": [minimal_sentence("../bad.mp3")],
                },
            )

        word_batch = validate_and_fix(
            "words",
            {
                "title": "Complete Hindi",
                "subtitle": "Chapter 01",
                "words": [minimal_word()],
            },
        )
        self.assertEqual(
            word_batch["words"][0]["forms"],
            [{"label": "plural", "hindi": "घरें", "roman": "gharẽ"}],
        )


if __name__ == "__main__":
    unittest.main()
