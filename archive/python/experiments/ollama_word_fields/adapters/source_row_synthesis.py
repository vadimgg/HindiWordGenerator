"""Word test: combine Hindi, romanisation, and English source-row metadata."""

PROMPT = """Read exactly one Hindi vocabulary source row and normalize the core fields.

The source row shape is:
Hindi item (romanisation);English gloss

Return raw JSON only:
{
  "hindi": "...",
  "romanisation": "...",
  "english": "...",
  "pos": "noun|verb|adjective|adverb|postposition|particle|other"
}

Rules:
- Use the supplied source row as reference, but fix obvious formatting only.
- Do not expand into a full card.
- Preserve multiple common senses when the source row gives them.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_source_row_synthesis",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {"source_row": item["input_line"]},
        }
        for item in items
    ]
