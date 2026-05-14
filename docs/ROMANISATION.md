# Romanisation Policy

Use a custom learner-facing romanisation based on common IAST-style diacritics,
with one project-specific preference: nasalised vowels use a tilde on the vowel.
The goal is readable study data, not strict ISO 15919 coverage.

## Scope

This policy applies to new or repaired:

- word card `romanisation`
- sentence card `romanisation`
- sentence `words[].roman`
- sentence `tokens[].roman`
- example-sentence breakdowns inside word cards

Do not rename existing audio files just to normalize older romanisation. File
paths may preserve historical forms until an explicit audio regeneration task.

## Core Rules

Use long-vowel macrons:

- Hindi `आ`
  Roman `ā`
  Avoid `aa`
- Hindi `ई`
  Roman `ī`
  Avoid `ee`
- Hindi `ऊ`
  Roman `ū`
  Avoid `oo`

Use retroflex dots:

- Hindi `ट`
  Roman `ṭ`
  Avoid plain `t` when retroflex
- Hindi `ड`
  Roman `ḍ`
  Avoid plain `d` when retroflex
- Hindi `ढ`
  Roman `ḍh`
  Avoid `dh` when retroflex
- Hindi `ड़`
  Roman `ṛ`
  Avoid `r` or `d`
- Hindi `ढ़`
  Roman `ṛh`
  Avoid `rh` or `dh`

Use aspirates as digraphs:

- Hindi `च`
  Roman `ch`
- Hindi `ख`
  Roman `kh`
- Hindi `घ`
  Roman `gh`
- Hindi `छ`
  Roman `chh`
- Hindi `झ`
  Roman `jh`
- Hindi `थ`
  Roman `th`
- Hindi `ध`
  Roman `dh`
- Hindi `फ`
  Roman `ph` or `f`, following source/common usage
  (policy guidance for new generation; not validator-enforced)
- Hindi `भ`
  Roman `bh`

Use tilde nasalisation on the vowel:

- `maĩ`
- `yahā̃`
- `haĩ`
- `gharõ`
- `laṛkiyā̃`

Avoid nasalisation forms such as:

- `main`
- `maiṃ`
- `maṁ`
- `maṅ`
- `yahāṁ`
- `yahā̃̃`
- `o̐`

## Reconstruction

For sentence cards, `tokens[].roman` must reconstruct `romanisation` when word
tokens are joined with the spaces and punctuation taken from the `romanisation`
string itself (not from the Hindi text). Compare after Unicode NFC
normalization. The viewer uses this relationship to highlight word-by-word
reading without storing spaces and punctuation as token entries.

`tokens` must contain words only. Do not add token entries for spaces,
commas, question marks, danda, or other punctuation.

## Existing Data

Some accepted output may contain older spellings such as `mahãgā` or filenames
that omit diacritics. Do not churn accepted data only for normalization. Apply
this policy to new generation, prompt updates, validation, and manual repair.
