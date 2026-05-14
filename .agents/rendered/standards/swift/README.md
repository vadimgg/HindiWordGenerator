# Swift Standard

Shared Swift and iOS rules for Swift engineers, Swift reviewers, and managers
reviewing Swift scope or standards drift.

Load this file first, then load focused rule files that match the task.

## High-Priority Rules

| Rule | Use when |
|---|---|
| [Architecture Boundaries](rules/architecture-boundaries.md) | choosing modules, features, services, view models, or persistence boundaries |
| [Domain Types](rules/domain-types.md) | identifiers, routes, state, persistence keys, dates, and money-like values |
| [State And Data Flow](rules/state-data-flow.md) | SwiftUI state, caches, stores, repositories, and generated views |
| [Concurrency](rules/concurrency.md) | async/await, actors, tasks, cancellation, MainActor, background work |
| [Error Handling](rules/error-handling.md) | throws, Result, user-facing errors, recovery, logging |
| [Testing](rules/testing.md) | behavior changes, view models, services, persistence, async code |
| [UI Composition](rules/ui-composition.md) | SwiftUI/UIKit screens, reusable views, navigation, user feedback |
| [Animations](rules/animations.md) | animated transitions, scroll behavior, glow effects, ripples, visual state models, design tokens |

## Default Guidance

- Prefer typed domain values over raw strings.
- Keep views declarative and thin.
- Put behavior in view models, domain services, reducers, stores, or owning
  modules.
- Keep persistence and network access behind explicit boundaries.
- Protect mutable state behind domain methods; do not expose internal
  dictionaries, arrays, caches, or stores for direct mutation.
- Keep async work cancellable and clear about actor isolation.
- Add focused tests for behavior changes.
- Use comments for intent, effects, invariants, and data surfaces when the code
  has non-obvious architecture or state implications.
