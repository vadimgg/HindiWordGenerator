"""Word-by-word prompt that receives the same shape as sentence input files."""

PROMPT = """Read exactly one sentence source row and return clean card translation pieces.

The source row shape is:
Hindi sentence (romanisation);English reference

Return raw JSON only:
{
  "english":"...",
  "romanisation":"...",
  "word_by_word":[
    {"hindi":"...", "roman":"...", "english":"..."}
  ]
}

Rules:
- Use the Hindi as the source of truth.
- Use the supplied romanisation and English only as reference context.
- Include only actual words in word_by_word; omit punctuation and spaces.
- Translate every word into English in word_by_word; do not put romanisation in the english field.
- Preserve names, numbers, question shape, and tone.
- Preserve "ji" as "ji" in English when it appears in Hindi.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "source_row_word_breakdown_translation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "source_row": item["input_line"],
            },
        }
        for item in items
    ]
