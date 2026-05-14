# State And Data Flow

Protect source-of-truth state and make derived state obvious.

## Applies When

- adding `@State`, `@Binding`, `@Observable`, `ObservableObject`, actors,
  stores, repositories, caches, dictionaries, arrays, or generated views

## Rule

- One durable fact should have one owner.
- Derived values should be computed or clearly marked as cache/projection.
- Mutable caches and stores must be private or actor-isolated.
- Expose semantic methods such as `addNote`, `deleteNote(id:)`,
  `refreshIndex`, or `resolveItem(id:)`; do not expose raw mutable storage.
- UI state should not silently override domain or persistence state.

## Drift Smells

- the same status is stored in a model, view model, UserDefaults, and database
- callers mutate `notesByID` directly
- a cached array is used after the source store changed
- view state is saved as durable truth without a domain transition

## Good Shape

```swift
actor NotesStore {
    private var notesByID: [NoteID: Note] = [:]

    func note(id: NoteID) -> Note? { notesByID[id] }
    func deleteNote(id: NoteID) { notesByID.removeValue(forKey: id) }
}
```
