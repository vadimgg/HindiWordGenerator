---
id: architecture-seam-planning
display_name: Architecture Seam Planning
type: skill
version: 0.1.0
description: Use before implementation when a change touches architecture, state, commands, UI flows, persistence, data ownership, or other behavior that needs a clear owning layer.
activation:
  mode: selected_or_inferred
  examples:
    - planning a non-trivial feature
    - defining a work package before implementation
    - reviewing architecture.md before code starts
    - deciding which layer should own a behavior
    - user asks whether a design fits the architecture
---

# Architecture Seam Planning

Use this before implementation when the change could drift across layers or
create a hidden alternate path.

## Mental Model

A seam is the owning boundary the change should exercise.

Good planning names:

- what behavior is changing
- which layer owns it
- which path the behavior should follow
- which tempting shortcut must not be taken
- which facts, files, caches, or stores could drift
- what validation proves the seam still holds

## Workflow

1. Identify the user-visible behavior or engineering outcome.
2. Name the architecture seam being exercised.
3. Write the expected path through the system.
4. Name the layers or files that must not be bypassed.
5. Name persistent data, generated views, caches, and source-of-truth rules.
6. Add drift checks and validation commands.
7. Record the packet in the conversation or the relevant project doc when the
   decision needs to survive the current task.

## Lightweight Packet

Use this shape for lightweight project planning:

```text
Goal:
Architecture Seam:
Expected Path:
Must Not Bypass:
Data / State Involved:
Drift Checks:
Read Scope:
Write Scope:
Validation:
Manual QA:
Stop Conditions:
```

## Review Questions

- Does the change strengthen the intended architecture path?
- Is there one clear owner for the behavior?
- Are commands/views/adapters thin at the boundary?
- Are facts derived from the strongest authority instead of copied into a new
  stored field?
- Can generated views, caches, or human-edited files drift from authority?
- Is mutable state protected behind semantic functions?
- Can the decision logic be tested without UI, filesystem, network, or database
  side effects?

## Must Not

- treat a friendly implementation shortcut as acceptable without naming the
  architecture cost
- let a command, view, adapter, or generated file own domain rules
- hide uncertainty in vague phrases such as "update the state" or "wire it up"
- add a new stored field when the same fact can be derived from stronger
  existing evidence
- skip drift checks for persisted files, caches, generated views, or duplicate
  indexes
