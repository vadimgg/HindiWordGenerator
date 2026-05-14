"""Source-row QA prompt for detecting bad sentence inputs before enrichment."""

PROMPT = """Audit exactly one Hindi sentence source row before flashcard generation.

The source row shape is:
Hindi sentence (romanisation);English reference

Return raw JSON only:
{
  "has_issue": true,
  "severity": "none|minor|major",
  "issues": ["..."],
  "corrected_english": "...",
  "corrected_romanisation": "..."
}

Rules:
- Use the Hindi sentence as the source of truth.
- Flag English that is awkward, ungrammatical, duplicated, missing meaning, or meaning-drifting.
- Flag romanisation that does not match the Hindi, including wrong gender agreement or missing words.
- Flag rows that are phrase/noun drills rather than full sentences.
- Do not flag a row just because the English is not word-for-word literal.
- If the row is good, set has_issue to false, severity to "none", and issues to [].
- corrected_english and corrected_romanisation should contain the best corrected values.
- Do not add commentary or alternatives.
"""


def source_row(hindi: str, romanisation: str, english: str) -> str:
    return f"{hindi} ({romanisation});{english}"


def case_item(base_item: dict, source_index: int, row: str, expected_issue: bool, issue_kind: str) -> dict:
    item = dict(base_item)
    item["item_id"] = f"issue_{source_index + 1:03d}"
    item["source_index"] = source_index
    item["input_line"] = row
    item["expected_issue"] = expected_issue
    item["expected_issue_kind"] = issue_kind
    return item


def build_issue_cases(items: list[dict]) -> list[dict]:
    if not items:
        return []

    base = items[0]
    cases = [
        case_item(
            base,
            0,
            source_row(
                "अध्यापक जी, यहाँ कितने विद्यार्थी हैं?",
                "adhyāpak jī, yahā̃ kitne vidyārthī haĩ?",
                "Teacher ji, how many students are there here?",
            ),
            True,
            "awkward_english_duplicate_location",
        ),
        case_item(
            base,
            1,
            source_row(
                "अभी चौदह हैं – नौ लड़कियाँ और पाँच लड़के।",
                "abhī caudah haĩ – nau laṛkiyā̃ aur pā̃c laṛke.",
                "At the moment there are 14 – nine girls and five boys.",
            ),
            False,
            "clean",
        ),
        case_item(
            base,
            2,
            source_row(
                "यह किताब कैसी है?",
                "yah kitāb kaisī hai?",
                "What book is this kind?",
            ),
            True,
            "bad_english_word_order",
        ),
        case_item(
            base,
            3,
            source_row(
                "क्या यह अच्छी है?",
                "kyā yah acchā hai?",
                "Is it good?",
            ),
            True,
            "romanisation_gender_mismatch",
        ),
        case_item(
            base,
            4,
            source_row(
                "हाँ, बुरी नहीं है।",
                "hā̃, burī nahī̃ hai.",
                "Yes, it's not bad.",
            ),
            False,
            "clean",
        ),
    ]
    return cases


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "source_row_issue_detection",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "source_row": item["input_line"],
                "expected_audit_focus": item["expected_issue_kind"],
            },
        }
        for item in build_issue_cases(items)
    ]
