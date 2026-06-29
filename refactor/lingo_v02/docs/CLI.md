# Lingo CLI Reference

`lingo` builds a local, sentence-centric library for learning a language (Hindi,
Japanese, …) and exports it to study targets (a custom iOS app, Anki). It is
**terminal-first and self-teaching**: every command that changes something prints
the next command to run, so you can complete the whole pipeline by copying the
`Next:` line each time.

This page is the hub. Each subcommand has its own page under [`cli/`](./cli/). How
the tool is used end-to-end is in [`workflows.md`](./workflows.md). The on-disk
layout is in [`package-and-agents.md`](./package-and-agents.md). The `library.db`
schema is in [`schema.md`](./schema.md).

---

## The one idea

Lingo is **a typed SQLite library with an offline LLM handoff.** `library.db` is
the source of truth. Lingo never calls a model — instead each model-facing stage
writes a **prompt file** and reads back a **reply file**:

```
          claim work        you / agent / web          validate + commit
state ──────────────▶ task.md ──────────▶ reply.{yaml,json} ──────────▶ library.db
        (lingo writes)                                       (lingo applies)
```

A human pasting into ChatGPT, a coding agent reading/writing files, and the web
viewer are the **same three commands** with a different actor in the middle.

---

## Mental model

`library → deck → sentence`

- **Library** — the workspace: one directory, one `library.db`, one language.
- **Deck** — one unit of material you study and publish, with a memorable slug
  (`ch01`). What becomes an Anki deck / a study unit in the app.
- **Sentence** — the studyable item; belongs to exactly one deck. Its id is
  **public and permanent** (exports key study progress to it).
- **Word** — derived lexicon across all sentences.
- **Run** — one prompt→reply handoff (a directory under `runs/` + a DB row).

> New code and docs use `deck`. If prototype code still mentions "batch",
> replace it during the refactor; that vocabulary is not part of the v0.2 model.

---

## Invocation & global options

```
lingo [OPTIONS] <COMMAND>

Options:
      --color <WHEN>   When to use color: auto | always | never  [default: auto]
      --no-color       Disable ANSI color (same as --color never)
      --ascii          Use ASCII fallbacks for box/status glyphs (planned)
  -h, --help           Print help (use after any command for that command's help)
  -V, --version        Print version
```

> `--ascii` is **planned, not yet implemented.** `--no-color` removes ANSI codes
> but does not change glyphs; `--ascii` will additionally swap `✓ ✗ ● → ♪ ·` for
> ASCII equivalents (`OK ! * -> (audio) -`) for terminals or doc pipelines that
> render Unicode poorly. The target-language text itself (Devanagari, kana, …) is
> never transliterated by `--ascii` — only the UI chrome.

