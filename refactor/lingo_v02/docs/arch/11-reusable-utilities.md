# 11 — Reusable Utilities

There should be no broad `utils` crate. Reusable code must be boring mechanics with clear ownership, tests, and no product policy.

## Time

```rust
pub struct UtcTimestamp(String);

pub trait Clock {
    fn now(&self) -> UtcTimestamp;
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> UtcTimestamp { UtcTimestamp::now_utc() }
}
```

Rules:

- timestamps are UTC ISO-8601 strings at persistence boundaries;
- tests use `FixedClock`;
- services receive `Clock`, they do not call system time directly.

## Content hash

```rust
pub struct ContentHash(String);

impl ContentHash {
    pub fn sha256(bytes: &[u8]) -> Self;
    pub fn as_hex(&self) -> &str;
}
```

Use for:

- reply hashes;
- audio file hashes;
- package integrity;
- canonical fingerprints.

Do not let callers format hashes by hand.

## Canonical fingerprint writer

```rust
pub struct CanonicalFingerprint {
    lines: Vec<String>,
}

impl CanonicalFingerprint {
    pub fn new() -> Self;
    pub fn field(mut self, key: &'static str, value: &str) -> Self;
    pub fn optional_field(mut self, key: &'static str, value: Option<&str>) -> Self;
    pub fn finish(self) -> String;
}
```

Domain owners should wrap this with semantic methods so stable keys do not spread:

```rust
pub struct AudioFingerprintWriter {
    inner: CanonicalFingerprint,
}

impl AudioFingerprintWriter {
    pub fn profile(&mut self, id: &ProfileId);
    pub fn backend(&mut self, backend: AudioBackendId);
    pub fn text(&mut self, text: &AudioInputText);
}
```

## Safe relative path mechanics

Shared parser mechanics are allowed. Domain path values remain distinct.

```rust
pub fn validate_safe_relative_path(raw: &str) -> Result<(), RelativePathError> {
    if raw.is_empty() { return Err(RelativePathError::Empty); }
    if raw.starts_with('/') { return Err(RelativePathError::Absolute); }
    if raw.split('/').any(|part| part == "..") { return Err(RelativePathError::ParentSegment); }
    if raw.contains('\\') { return Err(RelativePathError::Backslash); }
    Ok(())
}
```

## Atomic write

```rust
pub fn atomic_write_verified(path: &Path, bytes: &[u8]) -> Result<(), AtomicWriteError> {
    ensure_parent_exists(path)?;
    let tmp = temp_sibling(path);
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    let read_back = std::fs::read(path)?;
    if read_back != bytes {
        return Err(AtomicWriteError::VerificationFailed);
    }
    Ok(())
}
```

This is mechanics. It must not know about decks, runs, prompts, or audio policy.

## Code-fence extraction

```rust
pub fn extract_single_fence(text: &str, kind: FenceKind) -> Result<&str, FenceError>;
```

This function only extracts a single `json` or `yaml` fence. It does not parse Lingo reply semantics.

## Closed-set metadata pattern

Each closed enum owns wire names and display labels.

```rust
pub trait WireName {
    fn wire_name(self) -> &'static str;
}
```

Avoid a global string bag. The enum's module owns its vocabulary.

## Glyph policy

ASCII mode is a presentation utility, not domain behavior.

```rust
pub enum GlyphSet {
    Unicode,
    Ascii,
}

impl GlyphSet {
    pub fn success(self) -> &'static str { match self { Self::Unicode => "✓", Self::Ascii => "OK" } }
    pub fn audio(self) -> &'static str { match self { Self::Unicode => "♪", Self::Ascii => "audio" } }
    pub fn arrow(self) -> &'static str { match self { Self::Unicode => "→", Self::Ascii => "->" } }
}
```

Only CLI renderers should use this.

## Typed fixture builders

Test support should construct typed values through real constructors.

```rust
pub enum TestSentence {
    First,
    Second,
}

impl TestSentence {
    pub fn id(self) -> SentenceId {
        match self {
            Self::First => SentenceId::parse("sen-01jx9m7q8v6f2x4k9d3p1r0t5w").unwrap(),
            Self::Second => SentenceId::parse("sen-01jx9m7q8v6f2x4k9d3p1r0t5x").unwrap(),
        }
    }
}
```

Use raw literals only when the literal is the contract under test.

## What not to extract

Do not create shared helpers for:

- command names;
- SQL table names;
- config key values outside config owner;
- prompt stage semantics;
- language-specific normalization;
- one-off formatting;
- behavior that belongs to `Sentence`, `Run`, `AudioAsset`, or a service use case.
