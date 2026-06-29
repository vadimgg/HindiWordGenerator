# `lingo apply`

Validate a model reply and commit it to the library. **One command for every
stage** — it reads the run's manifest, figures out whether the reply is an
extract, enrich, or QA result, validates it against that stage's contract, and
commits. You never type a stage or a run id.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Validate and commit a model reply (stage auto-detected).

Usage: lingo apply [OPTIONS] [TARGET]

Arguments:
  [TARGET]  A run directory (runs/<id>/) or a reply file.
            Omit to apply the single pending run.

Options:
      --json     Machine-readable output
  -h, --help     Print help
```

Help colors: `apply`/flags **green**, `[TARGET]` **yellow**, headers **bold cyan**.

## How it resolves the target

| You run | Behavior |
|---|---|
| `lingo apply runs/ch01-enrich-9b2c/` | Applies that run. |
| `lingo apply runs/ch01-enrich-9b2c/reply.json` | Same — a reply path resolves to its run. |
| `lingo apply` (one run pending) | Applies it. |
| `lingo apply` (several pending) | Lists them and asks you to name one. |
| `lingo apply` (none pending) | Says so and prints the real `Next:`. |

The run's `run.json` carries `stage` and `deck`; the DB `runs` row is authoritative
for status. See [`package-and-agents.md`](../package-and-agents.md) for the run
shape and truth precedence.

## Validation is strict and re-tryable

`apply` rejects off-contract replies with a specific error and **leaves the run
pending** so you can fix the file and apply again — it never half-commits and
never starts a new run:

```
! reply.json: sentence "sen-ch01-03" has a breakdown token not present in target
  Fix the file and run: lingo apply runs/ch01-enrich-9b2c/
```

It also refuses to overwrite any `authority: human` field and reports the attempt.

## Example

```bash
lingo apply runs/ch01-extract-7f3a/
```

Sample output (applying an extract reply):

```
Deck   ch01   Complete Hindi · Chapter 01

  + sen-ch01-01  draft  I am a student.
      मैं एक छात्र हूँ।
      maĩ ek chātra hū̃.
      ♪ missing
  + sen-ch01-02  draft  She is my teacher.
      वह मेरी अध्यापिका हैं।
      ♪ missing

12 sentences added · 0 skipped

Next: lingo enrich ch01
```

Colors: `+` **green**; ids **dim**; `draft` status **dim**; target **cyan**;
romanisation **dim**; `♪ missing` **red**; `Next:` label **yellow**, command **cyan**.

## `Next:`

Computed from the new library state — typically the next stage for this deck
(`enrich` → `qa` → `audio` → `publish`), or another pending run if one is waiting.

## See also

[`extract`](./extract.md) · [`enrich`](./enrich.md) · [`qa`](./qa.md) ·
[`runs`](./runs.md) · [`status`](./status.md)
