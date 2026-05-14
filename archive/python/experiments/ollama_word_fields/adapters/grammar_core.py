"""Word test: generate POS, gender/transitivity, and forms."""

PROMPT = """Generate only the grammar fields for one Hindi vocabulary item.

Return raw JSON only:
{
  "pos": "noun|verb|adjective|adverb|postposition|particle|other",
  "gender": "masculine|feminine|both",
  "transitivity": "transitive|intransitive|both",
  "forms": [
    {"label": "...", "hindi": "...", "roman": "..."}
  ]
}

Rules:
- Omit gender unless the item is a noun/adjective where gender is useful.
- Omit transitivity unless the item is a verb.
- Omit forms entirely when spellings do not change or when you are not confident.
- Never include a form whose Hindi spelling is identical to the base Hindi item.
- Merge labels when two grammatical forms share the same Hindi spelling.
- Do not include postposition combinations.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_grammar_core",
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
