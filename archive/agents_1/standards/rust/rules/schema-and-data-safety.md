# Rust Schema And Data Safety

Use this rule for Rust parsing, validation, writes, output migrations, and
transcript-linked sentence work.

## Rules

- Parse source rows with a structured parser, not ad hoc string slicing spread
  across command handlers.
- Validate generated JSON before writing any learner-facing output.
- Reject unknown required-field drift between prompts, schema, viewer, and
  export code.
- Preserve `title` and `subtitle` exactly from the source metadata or documented
  fallback.
- Keep sentence `tokens` and `words` arrays to word entries only; spaces and
  punctuation do not belong there.
- Keep each token aligned with its corresponding `words` entry by `word_id`,
  `hindi`, and `roman`.
- Treat `output/sentences/` and `output/words/` as completed-card authority.
- Accepted sentence cards should carry durable source lineage once Rust
  generation starts: source file, item ID, and content fingerprint.
- Model output should be enrichment-only. Rust copies trusted source fields and
  source lineage from YAML/planner data.
- Do not rewrite existing batch files in normal generation.
- Audio may atomically add missing `audio` metadata, but it must not alter
  learner content without an explicit repair command.
- Any repair/regenerate command must name the target files and require explicit
  user intent.

## Validation Errors

Validation errors should identify:

- output path or run folder
- card type
- batch number or source index
- field path
- actual problem
- whether the file was written
