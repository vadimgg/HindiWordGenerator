# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi eval input --input <path> --prompt-id <id>` | Test a built-in prompt against YAML input with the currently running local model. | New command. | Writes ignored artifacts under `eval/<run-id>/`. |
| `hindi eval grade --run <eval-folder>` | Prepare and capture a human/agent grading result for an eval run. | New command. | Writes grading artifacts inside that eval run folder. |

## Help Text

```text
Hindi Word Generator

Usage:
  hindi eval input --input <path> --prompt-id <id> [--fields <list>] [--max-items <n>]
  hindi eval grade --run <eval-folder>

Runs built-in prompt templates against YAML input using the one currently
running Ollama model. Writes diagnostics to eval/<run-id>/ and never writes
accepted output.

Options:
  --input <path>       YAML source file.
  --prompt-id <id>     Built-in prompt id, e.g. sentence/register.
  --fields <list>      Comma-separated top-level item fields.
                      Default: id,hindi,romanisation,english
  --max-items <n>      Limit selected items before rendering.
  --run <eval-folder>  Eval run folder to grade.
```

## Success Output

```text
Eval Prompt

Model
  selected   ollama:translategemma:12b
  source     ollama ps

Input
  file       input/sentences/complete_hindi_chapter_02_sentences.yaml
  prompt id  sentence/register
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
  hindi eval grade --run eval/2026-05-15_143012_translategemma_12b_sentence_register
```

## Grade Output

```text
Eval Grade

Run
  folder     eval/2026-05-15_143012_translategemma_12b_sentence_register
  prompt id  sentence/register

Editor
  opened     $EDITOR
  file       grade_response.txt

Result
  parsed     ok
  grade      grade.json

Next
  less eval/2026-05-15_143012_translategemma_12b_sentence_register/summary.txt
```

## Progress And Log Messages

| Moment | Message | Notes |
|---|---|---|
| Before model detection | `checking running Ollama model` | Always print. |
| Before model call | `sending rendered prompt to model` | Always print. |
| After model response | `model response received in <time>` | Always print. |
| Before grade editor | `opening grading prompt in $EDITOR` | Always print. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| No running model | `No Ollama model is currently running.` | `ollama run translategemma:12b` |
| Multiple running models | `More than one Ollama model is running.` | Stop extra models, then rerun. |
| Missing field | `Field "english" is missing from item "0001".` | Change `--fields` or fix source YAML. |
| Template render error | `Could not render prompt template.` | Fix the `.hbs` file. |
| Unknown prompt ID | `Unknown prompt id "sentence/foo".` | List supported prompt IDs. |
| Grade parse error | `Could not parse grader response as YAML or JSON.` | Fix the response file and rerun. |

## Interactive Behavior

- Prompts: `$EDITOR` opens for `hindi eval grade` so the user can copy the
  rendered grading prompt to Claude/ChatGPT and paste the structured response.
- Non-interactive behavior: direct command, exits non-zero on errors.
- Picker or fzf behavior: None.

## Color And Emphasis

Use the existing plain CLI style first. Color can come later if the project
adopts a shared renderer.

## UX Review Notes

Keep output explicit that eval writes only to `eval/`, never accepted `output/`.
