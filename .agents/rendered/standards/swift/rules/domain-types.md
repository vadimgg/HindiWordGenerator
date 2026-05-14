# Domain Types

Use typed values for concepts with structure, validation, or multiple
representations.

## Applies When

- ids, slugs, routes, tabs, URLs, dates, durations, status, user settings,
  persistence keys, or enum-like strings cross a boundary

## Rule

- Prefer `struct` wrappers or enums for meaningful identifiers and states.
- Avoid passing raw `String` or `Int` when the value has rules.
- Validate at boundaries and pass validated values inward.
- Keep persistence keys and route names in one place.

## Bad

```swift
func loadNote(id: String) async throws -> Note
```

## Good

```swift
struct NoteID: Hashable, Codable {
    let rawValue: String
}

func loadNote(id: NoteID) async throws -> Note
```
