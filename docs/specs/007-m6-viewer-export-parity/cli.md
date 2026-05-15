# CLI And User Messages

## Purpose

M6 adds the cross-cutting viewer/export commands that make the Rust sentence
pipeline usable after generation and audio are complete.

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi viewer` | Open/serve the preview and interactive export app. | New command. | Starts a long-running Astro dev server. |
| `hindi export --source <title> --topic <subtitle>` | Produce a scripted Anki import artifact for a source/topic. | New command. | Writes a file under `exports/`. |
| `hindi --help` | Discover the full happy path. | Add `viewer` and `export`. | None. |

## Help Text

Top-level usage should include:

```text
Hindi Word Generator

Usage:
  hindi doctor
  hindi source ids check
  hindi source ids migrate [--check]
  hindi sentences plan --max-batches <n>
  hindi sentences generate --max-batches <n>
  hindi sentences audio
  hindi viewer
  hindi export --source <title> --topic <subtitle>
```

Viewer help:

```text
Hindi Word Generator

Usage:
  hindi viewer

Serves the Astro viewer from viewer/ and prints the local URL.
```

Export help:

```text
Hindi Word Generator

Usage:
  hindi export --source <title> --topic <subtitle>

Options:
  --source   Match accepted sentence batch title
  --topic    Match accepted sentence batch subtitle
```

## Success Output

Viewer command:

```text
Viewer

  app        viewer/
  url        http://localhost:4321
  command    npm run dev

Press Ctrl-C to stop the viewer.
```

Export command:

```text
Anki Export

  source          Complete Hindi
  topic           Chapter 02
  sentences       20
  missing audio   0
  artifact        exports/complete_hindi_chapter_02_sentences.tsv

Next
  Import the artifact into Anki or use hindi viewer for interactive export.
```

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| `viewer/package.json` missing | `Viewer app not found: viewer/package.json` | Run from the project root or restore `viewer/`. |
| `npm run dev` exits | Forward the child status and say viewer exited. | Inspect npm output. |
| export source/topic has no matches | Names source/topic and says no accepted sentence cards matched. | Run `hindi sentences plan` or choose a listed title/subtitle. |
| export write fails | Names the artifact path and IO error. | Fix permissions/free space and rerun. |

## Interactive Behavior

- Prompts: None.
- Non-interactive behavior: `hindi export` writes and exits.
- Long-running behavior: `hindi viewer` runs until interrupted.

## UX Review Notes

Keep this boring. The user wants one command to see cards and one command to
produce a file. Avoid exposing Astro implementation details beyond the command
and URL.
