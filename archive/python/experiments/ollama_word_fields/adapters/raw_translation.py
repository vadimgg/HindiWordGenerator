"""Word test: infer basic English and romanisation from raw Hindi only."""

PROMPT = """Convert exactly one Hindi vocabulary item into English and romanisation.

Return raw JSON only:
{
  "english": "...",
  "romanisation": "..."
}

Rules:
- Use the Hindi as the only source of truth.
- If the Hindi item is a phrase or compound verb, preserve that structure.
- Use practical learner-facing English, not dictionary noise.
- Use readable romanisation with tilde nasalisation when needed.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_raw_translation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {"hindi": item["hindi"]},
        }
        for item in items
    ]
