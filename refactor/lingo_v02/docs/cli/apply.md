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
      --dry-run   Validate the reply and report what would change; commit nothing
      --oldest    Apply the oldest pending run (deterministic, for scripts/agents)
      --all       Apply all pending runs in created order
      --json      Machine-readable output
  -h, --help      Print help
```

Help colors: `apply`/flags **green**, `[TARGET]` **yellow**, headers **bold cyan**.

## How it resolves the target

| You run | Behavior |
|---|---|
| `lingo apply runs/ch01-enrich-9b2c/` | Applies that run. |
| `lingo apply runs/ch01-enrich-9b2c/reply.json` | Same — a reply path resolves to its run. |
| `lingo apply` (one run pending) | Applies it. |
| `lingo apply` (several pending, interactive TTY) | Lists them so you can pick one. |
| `lingo apply` (several pending, `--json` / non-TTY) | **Never prompts** — returns a `blocked` result (see below). |
| `lingo apply --oldest` | Applies the oldest pending run deterministically. |
| `lingo apply --all` | Applies every pending run in created order. |
| `lingo apply` (none pending) | Says so and prints the real `Next:`. |

The run's `run.json` carries `stage` and `deck`; the DB `runs` row is authoritative
for status. See [`package-and-agents.md`](../package-and-agents.md) for the run
shape and truth precedence.

## Deterministic for agents (never prompts in `--json`)

In non-interactive mode with several pending runs, `apply` does not ask — it
returns a `blocked` result with exit code `3` (see the result contract and exit
codes in [`CLI.md`](../CLI.md)):

```json
{ "blocked": { "reason": "multiple_pending_runs",
               "fix": "lingo apply runs/ch01-enrich-9b2c/" },
  "pending_runs": ["ch01-enrich-9b2c", "ch02-qa-1d4e"] }
```

An agent then picks one (or uses `--oldest` / `--all`). Humans get the same
information as a listed choice.

## Validation is strict, transactional, and re-tryable

- Validates the **entire** reply before writing anything; commits in **one SQLite
  transaction** — it never half-commits.
- A bad reply **leaves the run pending** (records the validation error) so you fix
  the file and apply again — it never starts a new run:

```
! reply.json: sentence "sen-ch01-03" has a breakdown token not present in target
  Fix the file and run: lingo apply runs/ch01-enrich-9b2c/
```

- Refuses to overwrite any `authority: human` field and reports the attempt.
- **Idempotent:** re-applying a run whose reply is byte-identical to what was
  already applied is a no-op; applying a *different* reply to an already-applied
  run is rejected (`already_applied_different_reply`).

## `--dry-run`

Validates and reports what *would* change without touching the library — ideal
for agents to check a reply before committing:

```
Valid reply.

Would update:
  12 sentences enriched
  94 token rows replaced
  0 human-authored fields touched

Next: lingo apply runs/ch01-enrich-9b2c/
```

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
