# `lingo init`

Create a new library (workspace): a `library.db`, a `config.toml`, an `AGENTS.md`
contract for coding agents, and the standard folders. Run once per language.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Create a new library.

Usage: lingo init [OPTIONS] [DIR]

Arguments:
  [DIR]  Directory to create the library in [default: .]

Options:
      --lang <PROFILE>   Language profile to scaffold [default: hindi]
      --example          Also write a sample raw/example.md to start from
      --json             Machine-readable output
  -h, --help             Print help
```

Help colors: `init`/flags **green**, `[DIR]`/`<PROFILE>` **yellow**, headers **bold cyan**.

## What it creates

```
<DIR>/
  library.db        # empty library at the current schema version
  config.toml       # language, default titles, audio backend, publish dests
  AGENTS.md         # the file-handoff contract (see workflows.md, Workflow B)
  raw/  runs/  audio/  out/  prompts/
```

The `--lang` profile seeds language-specific defaults (script, romanisation
convention, prompt style rules). `hindi` is built in; others are added as
profiles. `--example` additionally writes a small `raw/example.md` so the `Next:`
line can point at a real file you can run immediately.

## Examples

```bash
lingo init my-hindi-library --lang hindi --example
cd my-hindi-library
```

Sample output (with `--example`):

```
✓ Library created   my-hindi-library   (hindi · hi)

  ✓ library.db
  ✓ config.toml
  ✓ AGENTS.md
  ✓ raw/example.md
  ✓ raw/ runs/ audio/ out/ prompts/

Next: lingo extract raw/example.md --deck ch01
```

Without `--example` there is no file to point at, so init prints an instruction
block instead of a `Next:` line (keeping the no-placeholder rule):

```
✓ Library created   my-hindi-library   (hindi · hi)

Add your source text under raw/, then run:
  lingo extract raw/<your-file>.md --deck ch01
```

Colors: heading **bold cyan**; each `✓` **green**; path/profile detail **dim**;
the instruction verb line **dim**, the indented command **cyan**; a real `Next:`
uses **yellow** label + **cyan** command.

## `Next:`

With `--example`, points at `lingo extract raw/example.md …` (a real, runnable
command). Without it, init prints the instruction block above rather than a
`Next:` with a `<placeholder>` — see the result-contract rules in
[`CLI.md`](../CLI.md).

## See also

[`config`](./config.md) · [`extract`](./extract.md) · [`workflows.md`](../workflows.md)
