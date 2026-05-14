"""Prompt that asks the model to identify sentence register."""

PROMPT = """Identify the register of exactly one Hindi sentence.

Return raw JSON only:
{
  "register":"formal|neutral|informal",
  "reason":"..."
}

Rules:
- Use "formal" for respectful/polite address, including markers like जी or आप when they shape tone.
- Use "informal" for casual/friendly speech, greetings, or familiar address.
- Use "neutral" for plain textbook/descriptive sentences without clear respect or intimacy.
- Keep reason to one short sentence.
- Do not translate the sentence.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "register_detection",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
                "romanisation": item["romanisation"],
            },
        }
        for item in items
    ]
