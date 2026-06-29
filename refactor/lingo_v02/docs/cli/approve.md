# `lingo approve`

Approve or unapprove enriched sentences for study. Approval is the curation gate
for study targets: [`publish --format study`](./publish.md) and
[`publish --format anki`](./publish.md) export approved rows by default.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Approve or unapprove enriched sentences for study.

Usage: lingo approve [OPTIONS] <TARGET>
       lingo unapprove [OPTIONS] <TARGET>

Arguments:
  <TARGET>  A deck slug or sentence id

Options:
      --all          Approve all enriched sentences in the deck (default for deck targets)
      --json         Machine-readable output
  -h, --help         Print help
```

Help colors: `approve`/`unapprove`/flags **green**, `<TARGET>` **yellow**, headers
**bold cyan**.

## Behavior

- `lingo approve <sentence-id>` approves one enriched sentence.
- `lingo approve <deck>` approves all enriched sentences in that deck.
- `lingo approve <deck> --all` is accepted as the explicit form of the same
  deck-wide action.
- `lingo unapprove <sentence-id>` removes approval from one sentence.
- `lingo unapprove <deck>` removes approval from all sentences in that deck.
- Draft rows cannot be approved. They are reported as skipped with a fix command.
- Approval changes only the curation flag; sentence text, derived fields, QA, and
  audio are untouched.

`--interactive` is intentionally **not** part of Phase 4. Bulk review can happen
in the viewer later. The first CLI implementation supports one sentence or a
whole deck so the pipeline can continue without building a TUI.

## Examples

```bash
lingo approve sen-ch01-01
lingo approve ch01
lingo approve ch01 --all
lingo unapprove sen-ch01-01
```

Sample output:

```
Approved  ch01

  ✓ sen-ch01-01  enriched  I am a student.
      मैं एक छात्र हूँ।
      maĩ ek chātra hū̃.
      ♪ audio/sen-ch01-01.mp3

  ~ sen-ch01-04  skipped draft  Enrich before approving.

11 approved · 1 skipped

Next: lingo audio ch01
```

Colors: `✓` **green**, `~` **yellow**; ids and file paths **dim**; target text
**cyan**; romanisation **dim**; `Next:` label **yellow**, command **cyan**.

## `Next:`

Usually `lingo audio <deck>` when approved rows need audio, otherwise
`lingo publish <deck>` or `lingo status`.

## See also

[`status`](./status.md) · [`ls`](./ls.md) · [`publish`](./publish.md) ·
[`workflows.md`](../workflows.md)
