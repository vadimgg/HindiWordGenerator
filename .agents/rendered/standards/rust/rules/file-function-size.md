# File And Function Size

Large files and functions are not automatically wrong, but they should trigger
a split review.

## Applies When

- a function becomes hard to scan
- a file mixes multiple concerns
- a module grows past its original boundary

## Rule

Targets and thresholds:

- functions: one comfortable screen, about 40 lines
- files: about 200 lines when practical
- nesting: maximum two levels

Files around 300 lines need either a split or a short explanation for why the
code should stay together.

Files over 500 lines are refactor candidates unless they are generated,
data/config, or one cohesive algorithm with a documented reason.

Long functions are allowed only when they represent one cohesive algorithm and
splitting would make the logic harder to understand.

## Review Question

Can this be split by domain concept, rendering step, parser step, or side
effect boundary?

Do not split by vague buckets like `utils`, `helpers`, or `common`. Split by
ownership, domain concept, or side-effect boundary.
