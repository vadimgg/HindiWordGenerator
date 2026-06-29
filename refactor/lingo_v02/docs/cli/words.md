# `lingo words`

Show the derived lexicon: every word across all sentences, how many sentences use
it, and its glosses. This is the cross-cutting view the iOS app turns into "tap a
word → every sentence that uses it."

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Show the derived word lexicon.

Usage: lingo words [OPTIONS]

Options:
      --min-count <N>   Only words appearing in at least N sentences
      --deck <SLUG>     Restrict to words used in this deck
      --json            Machine-readable output
  -h, --help            Print help
```

Help colors: `words`/flags **green**, `<N>`/`<SLUG>` **yellow**, headers **bold cyan**.

## Notes

The lexicon is **derived** from the enriched breakdowns — there's nothing to edit
here. It updates automatically as sentences are enriched. Words are keyed by a
normalized form so inflections can be grouped by the underlying breakdown.

## Example

```bash
lingo words --min-count 3
```

Sample output:

```
Words   (showing 18 of 187, used in ≥3 sentences)

  है     hai      is / are              in 11 sentences
  मैं    maĩ      I / me                in  9 sentences
  एक     ek       one / a               in  9 sentences
  हूँ    hū̃       am (1sg)              in  7 sentences
  यह     yah      this / it             in  5 sentences

Next: lingo ls --status enriched
```

Colors: `Words` heading **bold cyan**; word surface **cyan**; roman & glosses
**dim**; counts **dim**; `Next:` label **yellow**, command **cyan**.

## `--json`

```json
[
  { "form": "है", "key": "hai", "roman": "hai", "count": 11,
    "meanings": ["is", "are"] }
]
```

## See also

[`show`](./show.md) (per-word usage on a sentence) · [`ls`](./ls.md)
