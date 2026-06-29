# `lingo doctor`

Check that the library can run the workflow, and surface recoverable problems with
a `Next:` line to fix each one. Run it when something feels off.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Check setup and find recoverable problems.

Usage: lingo doctor [OPTIONS]

Options:
      --json     Machine-readable output
  -h, --help     Print help
```

## What it checks

- `library.db` present, with `meta.schema_version` at a supported value
- standard directories exist (`raw/`, `runs/`, `audio/`, `out/`)
- the configured audio backend is available (and any credentials a future backend
  needs)
- optional tools (`fzf`, used by future voice selection)
- **pending replies** that can be applied right now
- **abandoned pending runs** (a claim with no applied reply; `enriching` is derived
  from these, so an abandoned run is what leaves sentences looking stuck)
- **abandoned empty decks** (created by `extract`, never applied)
- **stale or broken audio** (fingerprint mismatch, or a referenced file missing)
- `run.json` manifests that disagree with the DB (can be regenerated)

## Example

```bash
lingo doctor
```

Healthy:

```
✓ library.db        schema v1
✓ directories       raw/ runs/ audio/ out/
✓ audio backend     gtts (networked)
✓ no pending runs · no stale audio

All good.
```

Problems found:

```
✓ library.db        schema v1
! pending run        ch02-enrich-3a7f has a reply waiting
! abandoned run      ch01-enrich-9b2c claimed 9 sentences, never applied
! abandoned deck     ch03 has 0 applied sentences

Next: lingo apply runs/ch02-enrich-3a7f/
```

Colors: `✓` checks **green**; `!` problems **red**; the check label **dim**;
`All good.` **green**; `Next:` label **yellow**, command **cyan**. When healthy,
output stays quiet — no wall of green.

## `Next:`

The single highest-priority fix (apply a pending run, reset stuck sentences, clean
abandoned decks, …). When healthy, no `Next:` — it just says `All good.`

## See also

[`status`](./status.md) · [`runs`](./runs.md) · [`enrich`](./enrich.md)
(`--reset`)
