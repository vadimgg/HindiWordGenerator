"""Word test: generate optional etymology fields conservatively."""

PROMPT = """Generate conservative etymology information for one Hindi vocabulary item.

Return raw JSON only:
{
  "etymology_journey": [
    {"stage": "...", "form": "...", "roman": "...", "meaning": "..."}
  ],
  "origin_note": "..."
}

Rules:
- Include only attested stages you are confident about.
- Omit etymology_journey when you are not confident.
- Omit origin_note when origin is uncertain.
- Every non-Latin form must include roman.
- Do not guess or invent missing stages.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_etymology",
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
