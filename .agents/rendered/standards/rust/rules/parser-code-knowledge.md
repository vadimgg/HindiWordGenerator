# Parser And Code Knowledge

Keep parser output structured and rendering separate.

## Applies When

- parsing source files
- generating code maps
- producing Markdown or JSON outputs

## Rule

- Keep language-specific parsing behind language-specific implementations.
- Keep rendering separate from parsing.
- Keep structured data as the source of truth.
- Markdown is a rendering, not the internal model.
- Preserve machine-useful data even when the human summary omits it.

## Bad

Parser writes Markdown directly.

## Good

Parser builds a structured index; renderers produce Markdown and JSON.
