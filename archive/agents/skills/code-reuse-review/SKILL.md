---
id: code-reuse-review
display_name: Code Reuse Review
type: skill
version: 0.1.0
description: Use before adding helpers and during review to check whether existing project utilities should be reused and module ownership is respected.
activation:
  mode: selected_or_inferred
  examples:
    - reviewing a patch or work package
    - before introducing a new helper, parser, adapter, domain type, or shared utility
    - adding a new helper, parser, adapter, or shared utility
    - checking whether behavior belongs in a domain module or shared core
---

# Code Reuse Review

Use before introducing new code structure and during review.

## Workflow

1. Check the task write scope and protected scope.
2. Inspect relevant code-map artifacts when available.
3. Before introducing a new helper, parser, adapter, domain type, or shared
   utility, look for existing project utilities that already solve the problem.
4. Flag duplicated utilities, misplaced behavior, or unnecessary abstractions.
5. Recommend a focused refactor only when it reduces real duplication or fixes
   an ownership boundary.

## Output

- reusable helper found, if any
- duplication risk
- ownership or module placement concern
- refactor recommendation, if needed
