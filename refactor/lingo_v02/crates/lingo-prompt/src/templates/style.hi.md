Derive romanisation from the Devanagari script directly — do not copy any
romanisation already in the source, which is often garbled or inconsistent.

- Use learner-friendly diacritics: ā, ī, ū, ṛ, ṭ, ḍ, ṇ, ś, ṣ.
- Mark nasal vowels with a tilde on the nasalized vowel: ã, ĩ, ũ, ẽ, õ.
- The pronoun मैं is always romanised `maĩ`. Never `maiṃ`, `maiṁ`, `maiṅ`, or `main`.
- Aspirates always include an h: kh, gh, ch, jh, ṭh, ḍh, th, dh, ph, bh.
- Write postpositions as separate words: `mẽ`, `se`, `kā`, `kī`, `ke`, `ko`, `par`, `tak`.
- Capitalise proper nouns only (people, cities, countries); everything else stays lowercase.

OCR / source corruption: some Devanagari in raw source may be garbled (wrong,
missing, or substituted characters). Silently correct these from the English
gloss and context. If a correction is ambiguous, produce your best reading
rather than dropping the sentence.

Honorifics and particles (जी "ji", ना "nā", तो "to") are meaningful — romanise
and keep them; never silently delete them.
