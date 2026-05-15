# CLI And User Messages

## Purpose

This spec adds one visible command: `hindi sentences audio`. The command should
feel like a safe backfill: it scans accepted sentence batches, creates missing
media, patches missing metadata, and tells the user what changed.

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi sentences audio` | Backfill missing sentence audio for accepted output. | New command. | May write `audio/sentences/**/*.mp3` and patch `output/sentences/*.json`. |
| `hindi sentences --help` | Discover sentence commands. | Add `audio`. | None. |

## Help Text

| Command | Expected Help Change |
|---|---|
| `hindi sentences --help` | Shows `plan`, `generate`, and `audio`. |

Expected shape:

```text
Hindi Word Generator

Usage:
  hindi sentences plan --max-batches <n>
  hindi sentences generate --max-batches <n>
  hindi sentences audio

Commands:
  plan       Preview pending sentence batches without writing output
  generate   Generate pending sentence batches with the configured local model
  audio      Backfill missing audio for accepted sentence batches
```

## Success Output

When work is performed:

```text
Sentence Audio

  scanned batches    4
  scanned cards      20
  generated mp3s     3
  patched cards      3
  skipped existing   17

Generated Audio
  audio/sentences/complete_hindi_chapter_02_sentences_batch_05/01_kya_ap_kamala_ji_hain.mp3

Updated Output
  output/sentences/complete_hindi_chapter_02_sentences_batch_05.json

Next
  hindi viewer
```

When nothing is missing:

```text
Sentence Audio

  scanned batches    4
  scanned cards      20
  generated mp3s     0
  patched cards      0
  skipped existing   20

Nothing to do. Sentence audio is already complete.
```

When no accepted sentence output exists:

```text
Sentence Audio

Problem
  No accepted sentence batches found in output/sentences.

Run
  hindi sentences generate --max-batches 1
```

## Progress And Log Messages

| Moment | Message | Notes |
|---|---|---|
| Before synthesis starts | `Generating missing sentence audio...` | Print only when at least one MP3 is missing. |
| Before JSON patching | `Updating accepted sentence JSON...` | Print only when at least one audio field is missing. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| TTS backend unavailable | `Audio backend not ready` plus backend-specific detail. | Install/configure the backend, then rerun `hindi sentences audio`. |
| Synthesis failed for one card | Names the batch and card index. | Rerun after backend/network issue is fixed; existing MP3s are reused. |
| Batch JSON cannot be parsed | Names the batch path and parse error. | Fix or remove the malformed accepted batch. |
| JSON patch write fails | Names the batch path and leaves existing JSON untouched. | Fix filesystem permissions/space and rerun. |

## Interactive Behavior

- Prompts: None.
- Non-interactive behavior: direct command, exits non-zero on failed synthesis
  or patching.
- Picker or fzf behavior: None.

## Color And Emphasis

Follow existing Rust CLI plain text for M5. Rich color can come later if the
Rust CLI adopts a shared renderer.

## UX Review Notes

The output should avoid implying that audio generation changes sentence content.
Use "patched cards" for metadata updates and "generated MP3s" for media writes.
