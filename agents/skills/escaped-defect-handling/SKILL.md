---
id: escaped-defect-handling
display_name: Escaped Defect Handling
type: skill
version: 0.1.0
description: Use when the user reports a bug after implementation, review, merge, or normal use, so the fix records cause, guardrail, and similar surfaces checked.
activation:
  mode: selected_or_inferred
  examples:
    - user reports a bug after a task was marked done
    - fixing a regression found during manual QA
    - reviewing a fix for a previously escaped defect
    - bug was missed by tests or prior review
---

# Escaped Defect Handling

Use this when a bug escaped implementation, review, or normal validation. The
goal is not blame. The goal is to turn the failure into a reusable guardrail.

## Workflow

1. State the user-visible failure in plain language.
2. Identify the likely failed assumption or boundary before editing.
3. Check for similar surfaces where the same assumption may exist.
4. Fix the smallest owned path.
5. Add a guardrail when practical: test, stronger type, explicit lookup,
   validation, comment, standard, or documentation note.
6. Report what changed and what the user should retest.
7. Include the escaped-defect note in the response or project notes. If the
   guardrail remains out of scope, record a backlog item.

## Note Shape

```text
Issue:
Cause:
Fix:
Guardrail Added:
Similar Surfaces Checked:
What To Test:
```

## Review Shape

```text
Escaped Defect Review:
  Issue:
  Cause:
  Missing Guardrail:
  Standard / Test To Add:
  Similar Surfaces Checked:
```

## Common Failure Patterns

- generic lookup found the wrong target
- UI event handling fell through after a surface consumed the event
- command execution bypassed the shared executor or domain path
- generated view or cache was treated as authority
- stored state duplicated a derivable fact and drifted
- raw platform object leaked past an adapter boundary
- async response arrived after state had changed
- fix changed the symptom but not the failed assumption

HindiWordGenerator-specific:

- a CSV source row was malformed but generation made it look polished
- `manifest.json` implied completion while output batches were missing
- audio existed on disk but the JSON `audio` path was stale or missing
- viewer behavior hid a schema gap that `check` should have reported
- a prompt change and `process.py` validation drifted apart

## Must Not

- fix only the symptom without naming the cause
- skip similar-surface checks when the bug came from duplicated logic or generic
  lookup
- add low-signal comments that restate obvious code instead of documenting the
  failed assumption
- treat missing validation as acceptable without saying what manual QA should
  cover
- close the issue silently when the guardrail remains missing
