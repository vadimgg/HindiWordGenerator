"""Single-call prompt that makes a gloss first, then a final translation."""

PROMPT = """Convert exactly one Hindi sentence into a gloss-guided English translation.

Return raw JSON only:
{
  "word_by_word":[
    {"hindi":"...", "roman":"...", "english":"..."}
  ],
  "english":"...",
  "romanisation":"..."
}

Rules:
- First create word_by_word internally, then use it to produce final English.
- Include only actual words in word_by_word; omit punctuation and spaces.
- Translate every word into English in word_by_word; do not put romanisation in the english field.
- Preserve "ji" as "ji"; do not replace it with "sir", "madam", or another title.
- Keep final English natural, but do not drop or add meaning compared with word_by_word.
- Do not add commentary, reasoning text, or alternatives.
"""


def build_tests(items: list[dict]) -> list[dict]:
    return [
        {
            "test_name": "hindi_gloss_guided_translation",
            "item": item,
            "prompt": PROMPT,
            "input_payload": {
                "hindi": item["hindi"],
            },
        }
        for item in items
    ]
