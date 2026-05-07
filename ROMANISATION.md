# Romanisation Policy

Use learner-friendly romanisation with diacritics and tilde nasalisation.

## Nasalisation

Use tilde nasalisation consistently:

- `maĩ`
- `yahā̃`
- `haĩ`
- `gharõ`
- `laṛkiyā̃`

Do not use `ṃ`, `ṁ`, or `o̐` for nasalised vowels in newly generated cards.

## Existing Data

Do not migrate existing generated output or audio filenames just to normalize
romanisation. Some filenames and already-generated cards may preserve older
forms. Apply this policy to new generation, prompt updates, and manual repairs
going forward.

Sentence `tokens[].roman` values must still reconstruct `romanisation` exactly,
even when the source romanisation contains older or inconsistent forms.
