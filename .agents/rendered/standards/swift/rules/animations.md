# Animations

SwiftUI animation must be calm, state-driven, and tuneable. All animation values
live in one place so they can be adjusted without searching through view files.

## Applies When

- adding or changing animated transitions, gestures, scroll behavior, glow
  effects, ripples, background motion, or any visual state transition

## Rules

### Design Tokens

All animation durations, easing curves, spring parameters, glow radii, ripple
values, opacity levels, spacing, and padding values must reference
`DesignTokens.*`. Do not write numeric literals for these values in view files.

- Accept: `withAnimation(DesignTokens.Animation.linger)`
- Reject: `withAnimation(.easeInOut(duration: 0.4))`
- Accept: `.padding(DesignTokens.Spacing.sentenceRow)`
- Reject: `.padding(20)`

### State-Driven

All animations are driven through state transitions. Never mutate visual
properties directly. Scope `withAnimation` to the smallest meaningful state
change, not around unrelated mutations.

Drive linger, focus, menu, and play button animations from the feature model's
state, not from view-local timers or booleans.

### Unified Linger System

When multiple components animate in response to the same linger state, they all
observe the same source — the feature model's visual state value — rather than
each owning an independent timer or boolean.

- Reject: `Timer.publish` in a menu or button component to drive linger motion.
- Reject: A separate `isMenuGlowing: Bool` state variable in the menu view.
- Accept: All linger-connected components reading `viewModel.sentenceVisualState == .linger`.

### Explicit Visual State Models

Visual behavior is represented as named state enums, not bags of unrelated
booleans. Future states must extend the enum instead of adding new flags.

- Reject: `var isLingeringRow: Bool`, `var isFocusedRow: Bool` alongside each
  other.
- Accept: `enum SentenceVisualState { case idle, focused, linger }`.

### Transition-Only Effects

Effects that decorate a state transition — such as tap glow waves — must fire
only when the state actually changes, not on every interaction. Guard against
emitting effects for no-op transitions (e.g., tapping an already-focused
sentence must not emit a glow wave).

### Scroll Behavior

Scrollers that snap to items must use `.scrollTargetBehavior(.viewAligned)` to
let natural momentum decelerate and snap to the nearest item boundary. When a
mid-motion tap must stop the scroll without triggering item selection, resolve
the gesture priority explicitly — do not leave it to hit-test defaults.

### Background Motion

Ambient background animations are standalone components. They observe scroll
position and linger state as inputs but own no app state themselves. Fast
scrolling increases gradient activity; stopped scrolling lets it calm. The
background component must not become a source of truth for any feature state.

### Separate Animation Components

Build each animated piece as a standalone SwiftUI component with a clear
boundary. No component inherits another's internal animation state directly.

## Review Questions

- Do all animation values reference `DesignTokens.*` with no inline numeric
  literals in view files?
- Is every animation driven by a named state change, not a direct property
  mutation?
- Do all linger-connected animations share one linger state source from the
  feature model?
- Are transition-only effects (glow waves, ripples) suppressed on no-op
  interactions?
- Does the word scroller use `.scrollTargetBehavior(.viewAligned)` with explicit
  tap-to-stop gesture handling?
- Is the background a standalone component with no owned feature state?
- Does each animated piece have a clear component boundary?
