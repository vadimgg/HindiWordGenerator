# `lingo edit`

Hand-edit or reorder a single sentence. Any field you change is marked
`authority: human`, so later [`enrich`](./enrich.md) and [`qa`](./qa.md) passes
leave it alone — this is how you lock in nuances the model keeps getting wrong.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Hand-edit a sentence (edited fields become human-authored).

Usage: lingo edit [OPTIONS] <ID>

Arguments:
  <ID>  Sentence id

Options:
      --target <TEXT>         Target-language sentence
      --romanisation <TEXT>   Romanisation
      --english <TEXT>        English translation
      --literal <TEXT>        Literal word-for-word gloss
      --register <REGISTER>   informal | standard | formal
      --tag <TAG>             Add a tag (repeatable)
      --status <STATUS>       draft | enriching | enriched | active
      --to <N>                Move to 1-based position N within its deck
      --json                  Machine-readable output
  -h, --help                  Print help
```

Help colors: `edit`/flags **green**, `<ID>`/placeholders **yellow**, headers **bold cyan**.

## Notes

- Only the fields you pass change; everything else is untouched.
- Editing a field flips its authority to `human`. To re-open a field for the model
  again, clear it (e.g. `--english ""`) — that drops back to `ai` authority.
- `--status active` is how you mark a sentence curated/approved (the viewer shows
  `active` prominently).
- `--to <N>` reorders within the deck; combine with nothing else for a pure move.

## Examples

```bash
lingo edit sen-ch01-02 --english "She is my teacher ji."   # lock a nuance
lingo edit sen-ch01-05 --register formal
lingo edit sen-ch01-02 --to 1                              # move to the top
lingo edit sen-ch01-09 --status active
```

Sample output:

```
✓ Updated  sen-ch01-02   (ch01 · Complete Hindi · Chapter 01)

  english  → She is my teacher ji.   (now human-authored)

Next: lingo show sen-ch01-02
```

Colors: `✓` **green**; id **dim**; changed field name **dim**, new value **cyan**;
the "now human-authored" note **yellow**; `Next:` label **yellow**, command **cyan**.

## `Next:`

Usually `lingo show <id>` to confirm, or `lingo status` if the edit changed what's
pending.

## See also

[`show`](./show.md) · [`ls`](./ls.md) · [`enrich`](./enrich.md) ·
[`qa`](./qa.md) · [`deck`](./deck.md)
