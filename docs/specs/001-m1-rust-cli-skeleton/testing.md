# Testing

## Drift This Must Prevent

- `hindi doctor` writes or repairs project data.
- Help output exposes `hindi sentences plan` before M2.
- Missing optional config fails the command.
- Missing required paths are reported as success.
- Ollama reachability accidentally loads a model.
- Root discovery works only from the repository root.

## Coverage Summary

| Change | Risk | Test | Purpose |
|---|---|---|---|
| Project-root discovery | Command cannot run from child directories. | Unit test with temp project and child cwd/path input. | Proves discovery is not hard-coded. |
| Required path checks | Doctor hides missing setup. | Unit test missing required paths. | Proves non-zero result and no auto-create. |
| Optional config check | M1 blocks before config exists. | Unit test missing `hindi.toml`. | Proves config is informational. |
| CLI command surface | Future command leaks into M1. | CLI test/help assertion. | Proves only `doctor` is exposed. |
| Ollama check seam | Tests require local Ollama or call a model. | Fake reachability checker. | Proves service status can be tested without network/model calls. |

## Unit Tests

- Project-root discovery finds a temp project from a child directory.
- Project-root discovery fails outside a project.
- Doctor report marks missing optional config without failing.
- Doctor report fails when a required folder is missing.
- Doctor report fails when a required prompt is missing.
- Doctor report can represent Ollama reachable and unreachable states.

## Integration Or CLI Tests

- `cargo run -- doctor` succeeds in this repository when Ollama is reachable and
  required paths exist.
- `cargo run -- --help` shows `doctor`.
- `cargo run -- doctor --help` works.
- `cargo run -- sentences plan` fails because M2 has not exposed that command.

## Drift Checks

Run:

```bash
cargo fmt
cargo test
cargo run -- doctor
python3 archive/python/scripts/check-agent-workflows.py
uv run python archive/python/scripts/check-python-contracts.py
git diff --check
```

## Manual Review Checks

- Doctor output is readable and specific about paths.
- Error output includes a recovery hint.
- No accepted data files changed under `input/`, `output/`, or `audio/`.
- No generation endpoint appears in the Ollama implementation.

## Not Covered

- Real model availability is not covered in M1. It belongs to M4.
- YAML source parsing is not covered in M1. It belongs to M1.5/M2.
- Viewer/export behavior is not covered in M1.
