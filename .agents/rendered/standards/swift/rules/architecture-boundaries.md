# Architecture Boundaries

Keep behavior in the owning layer and keep UI code from becoming the domain.

## Applies When

- adding screens, view models, services, stores, repositories, or app features
- deciding where state, persistence, networking, or navigation belongs
- reviewing a refactor

## Rule

The default path is:

```text
Views -> ViewModels / Feature Models -> Domain Services / Stores -> Adapters
```

- Views render state and send user intent.
- View models or feature models coordinate UI state and user actions.
- Domain services own business rules.
- Stores/repositories own persistence and cache mutation.
- Network, file, database, Keychain, UserDefaults, and OS integrations live
  behind adapters.

Push back when:

- a SwiftUI view owns persistence, networking, or business rules
- a service mutates UI state directly
- a repository exposes raw mutable collections instead of semantic methods
- multiple modules can write the same durable fact
- generated or cached state is treated as authority

## Review Questions

- Which module owns this behavior?
- What must this module never do?
- What is the source of truth?
- What is a cache, projection, or UI snapshot?
- Can two surfaces disagree, and who wins?
