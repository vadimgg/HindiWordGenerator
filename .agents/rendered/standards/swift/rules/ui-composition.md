# UI Composition

User-facing SwiftUI/UIKit code should be understandable, accessible, and thin.

## Applies When

- adding screens, components, navigation, empty states, errors, or loading
  states

## Rule

- Views render state and send intent; they do not own business rules.
- Name actions from the user’s point of view.
- Show clear loading, empty, error, and success states.
- Keep accessibility labels, dynamic type, contrast, and hit targets in mind.
- Keep reusable views focused on presentation, not persistence or networking.
- Avoid placing important side effects in view initializers or computed body
  branches.

## Review Questions

- Can the user tell what happened and what to do next?
- Does the UI state come from one owner?
- Are side effects triggered by explicit actions or lifecycle hooks with clear
  ownership?
