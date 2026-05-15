# CLI And User Messages

## Purpose

Use this file to describe every user-facing command, prompt, help change, log
line, warning, error, and message shape introduced or changed by this spec.
If the user can see it while running the command, it belongs here.

If this spec does not touch CLI or user messages, write:

```text
Not touched in this spec.
```

The CLI UX reviewer should leave feedback here during planning or review. Keep
the examples finished and concrete enough that an implementer can match them
without guessing.

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `brief ...` | Say what the user is trying to do in plain English. | Say what changes in this spec. | List files, events, git calls, prompts, or `None`. |

## Help Text

| Command | Expected Help Change |
|---|---|
| `brief ... --help` | Say what should appear, disappear, or stay unchanged. |

## Success Output

For each changed or added command, show the expected output shape.

```text
What Happened
  Describe the result in one or two direct lines.

Changed Files
  docs/path/that/changed.md

Next
  brief next command
  git add ...
```

## Progress And Log Messages

List any progress, log, or status messages printed while the command runs.
Include messages that appear before success or failure, not only final output.

| Moment | Message | Notes |
|---|---|---|
| Before a long-running step | `Checking ...` | Say whether this should always print or only in verbose mode. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| Missing required input | Say exactly what is required. | Show one valid example command. |
| Operation blocked | Say what stopped the command. | Show the next command or manual fix. |

## Interactive Behavior

- Prompts:
- Non-interactive behavior:
- Picker or fzf behavior:

Write `None` for any row that does not apply. Do not invent interactive flows
for commands that should stay direct.

## Color And Emphasis

| Element | Style | Reason |
|---|---|---|
| Section headings | bold/cyan | Make output easy to scan. |
| Commands | blue | Highlight actions the user can run. |
| Paths | cyan | Make changed files easy to find. |
| Success state | green | Confirm completion. |
| Warning or blocked state | yellow | Show that user action is needed. |
| Error state | red | Show failure clearly. |

## UX Review Notes

_CLI UX reviewer fills this during planning or review._
