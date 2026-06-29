# `lingo ls`

List sentences. Grouped by deck when more than one deck is shown. Filterable by
deck, status, and audio.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
List sentences.

Usage: lingo ls [OPTIONS]

Options:
      --deck <SLUG>       Show only this deck
      --status <STATUS>   draft | enriching | enriched | active
      --missing-audio     Only sentences without audio
      --id <ID>           Show a specific sentence id (repeatable)
      --json              Machine-readable output
  -h, --help              Print help
```

Help colors: `ls`/flags **green**, `<SLUG>`/`<STATUS>`/`<ID>` **yellow**, headers
**bold cyan**.

## Examples

```bash
lingo ls                       # everything, grouped by deck
lingo ls --deck ch01           # one deck
lingo ls --status draft        # all drafts, across decks
lingo ls --missing-audio       # what still needs audio
```

Sample output:

```
Deck ch01   Complete Hindi · Chapter 01

  sen-ch01-01  enriched  I am a student.
      मैं एक छात्र हूँ।
      maĩ ek chātra hū̃.
      ♪ audio/ch01/sen-ch01-01.mp3
  sen-ch01-02  draft     She is my teacher.
      वह मेरी अध्यापिका हैं।
      ♪ missing

Deck ch02   Complete Hindi · Chapter 02

  sen-ch02-01  enriching Then I went to Delhi.
      फिर मैं दिल्ली गया।
      ♪ missing

Showing 27 sentences · use --deck or --status to filter

Next: lingo show sen-ch01-02
```

Colors: deck headings **bold cyan**; ids **dim**; status words colored by status;
target **cyan**; romanisation **dim**; `♪` present **dim** / `missing` **red**;
`Next:` label **yellow**, command **cyan**.

## Notes

- With `--deck` the per-deck header is omitted (already scoped).
- `--id` may be repeated to pull a specific handful of sentences.
- `--json` returns an array of sentence objects (id, deck, status, target,
  romanisation, english, audio, …) plus a `next` field.

## `Next:`

Points at a useful drill-down — typically `lingo show <id>` for the first
incomplete sentence, or the next pipeline step if the filter implies one.

## See also

[`show`](./show.md) · [`status`](./status.md) · [`words`](./words.md) ·
[`edit`](./edit.md)
