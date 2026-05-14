# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi source ids check` | Verify active source YAML is ready for planning. | New M1.5 validation command. | None. |
| `hindi source ids migrate --check` | Preview which YAML files would receive IDs. | New dry-run migration mode. | None. |
| `hindi source ids migrate` | Add missing stable IDs to active source YAML. | New one-off migration command. | Writes `input/sentences/*.yaml` and `input/words/*.yaml` only. |

## Help Text

```text
hindi source ids --help

Usage:
  hindi source ids check
  hindi source ids migrate [--check]

Commands:
  check      Validate source IDs without writing files
  migrate   Add missing source IDs

Options:
  --check    Preview migration without writing files
```

## Success Output

### Already Complete

```text
Source IDs

Scope
  sentences  input/sentences/*.yaml
  words      input/words/*.yaml

Result
  files      13
  items      182
  missing    0
  duplicate  0
  malformed  0

Ready
  Source YAML has stable item IDs.
```

### Migration Needed

```text
Source IDs

Result
  files      13
  items      182
  missing    182
  duplicate  0
  malformed  0

Next
  cargo run -- source ids migrate
```

### Migration Applied

```text
Source ID Migration

Changed Files
  input/sentences/complete_hindi_chapter_02_sentences.yaml  added 13 ids
  input/words/complete_hindi_chapter_01_words.yaml          added 34 ids

Result
  files changed  13
  ids added      182

Next
  cargo run -- source ids check
```

Whenever a Hindi source item is printed in an error or review message, follow
the project Hindi display rule:

```text
Hindi   क्या आप कमला जी हैं?
Roman   kyā āp Kamalā jī haĩ?
English Are you Kamala?
```

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Duplicate ID in one file | `Duplicate source id "0007" in input/sentences/...` | Manually choose a new file-scoped ID for one item, then rerun check. |
| Malformed ID | `Malformed source id "chapter-2-1"; expected a quoted zero-padded numeric string like "0001".` | Edit the source YAML ID manually or remove it and rerun migration. |
| Source YAML parse error | `Could not parse input/sentences/...` | Fix YAML syntax, then rerun check. |

## Interactive Behavior

- Prompts: none.
- Non-interactive behavior: all commands must run unattended.
- Picker or fzf behavior: none.

## Color And Emphasis

M1.5 may use the plain text style already used by `hindi doctor`. Color is
optional; stable wording matters more than styling for this spec.

## UX Review Notes

The command is intentionally under `source ids`, not `sentences`, because it
touches both sentence and word source YAML. It is a one-off migration helper,
not part of the normal learner workflow.
