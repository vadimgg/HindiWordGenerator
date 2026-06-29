# `lingo status`

The home screen. Shows the whole library at a glance and the single most useful
command to run next. When you're unsure what to do, run this.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Show library state and the next useful command.

Usage: lingo status [OPTIONS]

Options:
      --json     Machine-readable output (agents read this)
  -h, --help     Print help
```

## Example

```bash
lingo status
```

Sample output:

```
my-hindi-library          (hi · 3 decks)

Decks
  ch01   Complete Hindi · Chapter 01   12 sentences   8 enriched · 4 draft     ♪ 4 missing
  ch02   Complete Hindi · Chapter 02    9 sentences   9 enriching              ♪ 9 missing
  hb     Hindi Basics                   6 sentences   6 enriched · QA'd         ♪ 0 missing

Total   27 sentences · 14 enriched · 9 enriching · 4 draft · 13 audio missing

What to do next
  1  lingo apply runs/ch02-enrich-3a7f/   ch02 has a reply waiting
  2  lingo enrich ch01                    4 draft sentences ready
  3  lingo audio                          13 sentences missing audio

Next: lingo apply runs/ch02-enrich-3a7f/
```

Colors: library name & `Decks`/`Total`/`What to do next` headings **bold cyan**;
slugs **dim**; status words colored by status (`draft` dim, `enriching` yellow,
`enriched`/QA'd green); `♪ N missing` **red**; the ranked list numbers **dim**,
their commands **cyan**; the final `Next:` label **yellow**, command **cyan**.

## How `Next:` is ranked

Most urgent first, at most three shown:

1. A **pending run** waiting for a reply to apply.
2. **Draft** sentences ready to enrich.
3. **Enriched but not QA'd** sentences ready for `qa`.
4. Sentences **missing audio**.
5. Everything enriched + audio present → `publish`.
6. Nothing left → `Done:` message.

## `--json`

```json
{
  "library": "my-hindi-library",
  "language": "hi",
  "decks": [ { "slug": "ch01", "sentences": 12, "enriched": 8, "draft": 4, "audio_missing": 4 } ],
  "totals": { "sentences": 27, "enriched": 14, "enriching": 9, "draft": 4, "audio_missing": 13 },
  "pending_runs": ["ch02-enrich-3a7f"],
  "next": "lingo apply runs/ch02-enrich-3a7f/"
}
```

## See also

[`ls`](./ls.md) · [`deck`](./deck.md) · [`runs`](./runs.md) ·
[`workflows.md`](../workflows.md)
