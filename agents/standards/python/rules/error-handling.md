# Error Handling

Errors should explain what failed and preserve useful context.

- File, JSON, process, and validation failures should include the relevant path,
  stem, batch, or pipeline type.
- Do not silently swallow errors unless best-effort behavior is documented.
- Expected user-facing failures should produce actionable messages.
- For CLI output, include what failed, why when known, and the next concrete
  command or file to inspect when possible.
- Do not leak API keys, provider credentials, or full environment values in logs
  or error messages.
