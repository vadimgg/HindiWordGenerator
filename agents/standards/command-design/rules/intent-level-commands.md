# Intent-Level Commands

User-facing commands should describe what the user wants to do, not the internal
state transitions needed to do it.

## Mental Model

Prefer project commands that match the user's goal:

```text
uv run main.py check
uv run main.py run --type sentences --max-batches 1
uv run main.py audio
```

Avoid making the happy path expose every internal step:

```text
python3 process.py plan-internal
python3 process.py hash-inputs
python3 process.py split-lines
python3 process.py validate-json
python3 process.py update-manifest
```

The lower-level commands may still exist for recovery, debugging, or advanced
manual control. They should not be the normal path we teach first.

## Design Rules

- Start from the user goal: check pending work, run generation, backfill audio,
  inspect output, or transcribe source audio.
- Let the command run the needed planning, validation, writes, audio enrichment,
  and summaries, then print the next useful action.
- Keep lifecycle phase names out of the main command path when a simpler verb
  captures the intent.
- Prefer one memorable command over a sequence of commands that only exists
  because of internal state machinery.
- Avoid keeping old aliases only for compatibility. If a lower-level command
  remains for recovery, keep it out of the normal help and daily docs.
- Do not hide important decisions. If a command commits, approves, merges, or
  opens a pull request, make that behavior visible in the command name, flags,
  and output. For this project, also be clear when a command spends provider
  tokens, writes output JSON, generates audio, or overwrites test output.
- Keep ownership honest: `main.py` exposes the operator workflow, `process.py`
  owns planning/validation/writes, `generate.py` owns model orchestration, and
  `audio_generator.py` owns audio enrichment.

## Examples

Checking pending work:

```text
uv run main.py check --type sentences --max-batches 1
```

This should mean:

- show what would run
- show what would be skipped or deferred
- show missing audio or schema gaps when relevant
- avoid spending tokens

Running a bounded generation:

```text
uv run main.py run --type words --batch-size 5 --max-batches 1
```

This should mean:

- plan the pending slice
- call the configured provider
- validate JSON before writing
- write output only after validation succeeds
- generate or attach audio as designed
- summarize files written and any failures

Backfilling audio:

```text
uv run main.py audio
```

This should mean:

- scan existing generated output
- create missing MP3 files when possible
- write stable relative `audio` paths back into JSON
- report failures without silently corrupting card content

## Review Questions

- Can the normal user explain the command without knowing internal phase names?
- Does the command match one real user intention?
- Are internal planning, validation, manifest, and audio details handled behind
  the command boundary?
- Is the dangerous part visible, such as commit, approve, merge, push, or pull
  request creation? For this project, is token spend or output overwrite
  visible?
- Does the output say what changed, what did not happen, and what the user
  should do next?
- Are optional capture commands, such as backlog and notes, clearly separate
  from required workflow steps?
