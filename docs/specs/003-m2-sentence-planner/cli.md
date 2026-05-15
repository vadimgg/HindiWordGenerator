# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi sentences plan --max-batches <n>` | Preview pending sentence generation. | New read-only planner command. | None. |
| `hindi doctor` | See next recommended command. | Update next-step text to the real planner command. | None. |

## Help Text

```text
hindi sentences --help

Usage:
  hindi sentences plan --max-batches <n>

Commands:
  plan    Preview pending sentence batches without writing output
```

`hindi --help` should list `sentences plan` as available once implemented.

## Success Output

```text
Sentence Plan

Sources
  files              6
  source items       296
  valid ids          296

Accepted Output
  batch files        4
  accepted cards     20
  done               0
  missing lineage    20
  source changed     0

Plan
  max batches        1
  batch size         5
  planned batches    1
  planned items      5
  pending items      296
  deferred items     291

Planned Files
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Next
  M4 adds: hindi sentences generate --max-batches 1
```

If the command prints source examples or source errors containing Hindi, it must
follow the Hindi display rule:

```text
Hindi   क्या आप कमला जी हैं?
Roman   kyā āp Kamalā jī haĩ?
English Are you Kamala?
```

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Missing `--max-batches` | `Missing required option: --max-batches <n>` | Rerun with `cargo run -- sentences plan --max-batches 1`. |
| Invalid `--max-batches` | `--max-batches must be a positive integer.` | Use a value such as `1`. |
| Duplicate source ID | Same source-ID error style as M1.5. | Fix YAML ID and rerun. |
| Malformed output JSON | Print the file path and parsing problem. | Fix/remove the output file or defer until audit/repair spec. |

## Interactive Behavior

- Prompts: none.
- Non-interactive behavior: command must run unattended.
- Picker or fzf behavior: none.

## Color And Emphasis

Use the same plain, aligned text style as `hindi doctor` and `source ids`.
Color is optional.

## UX Review Notes

`plan` is the right verb here: the command answers “what would happen?” and
writes nothing.
