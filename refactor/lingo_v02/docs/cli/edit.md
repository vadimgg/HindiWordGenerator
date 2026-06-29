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
      --keep-derived          When editing target, keep AI-derived fields (warn) instead of clearing
      --to <N>                Move to 1-based position N within its deck
      --json                  Machine-readable output
  -h, --help                  Print help
```

Help colors: `edit`/flags **green**, `<ID>`/placeholders **yellow**, headers **bold cyan**.

## Notes

- Only the fields you pass change; everything else is untouched.
- Editing a field flips its authority to `human`. To re-open a field for the model
  again, clear it (e.g. `--english ""`) — that drops back to `ai` authority.
- Approval is curation, not editing. Use [`approve`](./approve.md) /
  `unapprove` to mark rows ready for study. Approval is separate from lifecycle
  status and is what `study`/`anki` export by default (see
  [`publish`](./publish.md)).
- `--to <N>` reorders within the deck; combine with nothing else for a pure move.

## Editing `target` invalidates derived fields

The target is what everything else is derived from, so changing it (a real content
change) cascades — **impact-based**:

- AI-authored `romanisation` / `english` / `literal` / `register` are **cleared**;
  human-authored fields are **kept, with a warning** that they may now be inconsistent.
- the word-by-word breakdown is cleared and `qa_checked_at` is reset;
- lifecycle drops back to `draft` (it will be re-enriched), which **clears
  approval** too — the approval was for the old content;
- audio is marked **stale** (regenerated on the next `lingo audio`).

`--keep-derived` overrides the clear and keeps the old derived fields with a warning
instead. A trivial edit that doesn't change the sentence's identity (or only affects
pronunciation) invalidates less — see the impact classifier in the architecture
docs.

## Examples

```bash
lingo edit sen-ch01-02 --english "She is my teacher ji."   # lock a nuance
lingo edit sen-ch01-05 --register formal
lingo edit sen-ch01-02 --to 1                              # move to the top
lingo approve sen-ch01-09                                  # approve for study
```

Sample output:

```
✓ Updated  sen-ch01-02   (ch01 · Complete Hindi · Chapter 01)

  english  → She is my teacher ji.   (now human-authored)

Next: lingo show sen-ch01-02
```

Editing `target` shows the invalidation:

```
✓ Updated  sen-ch01-02   (ch01 · Complete Hindi · Chapter 01)

  target invalidated derived fields:
    cleared:  romanisation, literal, breakdown   · qa reset · approval cleared · audio stale
    kept (human, may be inconsistent):  english

Next: lingo enrich ch01
```

Colors: `✓` **green**; id **dim**; changed field name **dim**, new value **cyan**;
the "now human-authored" / invalidation notes **yellow**; `Next:` label **yellow**,
command **cyan**.

## `Next:`

Usually `lingo show <id>` to confirm, or `lingo status` if the edit changed what's
pending.

## See also

[`show`](./show.md) · [`ls`](./ls.md) · [`enrich`](./enrich.md) ·
[`qa`](./qa.md) · [`approve`](./approve.md) · [`deck`](./deck.md)
