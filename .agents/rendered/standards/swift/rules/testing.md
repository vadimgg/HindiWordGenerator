# Testing

Behavior changes need focused tests.

## Applies When

- changing view models, services, stores, parsing, persistence, networking,
  routing, or async behavior

## Rule

- Add focused tests for behavior changes.
- Prefer tests around view models, reducers, stores, services, and adapters.
- Avoid brittle UI snapshot tests as the only coverage for behavior.
- Use dependency injection for clocks, UUIDs, persistence, network clients, and
  schedulers when behavior depends on them.
- Test async success, failure, cancellation, and out-of-order responses when
  relevant.

## Validation

Use the project’s validation command, commonly one of:

```text
swift test
xcodebuild test -scheme <Scheme> -destination <Destination>
```
