# 006 - M5 Audio Parity

This spec adds the Rust sentence audio backfill command:

```bash
hindi sentences audio
```

The command scans accepted sentence JSON, creates missing MP3s, and patches only
missing `audio` metadata. It keeps generation append-only and preserves the
viewer/export audio path contract.

Read in order:

1. [spec.md](spec.md)
2. [architecture.md](architecture.md)
3. [cli.md](cli.md)
4. [plan.md](plan.md)
5. [testing.md](testing.md)
6. [tasks.md](tasks.md)
