# 01 — Boundary Map

## Allowed dependency arrows

```text
lingo-cli
  -> lingo-service
  -> lingo-sqlite        only in composition root
  -> lingo-workspace     only in composition root
  -> future lingo-audio  only in composition root
  -> future lingo-publish only in composition root

lingo-sqlite
  -> lingo-service ports
  -> lingo-domain

lingo-workspace
  -> lingo-service ports
  -> lingo-domain

lingo-service
  -> lingo-domain

lingo-domain
  -> std / small crates only
```

`lingo-service` owns use-case ports. Concrete adapters implement those ports and are wired by `lingo-cli` or future UI composition roots.

## Forbidden dependencies

```text
lingo-domain  -> clap, rusqlite, filesystem layout, serde_json::Value behavior
lingo-service -> clap, terminal colors, concrete SQLite connection, concrete path layout
lingo-sqlite  -> CLI renderers, prompt templates, TTS providers, Anki details
lingo-workspace -> sentence policy, SQL table names, CLI output
lingo-cli     -> SQL, prompt parsing internals, direct DB mutations
```

## Source-of-truth map

| Concept | Authority | Derived/read views |
|---|---|---|
| Deck metadata | `decks` table | CLI status/list output, package manifest |
| Sentence text/lifecycle/approval/QA | `sentences` table | CLI blocks, study/Anki cards |
| Sentence origin/provenance | `sentences.origin` + source columns | show/import/package reports |
| Field authority | `sentence_field_authority` | task prompts, show output |
| Token breakdown | `sentence_tokens` | words view, package/study export |
| In-flight model claim | `runs` + `run_sentences` | visible `enriching`, `runs ls` |
| Run status | `runs` table | `run.json` mirror, `runs ls` |
| Reply bytes | `runs/<id>/reply.*` until applied | `reply_sha256` after apply |
| Audio provenance | `sentence_audio` + file existence | stale/missing audio report |
| Audio file path | deterministic policy `audio/<sentence-id>.mp3` | CLI/show/publish path strings |
| Published artifacts | `out/` | Regenerated from DB + audio |
| Package source identity | package manifest (`source_library_id`, `package_id`) | import approval policy decisions |
| Config | `config.toml` parsed into typed config | command defaults |

## Reporting ownership

Services return typed reports. CLI renders those reports to terminal text or JSON.

```rust
pub struct StatusReport {
    summary: LibrarySummary,
    deck_rows: DeckStatusRows,
    pending_runs: RunRows,
    terminal: TerminalDirective,
}
```

The report owns facts, not colors or prose. The CLI owns labels, ANSI styles, JSON keys, and exact command strings.

## Composition root

The CLI composition root wires concrete adapters explicitly:

```rust
pub fn build_runtime(root: WorkspaceRoot) -> Result<CliRuntime, CliRuntimeError> {
    let workspace = FsWorkspace::open(root)?;
    let repo = SqliteLibraryRepository::open(workspace.layout().library_db())?;
    let profiles = BuiltinProfileCatalog::default();
    let clock = SystemClock;
    let ids = RandomIdGenerator::default();

    let services = LingoServices::new(ServiceDeps {
        repo: &repo,
        workspace: &workspace,
        profiles: &profiles,
        clock: &clock,
        ids: &ids,
    });

    Ok(CliRuntime { workspace, repo, services })
}
```

No hidden self-registration. Built-in catalogs are explicit and duplicate-checked.

## Boundary evidence required

Every implementation slice that touches a boundary should name evidence:

| Boundary | Evidence |
|---|---|
| service does not import CLI | dependency audit / Cargo tree |
| domain does not import store/workspace | dependency audit |
| repository port is real | fake repository used in service tests |
| SQLite schema authority | round-trip and migration tests |
| run.json is mirror | repair/rederive test |
| audio path is deterministic | deck rename leaves audio path stable test |
| approval invariant | active draft rejected at domain and DB layers |
| origin is durable | run cleanup does not erase sentence origin |
| package round-trip | package preserves approval, QA, authority, tokens, origin |
