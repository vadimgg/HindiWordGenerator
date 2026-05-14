# File And Function Size

Size is a review trigger, not an automatic failure.

- Target functions: one comfortable screen, roughly 40 lines.
- Target files: roughly 200 lines when practical.
- Files around 300 lines need either a split plan or a short reason to stay
  together.
- Files over 500 lines are refactor candidates unless they are generated,
  data/config, or one cohesive algorithm with a documented reason.
- Keep nesting to two levels when practical.

Existing large files should not be churned just to satisfy the threshold. When
touching a large file, prefer extracting a coherent ownership slice if the task
already creates a natural boundary.
