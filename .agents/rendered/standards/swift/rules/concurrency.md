# Concurrency

Async code should make ownership, cancellation, and actor isolation clear.

## Applies When

- adding async/await, `Task`, actors, `@MainActor`, Combine bridges, timers, or
  background work

## Rule

- UI mutations happen on `MainActor`.
- Long-running work must be cancellable or explicitly justified.
- Avoid unstructured `Task {}` unless the lifetime is clear.
- Prefer actors or isolated stores for mutable shared state.
- Do not block the main thread with file, network, parsing, or database work.
- Preserve errors unless the best-effort behavior is documented.

## Review Questions

- Who owns this task lifetime?
- What cancels it?
- Which actor owns mutable state?
- Can responses arrive out of order and overwrite newer state?
