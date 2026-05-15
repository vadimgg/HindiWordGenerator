# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi eval --input <path> --prompt <path>` | Test a prompt against YAML input with the currently running local model. | New command. | Writes ignored artifacts under `eval/<run-id>/`. |

## Help Text

```text
Hindi Word Generator

Usage:
  hindi eval --input <path> --prompt <path> [--fields <list>] [--max-items <n>]

Runs a prompt template against YAML input using the one currently running
Ollama model. Writes diagnostics to eval/<run-id>/ and never writes accepted
output.

Options:
  --input <path>      YAML source file.
  --prompt <path>     Handlebars prompt template.
  --fields <list>     Comma-separated top-level item fields.
                     Default: id,hindi,romanisation,english
  --max-items <n>     Limit selected items before rendering.
```

## Success Output

```text
Eval Prompt

Model
  selected   ollama:translategemma:12b
  source     ollama ps

Input
  file       input/sentences/complete_hindi_chapter_02_sentences.yaml
  prompt     prompts/sentences/register.yaml.hbs
  items      2
  fields     id,hindi,romanisation,english

Timing
  render     4ms
  model      12.3s

Output
  folder     eval/2026-05-15_143012_translategemma_12b
  prompt     prompt.txt
  response   response.txt
  result     result.json

Next
  less eval/2026-05-15_143012_translategemma_12b/summary.txt
```

## Progress And Log Messages

| Moment | Message | Notes |
|---|---|---|
| Before model detection | `checking running Ollama model` | Always print. |
| Before model call | `sending rendered prompt to model` | Always print. |
| After model response | `model response received in <time>` | Always print. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| No running model | `No Ollama model is currently running.` | `ollama run translategemma:12b` |
| Multiple running models | `More than one Ollama model is running.` | Stop extra models, then rerun. |
| Missing field | `Field "english" is missing from item "0001".` | Change `--fields` or fix source YAML. |
| Template render error | `Could not render prompt template.` | Fix the `.hbs` file. |

## Interactive Behavior

- Prompts: None.
- Non-interactive behavior: direct command, exits non-zero on errors.
- Picker or fzf behavior: None.

## Color And Emphasis

Use the existing plain CLI style first. Color can come later if the project
adopts a shared renderer.

## UX Review Notes

Keep output explicit that eval writes only to `eval/`, never accepted `output/`.
