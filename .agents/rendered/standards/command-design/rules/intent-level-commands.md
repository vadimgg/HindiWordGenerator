# Intent-Level Commands

User-facing commands should describe what the user wants to do, not the internal
state transitions needed to do it.

## Mental Model

Prefer:

```text
tool change ready
tool task done T01
tool review open
```

Avoid making the happy path expose every internal step:

```text
tool change check
tool change advance plan-ready
tool change check
tool change advance work-ready
removed low-level task lifecycle subcommands
tool change close
tool change merge
```

Removed lower-level commands should not be kept as normal guidance. If a
recovery-only workflow exists, keep it explicit, narrow, and outside the daily
happy path.

## Design Rules

- Start from the user goal: ready a change, finish a task, prepare review.
- Let the command run the checks, update the owning workflow files, refresh
  advisory indexes, and print the next useful action.
- Keep lifecycle phase names out of the main command path when a simpler verb
  captures the intent.
- Treat tasks like a checklist. Users should not have to start a task before
  finishing it if the final command can validate and record the workflow state.
- Prefer one memorable command over a sequence of commands that only exists
  because of internal state machinery.
- Avoid keeping old aliases only for compatibility. If a lower-level recovery
  path remains, keep it out of normal help and daily docs.
- Do not hide important decisions. If a command commits, approves, merges, or
  opens a pull request, make that behavior visible in the command name, flags,
  and output.
- Keep ownership honest: the workflow tool tracks work, git records code
  history, pull-request tooling opens reviews, and the review host owns merge.

## Examples

Spec authoring:

```text
tool change ready
```

This should mean:

- validate the change package
- verify planned work packages from task files
- produce a clean handoff to implementation

Task completion:

```text
tool task done T01
```

This should mean:

- validate the work package exists
- check dependencies and validation rules
- run validation commands
- mark the checklist item done
- refresh advisory indexes
- clearly say whether a git commit was created

Spec completion:

```text
tool review open
```

This should mean:

- run close checks
- write or refresh closeout workflow files when needed
- stop with commit guidance if the worktree is dirty
- push the branch and create or show the pull request when clean
- leave the merge to the review host

## Review Questions

- Can the normal user explain the command without knowing internal phase names?
- Does the command match one real user intention?
- Are internal state changes and advisory indexes handled behind the command
  boundary?
- Is the dangerous part visible, such as commit, approve, merge, push, or pull
  request creation?
- Does the output say what changed, what did not happen, and what the user
  should do next?
- Are optional capture commands, such as backlog and notes, clearly separate
  from required workflow steps?
