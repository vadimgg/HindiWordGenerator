"""Hindi-only prompt that asks for word-by-word translation plus final output."""

PROMPT = """Convert exactly one Hindi sentence into English, romanisation, and word-by-word meaning.

Return raw JSON only:
{
  "english":"...",
  "romanisation":"...",
  "word_by_word":[
    {"hindi":"...", "roman":"...", "english":"..."}
  ]
}

Rules:
- Include only actual words in word_by_word; omit punctuation and spaces.
- Translate every word into English in word_by_word; do not put romanisation in the english field.
- Preserve names, numbers, question shape, and tone.
- Preserve "ji" as "ji" in English when it appears in Hindi.
- The final English must be faithful to the word meanings.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "hindi_word_breakdown_translation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
            },
        }
        for item in items
    ]
