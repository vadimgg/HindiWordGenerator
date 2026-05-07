# Single Responsibility

Functions and modules should have one reason to change.

Prefer this shape:

```text
load -> parse -> validate -> transform -> render -> write
```

Split when a function both decides what should happen and performs unrelated
side effects. Do not split when the extracted helper would have a vague name
like `handle_step`, `process_inner`, or `do_work`.
