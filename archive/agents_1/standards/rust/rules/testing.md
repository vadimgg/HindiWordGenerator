# Rust Testing

Use this rule when adding Rust behavior or replacing Python behavior.

## Minimum Expectations

- Unit-test source row parsing.
- Unit-test title/subtitle metadata parsing.
- Unit-test sentence schema validation, including missing fields and bad word
  breakdowns.
- Unit-test append-only output path decisions.
- Smoke-test CLI commands that are user-facing.

## Migration Parity

When replacing a Python path, compare Rust and Python on the same small fixture:

- planned pending rows
- skipped rows
- output filenames
- validation pass/fail result
- user-facing summary

If exact output differs, document whether the difference is intentional.
