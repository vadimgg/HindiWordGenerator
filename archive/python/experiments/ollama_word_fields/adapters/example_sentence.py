"""Word test: generate a learner-facing example sentence."""

PROMPT = """Generate one natural Hindi example sentence for a vocabulary item.

Return raw JSON only:
{
  "example_sentence": {
    "hindi": "...",
    "roman": "...",
    "english": "...",
    "breakdown": [
      {"hindi": "...", "roman": "...", "meaning": "..."}
    ]
  }
}

Rules:
- The sentence must use the target item naturally.
- Keep the sentence short and practical for a learner living in India.
- breakdown must include only actual Hindi words, not spaces or punctuation.
- Every breakdown item must include roman and meaning.
- Use tilde nasalisation in romanisation when needed.
- Do not add commentary or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "word_example_sentence",
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
