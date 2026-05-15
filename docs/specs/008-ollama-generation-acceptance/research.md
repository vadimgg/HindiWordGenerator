# Research

## Purpose

This file is the file-by-file audit log for architecture, cleanup, data-drift,
or code-quality findings.

Use it while reading code. Add findings under the file where they were noticed,
even when the same theme appears in multiple files. Keep backlog items for
confirmed actionable work; keep this file for richer evidence and review
context.

If this spec does not need research, write:

```text
Not used in this spec.
```

## Status Values

- `candidate`: noticed during review, needs confirmation.
- `confirmed`: enough evidence to plan work.
- `deferred`: real issue, not part of the current slice.
- `fixed`: handled in this spec.
- `not-a-problem`: investigated and intentionally left alone.

## Files

### `path/to/file.rs`

#### R001 - Short Finding Name

Status: candidate  
Kind: improvement  
Backlog: none  
Confidence: medium

What we saw:
- Describe the evidence in this file.

Why it matters:
- Explain the architecture, data-drift, user, test, or maintenance risk.

Recommended action:
- Say whether to fix now, defer to backlog, or investigate further.

## Data Drift Themes Caught

- Add cross-file drift themes here once several file findings point at the same
  risk.

## Research Decisions

- Record decisions made while reviewing this research file.
