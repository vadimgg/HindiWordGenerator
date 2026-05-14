# Archive

This folder keeps the previous Python implementation and Python-era project
docs available during the Rust migration.

Nothing here is the active implementation target unless the user explicitly
asks to inspect, compare, or modify archived behavior.

## Contents

```text
archive/
  agents/             # Archived Python-specific agent packs and standards
  docs/               # Archived Python-era architecture/planning docs
  python/
    runtime/          # Previous Python CLI and pipeline modules
    scripts/          # Previous helper and experiment scripts
    tests/            # Previous Python contract tests
    experiments/      # Previous local-model experiment scripts/results
    manifest.json     # Previous Python manifest metadata
```

## Reference Use

Use the archive to compare Rust behavior against the old working pipeline:

```bash
uv run archive/python/runtime/main.py check --type sentences --max-batches 1
python3 archive/python/scripts/check-python-contracts.py
```

The archived Python implementation now reads the active YAML source format from
the project-level `input/` directory. Legacy source files were moved to
`archive/python/legacy-input/` only so the migration is auditable.
