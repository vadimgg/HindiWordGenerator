# Names

- Use names that describe the result or side effect.
- Boolean helpers should read like questions.
- Side-effect functions should name the effect, such as `write_*`, `save_*`,
  `update_*`, or `attach_*`.
- Avoid vague names like `handle`, `process`, `do_update`, and `get_data` when a
  more specific name is available.
- Name command flags for the user's intent, not the internal mechanism.
