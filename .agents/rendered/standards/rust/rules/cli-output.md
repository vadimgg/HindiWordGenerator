# CLI Output

CLI output should be explicit, calm, and actionable.

## Applies When

- adding or changing command output
- writing errors
- writing help text

## Rule

Output should include:

1. confirmation: what happened
2. warnings when needed
3. next steps with concrete commands

Errors should explain:

- what failed
- why it failed when known
- what the user should do next

## Good

```text
Created change 005-parser-cleanup.

Next steps:
tool change ready
```
