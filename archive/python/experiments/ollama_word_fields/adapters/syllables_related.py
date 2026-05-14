"""Word test: generate syllables and related words."""

PROMPT = """Generate syllables and useful related words for one Hindi vocabulary item.

Return raw JSON only:
{
  "syllables": "...",
  "related_words": [
    {"hindi": "...", "roman": "...", "english": "..."}
  ]
}

Rules:
- syllables must be Roman script only, separated with " · ".
- related_words must be genuinely useful for a beginner/intermediate learner.
- Every related Hindi word must include romanisation.
- Avoid obscure dictionary relatives.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_syllables_related",
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
