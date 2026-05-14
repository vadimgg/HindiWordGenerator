# Error Handling

Errors should preserve cause and give the caller a recovery path.

## Applies When

- filesystem, networking, persistence, parsing, permissions, user input, or
  async work can fail

## Rule

- Avoid `try!`, `as!`, and force unwraps in production paths.
- Use typed errors when callers need to branch.
- Use `throws` for recoverable failures.
- Convert technical errors to user-facing messages at the UI boundary.
- Do not swallow errors unless best-effort behavior is explicitly documented.

## Good

```swift
enum NotesError: Error {
    case noteNotFound(NoteID)
    case storageUnavailable
}
```
