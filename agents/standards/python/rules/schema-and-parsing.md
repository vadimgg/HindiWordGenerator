# Schema And Parsing

- Use structured CSV, JSON, and path handling instead of brittle string slicing
  when the standard library already gives a safe parser.
- Preserve the project input contract: optional `#` title, optional `##`
  subtitle, and content rows shaped as `Hindi (romanisation);English`.
- Treat semicolon as the content delimiter. Commas inside Hindi, romanisation,
  or English are normal content.
- Validate generated JSON before writing it into `output/`.
- Optional fields should be omitted when empty, not written as `null`, empty
  strings, or empty arrays.
- When adding transcript support, keep reference transcript, model transcript,
  corrected transcript, and word timestamps distinct so one surface cannot
  silently overwrite another.
