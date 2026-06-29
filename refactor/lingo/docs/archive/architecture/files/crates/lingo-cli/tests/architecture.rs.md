# `crates/lingo-cli/tests/architecture.rs`

> **Target kind:** Integration test  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Enforces allowed and forbidden Cargo dependency edges so architectural drift fails in CI.

## Scope: this file owns

- workspace package graph assertions
- forbidden-edge table

## Out of scope: this file must not own

- functional behavior tests
- source-code regex checks as the primary proof

## Allowed dependencies

- cargo_metadata

## Forbidden dependencies and shortcuts

- network calls

## Key implementation shape

```rust
#[test]
fn inward_dependencies_are_enforced() {
    let graph = WorkspaceGraph::load();
    graph.assert_no_edge("lingo-domain", ANY_WORKSPACE_CRATE);
    graph.assert_no_edge("lingo-application", "lingo-workspace-fs");
    graph.assert_no_edge("lingo-workspace-fs", "lingo-prompt");
    graph.assert_only_composition_root_depends_on_all_adapters("lingo-cli");
}
```

## Required tests / evidence

- all forbidden arrows from ARCHITECTURE are encoded
- test fails with a useful edge diff

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
