"""Simple Hindi-only translation and romanisation prompt."""

PROMPT = """Convert exactly one Hindi sentence into English and romanisation.

Return raw JSON only:
{
  "english":"...",
  "romanisation":"..."
}

Rules:
- Translate the Hindi sentence faithfully.
- Preserve names, numbers, question shape, and tone.
- Preserve "ji" as "ji" when it appears in Hindi.
- Use readable romanisation with diacritics when you are confident.
- Do not include word-by-word notes or commentary.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "hindi_simple_translation_romanisation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
            },
        }
        for item in items
    ]
