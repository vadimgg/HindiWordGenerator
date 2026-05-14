"""Strict Hindi-only translation and romanisation prompt."""

PROMPT = """Convert exactly one Hindi sentence into faithful English and romanisation.

Return raw JSON only:
{
  "english":"...",
  "romanisation":"..."
}

Rules:
- Prefer fidelity over fluent paraphrase.
- Preserve all names exactly as romanised in the final romanisation.
- Preserve "ji" as "ji"; do not replace it with "sir", "madam", or a title.
- Preserve numbers exactly; use digits in English when the meaning is numeric.
- Preserve whether the sentence is a question, statement, greeting, or answer.
- Do not infer missing nouns from context.
- Do not add commentary, alternatives, or word-by-word notes.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "hindi_strict_translation_romanisation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
            },
        }
        for item in items
    ]
