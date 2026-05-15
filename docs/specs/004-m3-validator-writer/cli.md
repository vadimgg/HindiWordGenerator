# CLI And User Messages

## Purpose

M3 does not add a production write command. It adds validation/writer
infrastructure that M4 will call later. The only expected user-visible CLI
behavior is that existing commands still work and continue to tell the user that
generation arrives in M4.

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi sentences plan --max-batches <n>` | Preview pending sentence work. | No user-visible output change expected, except harmless wording if shared planner code is extracted. | None. Read-only. |
| `hindi --help` / `hindi sentences --help` | Discover available commands. | Should not list `generate` yet. | None. |

## Help Text

| Command | Expected Help Change |
|---|---|
| `hindi --help` | Still lists `doctor`, `source ids`, and `sentences plan`; does not list `sentences generate`. |
| `hindi sentences --help` | Still lists only `plan`. |

## Success Output

No new success output is introduced by this spec.

Existing smoke command should still render the M2 shape:

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

## Progress And Log Messages

None.

## Warning And Error Output

No new CLI errors are introduced. Validation errors are internal typed reports
in M3; M4 will decide how to print them around generation.

## Interactive Behavior

- Prompts: none.
- Non-interactive behavior: all validation/writer APIs are direct and
  testable.
- Picker or fzf behavior: none.

## Color And Emphasis

No new color or emphasis rules.

## UX Review Notes

Review should focus on the absence of accidental CLI surface growth. If this
spec exposes a hidden write command, reject it.
