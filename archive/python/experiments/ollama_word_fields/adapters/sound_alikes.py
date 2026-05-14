"""Word test: generate optional multilingual sound-alike mnemonics."""

PROMPT = """Decide whether a Hindi vocabulary item has genuinely useful sound-alike mnemonics.

Return raw JSON only:
{
  "sound_alikes": [
    {
      "part": "...",
      "association": "...",
      "roman": "...",
      "language": "English|Russian|Hebrew",
      "note": "..."
    }
  ],
  "omit_reason": "..."
}

Rules:
- Use only English, Russian, or Hebrew associations.
- Include sound_alikes only when they are genuinely memorable and non-circular.
- Omit sound_alikes when the match is weak; then include a short omit_reason.
- Never use the Hindi word itself or a direct transliteration as the association.
- If association is non-Latin, include roman.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_sound_alikes",
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
