# Constants And Magic Values

- Shared paths belong in path constants or path helpers.
- Repeated JSON keys, schema key sets, model defaults, retry limits, batch
  sizes, and output labels should live in one clear place.
- Avoid scattering raw strings for pipeline types, directories, and schema field
  names when a local registry or constant already exists.
- Do not introduce a constant for a value used once unless the name captures a
  real domain decision.
