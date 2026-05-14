"""Word test: decide usage notes and Delhi usage note."""

PROMPT = """Generate practical usage notes for one Hindi vocabulary item.

Return raw JSON only:
{
  "usage_notes": "...",
  "delhi_note": "..."
}

Rules:
- Omit usage_notes if there is no essential learner warning.
- Omit delhi_note if the word sounds natural in everyday Delhi Hindi as-is.
- Include delhi_note when Delhi speakers often prefer English, Hinglish, Urdu/Persian alternative, or another everyday form.
- Keep notes concrete and practical, not linguistic jargon.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_delhi_usage",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
                "romanisation": item["romanisation"],
                "english": item["english"],
            },
        }
        for item in items
    ]
