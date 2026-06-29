# `lingo config`

Read or write library settings in `config.toml`. Settings are addressed by dotted
keys.

See [`CLI.md`](../CLI.md) for the shared color/output legend.

## `--help`

```
Read or write library settings.

Usage: lingo config <SUBCOMMAND>

Subcommands:
  get [KEY]          Print the whole config, or one dotted key
  set <KEY> <VALUE>  Set a dotted key and write config.toml

Run `lingo config <subcommand> --help` for details.
```

## Common keys

```toml
[target]
profile = "hindi"          # language profile

[library]
title = "Lingo Sentences"  # default library / Anki root title
language = "hi"

[display]
lead = "romanisation"      # which line leads in listings
show_secondary = true

[audio]
backend = "gtts"           # default TTS backend

[publish]
package = "out/<deck>"     # default package destination
study   = "out/study"      # default study destination
anki    = "out/<deck>.apkg"
deck    = "Lingo::Sentences"  # Anki deck-name template base
```

## Examples

```bash
lingo config get                       # print the whole file
lingo config get audio.backend         # → gtts
lingo config set display.lead target   # lead listings with the target script
lingo config set audio.backend gtts
```

Sample output (`set`):

```
✓ Settings   config.toml
  audio.backend = gtts

Next: lingo audio
```

Colors: `Settings` heading **bold cyan**; the key **cyan**, value plain;
`✓` **green**; `Next:` label **yellow**, command **cyan**. `get` of the whole file
prints raw TOML uncolored.

## `Next:`

`set` points at the command the changed setting affects (e.g. changing
`audio.backend` suggests `lingo audio`).

## See also

[`init`](./init.md) · [`audio`](./audio.md) · [`publish`](./publish.md)
