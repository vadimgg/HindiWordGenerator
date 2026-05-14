"""Translation prompt that receives the same shape as sentence input files."""

PROMPT = """Read exactly one sentence source row and return clean English and romanisation.

The source row shape is:
Hindi sentence (romanisation);English reference

Return raw JSON only:
{
  "english":"...",
  "romanisation":"..."
}

Rules:
- Use the Hindi as the source of truth.
- Use the supplied romanisation and English only as reference context.
- Preserve names, numbers, question shape, and tone.
- Preserve "ji" as "ji" when it appears in Hindi.
- Do not add commentary, alternatives, or word-by-word notes.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "source_row_simple_translation_romanisation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "source_row": item["input_line"],
            },
        }
        for item in items
    ]
