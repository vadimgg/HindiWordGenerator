# `lingo show`

Show one sentence in full: all fields, which are human-authored, the word-by-word
breakdown, and the words it shares with the rest of the library.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Show one sentence in full.

Usage: lingo show [OPTIONS] <ID>

Arguments:
  <ID>  Sentence id

Options:
      --json     Machine-readable output
  -h, --help     Print help
```

Help colors: `show`/flags **green**, `<ID>` **yellow**, headers **bold cyan**.

## Example

```bash
lingo show sen-ch01-01
```

Sample output:

```
sen-ch01-01   enriched · QA'd   (ch01 · Complete Hindi · Chapter 01)

  English       I am a student.
  Target        मैं एक छात्र हूँ।
  Romanisation  maĩ ek chātra hū̃.            (human)
  Literal       I / one / student / am
  Register      standard
  Audio         audio/ch01/sen-ch01-01.mp3
  Tags          beginner, identity

  Words
    मैं    maĩ      I / me            in 4 sentences
    एक     ek       one / a           in 9 sentences
    छात्र  chātra   student / pupil   in 2 sentences
    हूँ    hū̃       am (1sg)          in 7 sentences
```

Colors: id **dim**, status colored by status; field labels **dim**, English
**bold white**, Target **cyan**, romanisation/literal/gloss **dim**;
`(human)` authority note **yellow**; `Words` heading **bold cyan**; per-word
surface **cyan**, roman & gloss **dim**, the "in N sentences" count **dim**.

## Notes

- Fields the learner authored are flagged `(human)` so you can see what `enrich`
  and `qa` won't touch.
- "in N sentences" links the word to the lexicon ([`words`](./words.md)).
- `--json` returns the full record including `authority`, `breakdown`, and
  `provenance`.

## `Next:`

Typically `lingo edit <id>` (if a field looks wrong) or back to `lingo ls`.

## See also

[`ls`](./ls.md) · [`edit`](./edit.md) · [`words`](./words.md)
