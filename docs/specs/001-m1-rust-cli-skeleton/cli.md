# CLI And User Messages

## Purpose

M1 introduces the first user-visible Rust command:

```bash
hindi doctor
```

During development, run it as:

```bash
cargo run -- doctor
```

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi doctor` | Check whether the project is ready for later Rust workflow steps. | New read-only command. | None. |
| `hindi --help` | Discover available commands. | Shows `doctor`. | None. |
| `hindi doctor --help` | Learn doctor usage. | Shows doctor help. | None. |

`hindi sentences plan` is not available in M1.

## Success Output

Expected output shape:

```text
Hindi Word Generator

Project
  root       /Users/vadim/Projects/Hindi/HindiWordGenerator
  config     missing  hindi.toml

Data
  input      ok       input/
  sentences  ok       input/sentences/
  words      ok       input/words/
  output     ok       output/
  audio      ok       audio/

Prompts
  sentences  ok       generation_prompt_sentences_enrichment.txt
  python     ok       generation_prompt_sentences.txt

Ollama
  service    ok       http://localhost:11434
  model      not checked in M1

Next
  M2 adds: hindi sentences plan --max-batches 1
```

Exact spacing may differ. Section names and information should remain.

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Project root not found | `Project not found` | `Run this command from HindiWordGenerator or one of its subdirectories.` |
| Required path missing | Full report shows `missing` for the path. | Create or restore the missing project path, then rerun `hindi doctor`. |
| Required prompt missing | Full report shows `missing` for the prompt. | Restore the prompt file, then rerun `hindi doctor`. |
| Ollama unreachable | `Ollama is not reachable.` | `Start it, then rerun: hindi doctor`. |
| Unknown command | Parser error. | Show help from the argument parser. |

## Interactive Behavior

- Prompts: none.
- Non-interactive behavior: always direct.
- Picker or fzf behavior: none.

## Color And Emphasis

Color is optional in M1. If added, keep it simple:

| Element | Style | Reason |
|---|---|---|
| `ok` | green | Fast scan of passing checks. |
| `missing` / `unreachable` | yellow or red | Shows user action is needed. |
| paths | cyan or plain | Paths should be easy to copy. |

Do not make color required for understanding.

## UX Review Notes

Open for CLI UX review before implementation if the output shape changes.
