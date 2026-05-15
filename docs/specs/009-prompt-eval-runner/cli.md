# CLI And User Messages

## Commands Touched

| Command | User Goal | Change | Side Effects |
|---|---|---|---|
| `hindi eval run <prompt-id> <input-yaml>` | Test a built-in prompt against YAML input with the currently running local model. | New command. | Writes ignored artifacts under `eval/<prompt-id>/<run-id>/`. |
| `hindi eval grade <run-id-or-path> [--response <path>]` | Prepare and capture a human/agent grading result for an eval run. | New command. | Writes grading artifacts inside that eval run folder. |

## Help Text

```text
Hindi Word Generator

Usage:
  hindi eval run <prompt-id> <input-yaml> [--fields <list>] [--max-items <n>]
  hindi eval grade <run-id-or-path> [--response <path>]

Examples:
  hindi eval run sentence/register input/sentences/complete_hindi_chapter_02_sentences.yaml --max-items 2
  hindi eval grade sentence/register/2026-05-15_143012_translategemma_12b
  hindi eval grade sentence/register/2026-05-15_143012_translategemma_12b --response /tmp/grade.yaml

Runs built-in prompt templates against YAML input using the one currently
running Ollama model. Writes diagnostics to
eval/<prompt-category>/<prompt-name>/<run-id>/ and never writes accepted output.

Arguments:
  <prompt-id>          Built-in prompt id, e.g. sentence/register.
  <input-yaml>         YAML source file.
  <run-id-or-path>     Eval run folder or prompt-scoped run id to grade.

Options:
  --fields <list>      Comma-separated top-level item fields.
                      Default: id,hindi,romanisation,english
  --max-items <n>      Limit selected items before rendering.
  --response <path>    Optional YAML/JSON grader response file to import instead
                       of opening $EDITOR.

Compatibility aliases:
  hindi eval run --prompt-id <id> --input <path>
  hindi eval grade --run <run-id-or-path>
```

## Success Output

```text
Eval Prompt

Model
  selected   ollama:translategemma:12b
  source     Ollama /api/ps

Input
  file       input/sentences/complete_hindi_chapter_02_sentences.yaml
  prompt id  sentence/register
  items      2
  fields     id,hindi,romanisation,english

Timing
  render     4ms
  model      12.3s
  total      12.4s

Output
  folder     eval/sentence/register/2026-05-15_143012_translategemma_12b
  prompt     prompt.txt
  response   response.txt
  meta       meta.json

Next
  hindi eval grade sentence/register/2026-05-15_143012_translategemma_12b
```

## Grade Output

```text
Eval Grade

Run
  folder     eval/sentence/register/2026-05-15_143012_translategemma_12b
  prompt id  sentence/register

Editor
  opened     grade_packet.md
  response   grade_response.txt

Result
  parsed     ok
  grade      grade.json

Next
  less eval/sentence/register/2026-05-15_143012_translategemma_12b/summary.txt
```

With `--response`, the middle section changes:

```text
Import
  source     /tmp/grade.yaml
  response   grade_response.txt
```

## Progress And Log Messages

| Moment | Message | Notes |
|---|---|---|
| Before model detection | `checking running Ollama model` | Always print. |
| Before model call | `sending rendered prompt to model` | Always print. |
| After model response | `model response received in <time>` | Always print. |
| Before grade editor | `opening grade_packet.md in $EDITOR` | Always print to stderr. |

## Warning And Error Output

| Scenario | Expected Message | Recovery |
|---|---|---|
| No running model | `No Ollama model is currently running.` | `ollama run translategemma:12b` |
| Multiple running models | `More than one Ollama model is running.` | Stop extra models, then rerun. |
| Missing field | `Field "english" is missing from item "0001".` | Change `--fields` or fix source YAML. |
| Template render error | `Could not render prompt template.` | Fix the `.hbs` file. |
| Unknown prompt ID | `Unknown prompt id "sentence/foo".` | List supported prompt IDs. |
| Missing run | `Eval run not found: sentence/register/missing.` | Run `hindi eval run` first or pass a valid `eval/...` path. |
| Missing meta | `Eval run is missing meta.json.` | The run folder is incomplete; rerun eval. |
| Missing paired grading template | `Prompt id "sentence/register" has no grading template.` | Fix built-in prompt registration. |
| Missing editor | `$EDITOR is not set.` | Set `EDITOR` or use the future non-interactive import path. |
| Grade parse error | `Could not parse grader response as YAML or JSON.` | Fix the response file and rerun. |

## Interactive Behavior

- Prompts: `hindi eval grade` writes `grade_prompt.txt` and opens
  `grade_packet.md` in `$EDITOR`. The packet contains the rendered grading
  prompt plus this exact marker for the Claude/ChatGPT response:
  `## Paste Grader Response Below`.
- Import path: `hindi eval grade <run-id-or-path> --response <path>` writes the
  same grading artifacts but reads the grader YAML/JSON from the provided file
  instead of opening `$EDITOR`.
- Non-interactive behavior: direct command, exits non-zero on errors.
- Picker or fzf behavior: None.

## Color And Emphasis

Use the existing plain CLI style first. Color can come later if the project
adopts a shared renderer.

## UX Review Notes

Keep output explicit that eval writes only to `eval/`, never accepted `output/`.
