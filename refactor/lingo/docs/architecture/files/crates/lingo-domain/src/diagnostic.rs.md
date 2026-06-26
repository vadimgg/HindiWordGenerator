# `crates/lingo-domain/src/diagnostic.rs`

> **Target kind:** Rust source  
> **Authority:** implementation contract for the clean-slate design  
> [Back to `ARCHITECTURE.md`](../../../../../../ARCHITECTURE.md)

## Responsibility

Owns machine-readable validation diagnostics and severity, while presentation layers own prose formatting and colors.

## Scope: this file owns

- diagnostic code closed set
- severity
- structured field location
- validation report collection

## Out of scope: this file must not own

- ANSI output
- line wrapping
- CLI exit codes
- free-form parsing of error strings

## Allowed dependencies

- IDs and domain field paths

## Forbidden dependencies and shortcuts

- CLI types
- serde_json::Value as an internal model

## Key implementation shape

```rust
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    location: DiagnosticLocation,
    message: String,
}

pub struct ValidationReport {
    diagnostics: Vec<Diagnostic>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.diagnostics.iter().all(|d| d.severity() != Severity::Error)
    }
}
```

## Required tests / evidence

- every diagnostic code has stable wire name and severity
- structured locations are asserted directly, never parsed from message text

## Codex guardrails

- Keep the public function or type names aligned with the responsibility above; do not invent a second owner for the same concept.
- Do not move policy into a helper merely to reduce line count. Extract only a real concept or reusable mechanism.
- Parse raw boundary values into typed values once and carry those values forward.
- Return typed errors/reports; formatting belongs at the CLI or frontend edge.
- Any new dependency or abstraction must pass the deletion test and preserve the dependency arrows in `ARCHITECTURE.md`.