- With **no command**, `lingo` opens the local viewer (see [`cli/viewer.md`](./cli/viewer.md)).
- Color is auto-disabled when output is not a terminal, or when `NO_COLOR` is set.
- `--json` is available on every command that reports state; it emits the same
  data with no ANSI codes and always includes a terminal state — `next`, `done`,
  or `blocked` (see [Result contract](#result-contract)). Agents read this.

### Top-level `--help`

```
A local sentence library for language learning.

Usage: lingo [OPTIONS] <COMMAND>

Build (model handoff):
  extract   Start a deck from raw material → writes a prompt task
  enrich    Claim draft sentences for word-by-word enrichment → writes a task
  qa        Ask the model to review enriched sentences → writes a task
  apply     Validate and commit a model reply (stage auto-detected)

Build (direct, no model):
  import    Merge an existing lingo package into this library
  edit      Hand-edit a sentence (edited fields become human-authored)

Produce:
  approve   Approve or unapprove enriched sentences for study
  audio     Synthesize speech for sentences missing it
  publish   Export a deck or the library: package | study | anki

Inspect:
  status    Library state and the next useful command (the home screen)
  ls        List sentences
  show      Show one sentence in full
  words     Show the derived word lexicon
  deck      Inspect and manage decks (list, show, set, delete)
  runs      Manage the prompt/reply handoff directory (ls, clean)

Workspace:
  init      Create a new library
  config    Read or write library settings
  doctor    Check setup and find recoverable problems
  viewer    Serve the local web viewer (default when no command is given)

Options:
      --color <WHEN>  auto | always | never  [default: auto]
      --no-color      Disable ANSI color
  -h, --help          Print help
  -V, --version       Print version

Run `lingo <command> --help` for details. New here? Run `lingo status`.
```

Help rendering colors: section headers **bold cyan**, command/flag literals
**green**, value placeholders **yellow**, the closing hint line **dim**.

---

## Output & color legend  *(canonical — every page refers here)*

Color is **semantic**: the same kind of information is always the same color. With
`--no-color` the same information stays readable through prefixes and labels.

| Element | Color | ANSI |
|---|---|---|
| Heading / section title | bold cyan | `1;36` |
| Success, `✓`, `+` added/generated | green | `32` |
| `Next:` label, warnings, `~` skipped | yellow | `33` |
| Commands, target-language text, `Next:` command | cyan | `36` |
| Sentence id, file paths, romanisation, gloss, secondary detail | dim | `2` |
| Errors, `!` | bold red | `1;31` |

Lifecycle/status badges:

| Status | Meaning | Color |
|---|---|---|
| `draft` | extracted, not yet enriched | dim |
| `enriching` | claimed by an open enrich run | yellow |
| `enriched` | has romanisation, gloss, breakdown | green |
Approval badge:

| Badge | Meaning | Color |
|---|---|---|
| `✓approved` | approved for study/export | bold green |

Audio markers:

| Marker | Meaning | Color |
|---|---|---|
| `♪ audio/sen-….mp3` | present | dim |
| `♪ missing` | not generated yet | red |
| `♪ copied` | copied during import/publish | green |
| `♪ skipped` | left as-is (duplicate / already exists) | yellow |

Line prefixes:

| Prefix | Meaning | Color |
|---|---|---|
| `+` | added / enriched / generated | green |
| `~` | skipped | yellow |
| `✓` | completed action | green |
| `!` | warning or error | red |

### The sentence block

Wherever a sentence is printed, it uses the same ≤4-line block. Missing fields are
omitted silently (no blank lines):

```
  <prefix> <id>  <status>  <English>
      <target — Hindi/Japanese/…>
      <romanisation>
      ♪ <audio>
```

Example (colors annotated):

```
  + sen-ch01-01  enriched  I am a student.        ← + green · id dim · enriched green
      मैं एक छात्र हूँ।                              ← target cyan
      maĩ ek chātra hū̃.                            ← romanisation dim
      ♪ audio/sen-ch01-01.mp3                       ← present, dim
```

### The `Next:` line  *(the self-teaching contract)*

Every state-changing or inspecting command ends with a single copyable command:

```
Next: lingo enrich ch01
```

- Real and copyable — no placeholders like `<run-id>`.
- Reflects actual current state, not a generic suggestion.
- `Next:` label is yellow, the command is cyan.
- When there is genuinely nothing left:
  `Done: all sentences enriched, QA'd, audio generated, and published.`

---

## Result contract

Every completed command (except long-running ones like `viewer`) ends in exactly
one of three terminal states. In `--json` this is explicit; in styled output it's
the `Next:` / `Done:` / `!` line.

| State | Styled output | `--json` |
|---|---|---|
| **next** — there's a clear next step | `Next: lingo …` (copyable, no placeholders) | `"next": "lingo …"` |
| **done** — nothing left to do | `Done: …` | `"done": true` |
| **blocked** — needs a human/agent choice or fix | `! <reason>` + a fix line | `"blocked": { "reason": "...", "fix": "lingo …" }` |

Rules:
- No unresolved placeholders ever appear in a `next`. If a command can't form a
  concrete command (e.g. [`init`](./cli/init.md) without `--example`), it prints an
  instruction block instead of a `Next:`.
- Long-running commands (`viewer`) are exempt — they don't terminate.
- An agent reads `next` / `done` / `blocked` from `--json` and never has to parse
  styled text. A `blocked` result is how a non-interactive run reports a choice it
  refuses to guess (see [`apply`](./cli/apply.md) with multiple pending runs).

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (`next` or `done`). |
| `1` | Validation or user error (bad reply, bad argument). |
| `2` | Environment/setup error (no library, missing backend). |
| `3` | A choice is required and none was given (`blocked`; e.g. multiple pending runs in non-interactive mode). |
| `4` | Internal error. |

---

## Command index

Build (model handoff) — the loop you live in:

| Command | Purpose |
|---|---|
| [`extract`](./cli/extract.md) | Raw material → a deck + a prompt task |
| [`enrich`](./cli/enrich.md) | Draft sentences → word-by-word enrichment task |
| [`qa`](./cli/qa.md) | Review enriched sentences for mistakes |
| [`apply`](./cli/apply.md) | Validate & commit any model reply (stage auto-detected) |

Build (direct):

| Command | Purpose |
|---|---|
| [`import`](./cli/import.md) | Merge an existing lingo package |
| [`edit`](./cli/edit.md) | Hand-edit / reorder a sentence |

Produce:

| Command | Purpose |
|---|---|
| [`approve`](./cli/approve.md) | Approve or unapprove enriched sentences for study |
| [`audio`](./cli/audio.md) | Generate speech for sentences missing it |
| [`publish`](./cli/publish.md) | Export `package` / `study` / `anki` |

Inspect:

| Command | Purpose |
|---|---|
| [`status`](./cli/status.md) | Home screen: state + next action |
| [`ls`](./cli/ls.md) | List sentences |
| [`show`](./cli/show.md) | One sentence in full |
| [`words`](./cli/words.md) | The derived lexicon |
| [`deck`](./cli/deck.md) | list · show · set · delete |
| [`runs`](./cli/runs.md) | ls · clean |

Workspace:

| Command | Purpose |
|---|---|
| [`init`](./cli/init.md) | Create a library |
| [`config`](./cli/config.md) | get · set settings |
| [`doctor`](./cli/doctor.md) | Diagnose setup & recoverable problems |
| [`viewer`](./cli/viewer.md) | Serve the local web UI |

---

## Page template

Every `cli/<command>.md` page follows the same shape so they're predictable:

1. **Purpose** — one line.
2. **`--help`** — sample help output, with a note on help colors.
3. **Options** — table of flags.
4. **Examples** — real invocations and their sample output (colors annotated).
5. **`Next:`** — what the command suggests next and why.
6. **See also** — related pages.
