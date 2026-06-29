use lingo_application::ports::*;
use lingo_domain::*;
use rusqlite::{Connection, OptionalExtension, Row, params};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct SqliteLibraryStore {
    path: PathBuf,
}

impl SqliteLibraryStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, LibraryFailure> {
        let path = path.into();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| LibraryFailure::Io(error.to_string()))?;
        }
        let store = Self { path };
        let connection = store.connection()?;
        migrate(&connection)?;
        Ok(store)
    }

    pub fn path(&self) -> &Path { &self.path }

    fn connection(&self) -> Result<Connection, LibraryFailure> {
        let connection = Connection::open(&self.path).map_err(sql)?;
        connection.execute_batch("PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL;").map_err(sql)?;
        Ok(connection)
    }
}

impl LibraryStore for SqliteLibraryStore {
    fn summary(&self) -> Result<LibrarySummary, LibraryFailure> {
        let connection = self.connection()?;
        let collections = scalar_usize(&connection, "SELECT COUNT(*) FROM collections")?;
        let sentences = scalar_usize(&connection, "SELECT COUNT(*) FROM sentences")?;
        let draft = scalar_usize(&connection, "SELECT COUNT(*) FROM sentences WHERE status='draft'")?;
        let enriching = scalar_usize(&connection, "SELECT COUNT(*) FROM sentences WHERE status='enriching'")?;
        let enriched = scalar_usize(&connection, "SELECT COUNT(*) FROM sentences WHERE status='enriched'")?;
        let words = scalar_usize(&connection, "SELECT COUNT(*) FROM words")?;
        let audio = scalar_usize(&connection, "SELECT COUNT(*) FROM sentence_audio")?;
        Ok(LibrarySummary { collections, sentences, draft, enriching, enriched, words, audio })
    }

    fn list_sentences(&self, query: &SentenceQuery) -> Result<SentencePage, LibraryFailure> {
        let connection = self.connection()?;
        let ids = select_ids(&connection, query)?;
        let total = ids.len();
        let mut sentences = Vec::with_capacity(ids.len());
        for id in ids {
            sentences.push(load_sentence(&connection, &id)?);
        }
        Ok(SentencePage { sentences, total })
    }

    fn get_sentence(&self, id: &SentenceId) -> Result<Sentence, LibraryFailure> {
        let connection = self.connection()?;
        load_sentence(&connection, id)
    }

    fn insert_drafts(&self, request: InsertDrafts) -> Result<CommitReport, LibraryFailure> {
        if request.drafts.is_empty() { return Ok(CommitReport { created: 0, updated: 0, ids: Vec::new() }); }
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        upsert_collection(&tx, &request.collection, &request.title, &request.language)?;
        let mut next_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(ord),0)+1 FROM sentences WHERE collection_id=?1",
            params![request.collection.as_str()],
            |row| row.get(0),
        ).map_err(sql)?;
        let mut ids = Vec::with_capacity(request.drafts.len());
        for draft in request.drafts {
            let id = SentenceId::generate();
            let order = SentenceOrder::new(next_order).map_err(invalid)?;
            next_order += 1;
            let sentence = Sentence::draft(DraftSentenceInput {
                id: id.clone(),
                collection: request.collection.clone(),
                batch_id: request.batch_id.clone(),
                title: request.sentence_title.clone(),
                section: request.section.clone(),
                order,
                target: draft.target,
                romanisation: draft.romanisation,
                english: draft.english,
                authority: draft.authority,
                tags: draft.tags,
                provenance: draft.provenance,
            }).map_err(invalid)?;
            insert_sentence_row(&tx, &sentence)?;
            ids.push(id);
        }
        tx.commit().map_err(sql)?;
        Ok(CommitReport { created: ids.len(), updated: 0, ids })
    }

    fn import_sentences(&self, request: ImportSentences) -> Result<CommitReport, LibraryFailure> {
        if request.sentences.is_empty() { return Ok(CommitReport { created: 0, updated: 0, ids: Vec::new() }); }
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        upsert_collection(&tx, &request.collection, &request.title, &request.language)?;
        let mut next_order: i64 = tx.query_row(
            "SELECT COALESCE(MAX(ord),0)+1 FROM sentences WHERE collection_id=?1",
            params![request.collection.as_str()],
            |row| row.get(0),
        ).map_err(sql)?;
        let mut ids = Vec::with_capacity(request.sentences.len());
        for input in request.sentences {
            let id = SentenceId::generate();
            let order = SentenceOrder::new(next_order).map_err(invalid)?;
            next_order += 1;
            let sentence = Sentence::from_record(SentenceRecordInput {
                id: id.clone(),
                collection: request.collection.clone(),
                batch_id: input.batch_id,
                title: input.title,
                section: input.section,
                order,
                target: input.target,
                romanisation: input.romanisation,
                english: input.english,
                literal: input.literal,
                register: input.register,
                authority: input.authority,
                status: input.status,
                enrich_run: None,
                provenance: input.provenance,
                tags: input.tags,
                breakdown: input.breakdown,
                audio: None,
            });
            insert_sentence_row(&tx, &sentence)?;
            rebuild_word_projection(&tx, &sentence)?;
            ids.push(id);
        }
        tx.commit().map_err(sql)?;
        Ok(CommitReport { created: ids.len(), updated: 0, ids })
    }

    fn claim_for_enrichment(&self, request: ClaimEnrichment) -> Result<EnrichmentRun, LibraryFailure> {
        if request.limit == 0 { return Ok(EnrichmentRun { run_id: request.run_id, sentences: Vec::new() }); }
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        let force = request.force;
        // An explicit selection (the user hand-picked these ids, or a whole batch)
        // may re-claim sentences stuck in 'enriching' from an abandoned run; a bulk
        // `All` claim stays draft-only so it never re-enriches everything.
        let explicit = matches!(request.selection, SentenceSelection::Ids(_) | SentenceSelection::Batch(_));
        let selection = if force {
            request.selection.clone()
        } else {
            match request.selection {
                SentenceSelection::All => SentenceSelection::Status(SentenceStatus::Draft),
                other => other,
            }
        };
        let query = SentenceQuery { selection, sort: SentenceSort::LibraryOrder, page: PageRequest { limit: request.limit, offset: 0 } };
        let candidate_ids = select_ids(&tx, &query)?;
        let mut claimed_ids = Vec::with_capacity(candidate_ids.len());
        for id in &candidate_ids {
            // `active`/`enriched` are never re-claimed without --force (protects
            // confirmed work and reviewed results).
            let affected = if force {
                tx.execute(
                    "UPDATE sentences SET status='enriching', enrich_run=?2, updated_at=?3 WHERE id=?1",
                    params![id.as_str(), request.run_id.as_str(), now_utc()],
                ).map_err(sql)?
            } else if explicit {
                tx.execute(
                    "UPDATE sentences SET status='enriching', enrich_run=?2, updated_at=?3 WHERE id=?1 AND status IN ('draft','enriching')",
                    params![id.as_str(), request.run_id.as_str(), now_utc()],
                ).map_err(sql)?
            } else {
                tx.execute(
                    "UPDATE sentences SET status='enriching', enrich_run=?2, updated_at=?3 WHERE id=?1 AND status='draft'",
                    params![id.as_str(), request.run_id.as_str(), now_utc()],
                ).map_err(sql)?
            };
            if affected > 0 { claimed_ids.push(id.clone()); }
        }
        let mut sentences = Vec::with_capacity(claimed_ids.len());
        for id in &claimed_ids { sentences.push(load_sentence(&tx, id)?); }
        tx.commit().map_err(sql)?;
        Ok(EnrichmentRun { run_id: request.run_id, sentences })
    }

    fn apply_enrichment(&self, request: ApplyEnrichmentCommit) -> Result<CommitReport, LibraryFailure> {
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        let expected_ids = ids_for_run(&tx, &request.run_id)?;
        let provided = request.enrichments.iter().map(|e| e.id.clone()).collect::<Vec<_>>();
        if expected_ids != provided {
            return Err(LibraryFailure::Invalid(format!(
                "reply id set does not match claimed run {}; expected {:?}, found {:?}",
                request.run_id, expected_ids, provided
            )));
        }
        let mut updated = Vec::with_capacity(request.enrichments.len());
        for enrichment in request.enrichments {
            let mut sentence = load_sentence(&tx, &enrichment.id)?;
            if sentence.enrich_run() != Some(&request.run_id) {
                return Err(LibraryFailure::Invalid(format!("sentence {} is not owned by run {}", sentence.id(), request.run_id)));
            }
            sentence.apply_enrichment(enrichment).map_err(invalid)?;
            update_sentence_row(&tx, &sentence)?;
            rebuild_word_projection(&tx, &sentence)?;
            updated.push(sentence.id().clone());
        }
        tx.commit().map_err(sql)?;
        Ok(CommitReport { created: 0, updated: updated.len(), ids: updated })
    }

    fn reset_enrichment_claim(&self, request: ResetEnrichmentClaim) -> Result<ResetReport, LibraryFailure> {
        let connection = self.connection()?;
        let count = match request.run_id {
            Some(run) => connection.execute(
                "UPDATE sentences SET status='draft', enrich_run=NULL, updated_at=?2 WHERE status='enriching' AND enrich_run=?1",
                params![run.as_str(), now_utc()],
            ).map_err(sql)?,
            None => connection.execute(
                "UPDATE sentences SET status='draft', enrich_run=NULL, updated_at=?1 WHERE status='enriching'",
                params![now_utc()],
            ).map_err(sql)?,
        };
        Ok(ResetReport { reset: count })
    }

    fn reorder(&self, request: ReorderSentences) -> Result<ReorderReport, LibraryFailure> {
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        for (index, id) in request.ordered_ids.iter().enumerate() {
            let parked = -((index as i64) + 1);
            tx.execute(
                "UPDATE sentences SET ord=?3, updated_at=?4 WHERE collection_id=?1 AND id=?2",
                params![request.collection.as_str(), id.as_str(), parked, now_utc()],
            ).map_err(sql)?;
        }
        for (index, id) in request.ordered_ids.iter().enumerate() {
            let final_order = (index as i64) + 1;
            tx.execute(
                "UPDATE sentences SET ord=?3, updated_at=?4 WHERE collection_id=?1 AND id=?2",
                params![request.collection.as_str(), id.as_str(), final_order, now_utc()],
            ).map_err(sql)?;
        }
        tx.commit().map_err(sql)?;
        Ok(ReorderReport { moved: request.ordered_ids.len() })
    }

    fn update_sentence(&self, request: UpdateSentence) -> Result<SentenceReport, LibraryFailure> {
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        let mut sentence = load_sentence(&tx, &request.id)?;
        if let Some(section) = request.section { sentence.set_section(section); }
        let mut authority = sentence.authority().clone();
        if let Some(target) = request.target { update_text_field(&tx, sentence.id(), "target", Some(target.as_str()))?; authority.mark(SentenceField::Target, FieldAuthority::Human); }
        if let Some(romanisation) = request.romanisation.as_ref() { update_text_field(&tx, sentence.id(), "romanisation", romanisation.as_ref().map(Romanisation::as_str))?; authority.mark(SentenceField::Romanisation, FieldAuthority::Human); }
        if let Some(english) = request.english.as_ref() { update_text_field(&tx, sentence.id(), "english", english.as_ref().map(Gloss::as_str))?; authority.mark(SentenceField::English, FieldAuthority::Human); }
        if let Some(literal) = request.literal.as_ref() { update_text_field(&tx, sentence.id(), "literal", literal.as_ref().map(LiteralGloss::as_str))?; authority.mark(SentenceField::Literal, FieldAuthority::Human); }
        if let Some(register) = request.register { update_text_field(&tx, sentence.id(), "register", register.map(|r| r.wire_name()))?; authority.mark(SentenceField::Register, FieldAuthority::Human); }
        if let Some(tags) = request.tags { tx.execute("UPDATE sentences SET tags=?2, updated_at=?3 WHERE id=?1", params![request.id.as_str(), json(&tags)?, now_utc()]).map_err(sql)?; }
        if let Some(status) = request.status { tx.execute("UPDATE sentences SET status=?2, updated_at=?3 WHERE id=?1", params![request.id.as_str(), status.wire_name(), now_utc()]).map_err(sql)?; }
        if let Some(batch_id) = request.batch_id { tx.execute("UPDATE sentences SET batch_id=?2, updated_at=?3 WHERE id=?1", params![request.id.as_str(), batch_id.as_ref().map(BatchId::as_str), now_utc()]).map_err(sql)?; }
        if let Some(title) = request.title { tx.execute("UPDATE sentences SET title=?2, updated_at=?3 WHERE id=?1", params![request.id.as_str(), title.as_ref().map(SectionName::as_str), now_utc()]).map_err(sql)?; }
        tx.execute(
            "UPDATE sentences SET section=?2, authority=?3, updated_at=?4 WHERE id=?1",
            params![request.id.as_str(), sentence.section().map(SectionName::as_str), json(&authority)?, now_utc()],
        ).map_err(sql)?;
        let updated = load_sentence(&tx, &request.id)?;
        tx.commit().map_err(sql)?;
        Ok(SentenceReport { sentence: updated })
    }

    fn create_batch(&self, request: CreateBatch) -> Result<BatchReport, LibraryFailure> {
        let mut connection = self.connection()?;
        let tx = connection.transaction().map_err(sql)?;
        upsert_collection(&tx, &request.collection, &request.title, &request.language)?;
        let id = BatchId::generate();
        tx.execute(
            "INSERT INTO batches(id,collection_id,name,default_title,default_subtitle,raw_text,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?7)",
            params![
                id.as_str(),
                request.collection.as_str(),
                request.name.as_str(),
                request.default_title.as_ref().map(SectionName::as_str),
                request.default_subtitle.as_ref().map(SectionName::as_str),
                request.raw_text,
                now_utc(),
            ],
        ).map_err(sql)?;
        let batch = load_batch(&tx, &id)?;
        tx.commit().map_err(sql)?;
        Ok(BatchReport { batch })
    }

    fn append_batch_raw_text(&self, request: AppendBatchRawText) -> Result<BatchReport, LibraryFailure> {
        let connection = self.connection()?;
        let affected = connection.execute(
            "UPDATE batches SET raw_text = CASE WHEN raw_text='' THEN ?2 ELSE raw_text || char(10) || char(10) || ?2 END, updated_at=?3 WHERE id=?1",
            params![request.batch_id.as_str(), request.raw_text, now_utc()],
        ).map_err(sql)?;
        if affected == 0 { return Err(LibraryFailure::NotFound(request.batch_id.to_string())); }
        Ok(BatchReport { batch: load_batch(&connection, &request.batch_id)? })
    }

    fn list_batches(&self, collection: Option<&CollectionId>) -> Result<BatchPage, LibraryFailure> {
        let connection = self.connection()?;
        let base = "SELECT b.id,b.collection_id,b.name,b.default_title,b.default_subtitle,b.raw_text,b.created_at,b.updated_at,
                COALESCE(SUM(s.status='draft'),0),COALESCE(SUM(s.status='enriching'),0),
                COALESCE(SUM(s.status='enriched'),0),COALESCE(SUM(s.status='active'),0),COUNT(s.id)
             FROM batches b LEFT JOIN sentences s ON s.batch_id=b.id";
        let sql_text = match collection {
            Some(_) => format!("{base} WHERE b.collection_id=?1 GROUP BY b.id ORDER BY b.created_at"),
            None => format!("{base} GROUP BY b.id ORDER BY b.created_at"),
        };
        let mut statement = connection.prepare(&sql_text).map_err(sql)?;
        let map_row = |row: &Row<'_>| -> rusqlite::Result<BatchSummary> {
            Ok(BatchSummary {
                batch: batch_from_row(row)?,
                draft: row.get::<_, i64>(8)? as usize,
                enriching: row.get::<_, i64>(9)? as usize,
                enriched: row.get::<_, i64>(10)? as usize,
                active: row.get::<_, i64>(11)? as usize,
                total: row.get::<_, i64>(12)? as usize,
            })
        };
        let rows = match collection {
            Some(id) => statement.query_map(params![id.as_str()], map_row).map_err(sql)?.collect::<Result<Vec<_>, _>>(),
            None => statement.query_map([], map_row).map_err(sql)?.collect::<Result<Vec<_>, _>>(),
        }.map_err(sql)?;
        Ok(BatchPage { batches: rows })
    }

    fn get_batch(&self, id: &BatchId) -> Result<Batch, LibraryFailure> {
        let connection = self.connection()?;
        load_batch(&connection, id)
    }

    fn set_audio(&self, sentence: &SentenceId, audio: SentenceAudio) -> Result<(), LibraryFailure> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO sentence_audio(sentence_id,path,hash,backend,format,voice,model,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(sentence_id) DO UPDATE SET path=excluded.path, hash=excluded.hash,
                backend=excluded.backend, format=excluded.format, voice=excluded.voice,
                model=excluded.model, created_at=excluded.created_at",
            params![
                sentence.as_str(),
                audio.path().as_str(),
                audio.hash().as_str(),
                audio.backend().wire_name(),
                audio.format().wire_name(),
                audio.voice(),
                audio.model(),
                now_utc(),
            ],
        ).map_err(sql)?;
        Ok(())
    }

    fn list_words(&self, query: &WordQuery) -> Result<WordPage, LibraryFailure> {
        let connection = self.connection()?;
        let sql_text = match &query.collection {
            Some(_) => "SELECT w.id,w.collection_id,w.key,w.form,w.roman,w.kind,COUNT(DISTINCT sw.sentence_id) AS c FROM words w LEFT JOIN sentence_words sw ON sw.word_id=w.id WHERE w.collection_id=?1 GROUP BY w.id HAVING c>=?2 ORDER BY c DESC,w.form LIMIT ?3 OFFSET ?4",
            None => "SELECT w.id,w.collection_id,w.key,w.form,w.roman,w.kind,COUNT(DISTINCT sw.sentence_id) AS c FROM words w LEFT JOIN sentence_words sw ON sw.word_id=w.id GROUP BY w.id HAVING c>=?1 ORDER BY c DESC,w.form LIMIT ?2 OFFSET ?3",
        };
        let min = query.min_count.unwrap_or(1) as i64;
        let mut statement = connection.prepare(sql_text).map_err(sql)?;
        let mut rows = match &query.collection {
            Some(collection) => statement.query(params![collection.as_str(), min, query.page.limit as i64, query.page.offset as i64]).map_err(sql)?,
            None => statement.query(params![min, query.page.limit as i64, query.page.offset as i64]).map_err(sql)?,
        };
        let mut words = Vec::new();
        while let Some(row) = rows.next().map_err(sql)? {
            let id = WordId::parse(row.get::<_, String>(0).map_err(sql)?).map_err(invalid)?;
            let collection = CollectionId::parse(row.get::<_, String>(1).map_err(sql)?).map_err(invalid)?;
            let key = WordKey::parse(row.get::<_, String>(2).map_err(sql)?);
            let form = TargetText::parse(row.get::<_, String>(3).map_err(sql)?).map_err(invalid)?;
            let roman = row.get::<_, Option<String>>(4).map_err(sql)?.map(Romanisation::parse).transpose().map_err(invalid)?;
            let kind = row.get::<_, Option<String>>(5).map_err(sql)?;
            let count = row.get::<_, i64>(6).map_err(sql)? as usize;
            let meanings = load_meanings(&connection, &id)?;
            words.push(WordRow { word: WordEntry::new(id, collection, key, form, roman, kind), sentence_count: count, meanings });
        }
        let total = words.len();
        Ok(WordPage { words, total })
    }
}

/// The schema version this binary writes. Bump when the schema changes and add
/// a matching step to `apply_pending_migrations`.
const CURRENT_SCHEMA_VERSION: u32 = 3;

fn migrate(connection: &Connection) -> Result<(), LibraryFailure> {
    connection.execute_batch(SCHEMA).map_err(sql)?;
    let format: Option<String> = connection.query_row(
        "SELECT value FROM library_metadata WHERE key='format'",
        [],
        |row| row.get(0),
    ).optional().map_err(sql)?;
    match format.as_deref() {
        Some("lingo.library/v1") => apply_pending_migrations(connection),
        Some(other) => Err(LibraryFailure::Invalid(format!("unsupported library format {other}"))),
        None => {
            connection.execute("INSERT INTO library_metadata(key,value) VALUES('format','lingo.library/v1')", []).map_err(sql)?;
            connection.execute("INSERT INTO library_metadata(key,value) VALUES('schema_version',?1)", params![CURRENT_SCHEMA_VERSION.to_string()]).map_err(sql)?;
            connection.execute("INSERT INTO library_metadata(key,value) VALUES('created_by','lingo')", []).map_err(sql)?;
            connection.execute("INSERT INTO library_metadata(key,value) VALUES('created_at',?1)", params![now_utc()]).map_err(sql)?;
            // Fresh DBs are born at the current schema (batch_id is in SCHEMA), so the
            // batch index is created here rather than in SCHEMA (which also runs on
            // pre-v3 DBs where the column doesn't exist yet).
            connection.execute("CREATE INDEX IF NOT EXISTS sentences_by_batch ON sentences(batch_id)", []).map_err(sql)?;
            Ok(())
        }
    }
}

/// Upgrade an existing library in place. Each step is idempotent enough to be
/// re-run, and the version marker is only advanced after the step succeeds.
fn apply_pending_migrations(connection: &Connection) -> Result<(), LibraryFailure> {
    let version: u32 = connection.query_row(
        "SELECT value FROM library_metadata WHERE key='schema_version'",
        [],
        |row| row.get::<_, String>(0),
    ).optional().map_err(sql)?.and_then(|value| value.parse().ok()).unwrap_or(1);

    // v1 -> v2: widen the `status` CHECK to allow 'active' (table rebuild).
    if version < 2 {
        connection.execute_batch(MIGRATE_V2_SENTENCES).map_err(sql)?;
        connection.execute("UPDATE library_metadata SET value='2' WHERE key='schema_version'", []).map_err(sql)?;
    }
    // v2 -> v3: first-class batches + per-sentence title/batch (additive). The
    // `batches` CREATE already ran in SCHEMA; ADD COLUMN is guarded since SQLite
    // has no `ADD COLUMN IF NOT EXISTS`.
    if version < 3 {
        if !column_exists(connection, "sentences", "batch_id")? {
            connection.execute("ALTER TABLE sentences ADD COLUMN batch_id TEXT REFERENCES batches(id) ON DELETE SET NULL", []).map_err(sql)?;
        }
        if !column_exists(connection, "sentences", "title")? {
            connection.execute("ALTER TABLE sentences ADD COLUMN title TEXT", []).map_err(sql)?;
        }
        connection.execute("CREATE INDEX IF NOT EXISTS sentences_by_batch ON sentences(batch_id)", []).map_err(sql)?;
        connection.execute("UPDATE library_metadata SET value='3' WHERE key='schema_version'", []).map_err(sql)?;
    }
    Ok(())
}

fn column_exists(connection: &Connection, table: &str, column: &str) -> Result<bool, LibraryFailure> {
    let mut statement = connection.prepare(&format!("PRAGMA table_info({table})")).map_err(sql)?;
    let exists = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(sql)?
        .filter_map(Result::ok)
        .any(|name| name == column);
    Ok(exists)
}

/// Rebuild `sentences` to swap the `status` CHECK constraint, preserving rows,
/// dependent foreign keys, and indexes (the standard SQLite 12-step ALTER).
const MIGRATE_V2_SENTENCES: &str = r#"
PRAGMA foreign_keys=OFF;
BEGIN;
CREATE TABLE sentences_v2 (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  section TEXT,
  ord INTEGER NOT NULL,
  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register TEXT CHECK (register IS NULL OR register IN ('informal','standard','formal')),
  authority TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(authority)),
  breakdown TEXT CHECK (breakdown IS NULL OR json_valid(breakdown)),
  tokens TEXT CHECK (tokens IS NULL OR json_valid(tokens)),
  tags TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','enriching','enriched','active')),
  enrich_run TEXT,
  provenance TEXT NOT NULL CHECK (json_valid(provenance)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;
INSERT INTO sentences_v2 SELECT id,collection_id,section,ord,target,romanisation,english,literal,register,authority,breakdown,tokens,tags,status,enrich_run,provenance,created_at,updated_at FROM sentences;
DROP TABLE sentences;
ALTER TABLE sentences_v2 RENAME TO sentences;
CREATE UNIQUE INDEX IF NOT EXISTS sentences_unique_order ON sentences(collection_id, ord);
CREATE INDEX IF NOT EXISTS sentences_by_section ON sentences(collection_id, section, ord);
CREATE INDEX IF NOT EXISTS sentences_by_status ON sentences(status, updated_at);
CREATE INDEX IF NOT EXISTS sentences_by_enrich_run ON sentences(enrich_run) WHERE enrich_run IS NOT NULL;
COMMIT;
PRAGMA foreign_keys=ON;
"#;

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS library_metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;
CREATE TABLE IF NOT EXISTS collections (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  language TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;
CREATE TABLE IF NOT EXISTS batches (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  default_title TEXT,
  default_subtitle TEXT,
  raw_text TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;
CREATE INDEX IF NOT EXISTS batches_by_collection ON batches(collection_id, created_at);
CREATE TABLE IF NOT EXISTS sentences (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  batch_id TEXT REFERENCES batches(id) ON DELETE SET NULL,
  title TEXT,
  section TEXT,
  ord INTEGER NOT NULL,
  target TEXT NOT NULL,
  romanisation TEXT,
  english TEXT,
  literal TEXT,
  register TEXT CHECK (register IS NULL OR register IN ('informal','standard','formal')),
  authority TEXT NOT NULL DEFAULT '{}' CHECK (json_valid(authority)),
  breakdown TEXT CHECK (breakdown IS NULL OR json_valid(breakdown)),
  tokens TEXT CHECK (tokens IS NULL OR json_valid(tokens)),
  tags TEXT NOT NULL DEFAULT '[]' CHECK (json_valid(tags)),
  status TEXT NOT NULL DEFAULT 'draft' CHECK (status IN ('draft','enriching','enriched','active')),
  enrich_run TEXT,
  provenance TEXT NOT NULL CHECK (json_valid(provenance)),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;
CREATE UNIQUE INDEX IF NOT EXISTS sentences_unique_order ON sentences(collection_id, ord);
CREATE INDEX IF NOT EXISTS sentences_by_section ON sentences(collection_id, section, ord);
CREATE INDEX IF NOT EXISTS sentences_by_status ON sentences(status, updated_at);
CREATE INDEX IF NOT EXISTS sentences_by_enrich_run ON sentences(enrich_run) WHERE enrich_run IS NOT NULL;
CREATE TABLE IF NOT EXISTS sentence_audio (
  sentence_id TEXT PRIMARY KEY REFERENCES sentences(id) ON DELETE CASCADE,
  path TEXT NOT NULL,
  hash TEXT NOT NULL,
  backend TEXT NOT NULL CHECK (backend IN ('gtts','elevenlabs')),
  format TEXT NOT NULL DEFAULT 'mp3' CHECK (format = 'mp3'),
  voice TEXT,
  model TEXT,
  created_at TEXT NOT NULL
) STRICT;
CREATE TABLE IF NOT EXISTS words (
  id TEXT PRIMARY KEY,
  collection_id TEXT NOT NULL REFERENCES collections(id) ON DELETE CASCADE,
  key TEXT NOT NULL,
  form TEXT NOT NULL,
  roman TEXT,
  kind TEXT,
  UNIQUE(collection_id, key)
) STRICT;
CREATE TABLE IF NOT EXISTS word_meanings (
  id TEXT PRIMARY KEY,
  word_id TEXT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
  meaning TEXT NOT NULL,
  meaning_key TEXT NOT NULL,
  first_seen_sentence_id TEXT REFERENCES sentences(id) ON DELETE SET NULL,
  UNIQUE(word_id, meaning_key)
) STRICT;
CREATE TABLE IF NOT EXISTS sentence_words (
  sentence_id TEXT NOT NULL REFERENCES sentences(id) ON DELETE CASCADE,
  word_id TEXT NOT NULL REFERENCES words(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  surface_form TEXT NOT NULL,
  gloss TEXT,
  PRIMARY KEY (sentence_id, position)
) STRICT;
CREATE INDEX IF NOT EXISTS sentence_words_by_word ON sentence_words(word_id);
PRAGMA user_version = 1;
"#;

fn scalar_usize(connection: &Connection, sql_text: &str) -> Result<usize, LibraryFailure> {
    let value: i64 = connection.query_row(sql_text, [], |row| row.get(0)).map_err(sql)?;
    Ok(value as usize)
}

fn select_ids(connection: &Connection, query: &SentenceQuery) -> Result<Vec<SentenceId>, LibraryFailure> {
    let order = match query.sort { SentenceSort::LibraryOrder => "s.ord ASC", SentenceSort::UpdatedDesc => "s.updated_at DESC" };
    let (sql_text, args): (String, Vec<String>) = match &query.selection {
        SentenceSelection::All => (format!("SELECT s.id FROM sentences s ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![]),
        SentenceSelection::Collection(collection) => (format!("SELECT s.id FROM sentences s WHERE s.collection_id=?3 ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![collection.as_str().to_string()]),
        SentenceSelection::Section { collection, section } => (format!("SELECT s.id FROM sentences s WHERE s.collection_id=?3 AND s.section=?4 ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![collection.as_str().to_string(), section.as_str().to_string()]),
        SentenceSelection::Status(status) => (format!("SELECT s.id FROM sentences s WHERE s.status=?3 ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![status.wire_name().to_string()]),
        SentenceSelection::MissingAudio => (format!("SELECT s.id FROM sentences s LEFT JOIN sentence_audio a ON a.sentence_id=s.id WHERE a.sentence_id IS NULL ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![]),
        SentenceSelection::ReadyToPublish => (format!("SELECT s.id FROM sentences s JOIN sentence_audio a ON a.sentence_id=s.id WHERE s.status IN ('enriched','active') ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![]),
        SentenceSelection::Batch(batch) => (format!("SELECT s.id FROM sentences s WHERE s.batch_id=?3 ORDER BY {order} LIMIT ?1 OFFSET ?2"), vec![batch.as_str().to_string()]),
        SentenceSelection::Ids(ids) => {
            let mut out = Vec::new();
            for id in ids { out.push(id.clone()); }
            return Ok(out);
        }
    };
    let limit = query.page.limit as i64;
    let offset = query.page.offset as i64;
    let mut statement = connection.prepare(&sql_text).map_err(sql)?;
    let mapped = match args.len() {
        0 => statement.query_map(params![limit, offset], row_id).map_err(sql)?,
        1 => statement.query_map(params![limit, offset, args[0].as_str()], row_id).map_err(sql)?,
        2 => statement.query_map(params![limit, offset, args[0].as_str(), args[1].as_str()], row_id).map_err(sql)?,
        _ => return Err(LibraryFailure::Invalid("unsupported query arity".to_string())),
    };
    mapped.collect::<Result<Vec<_>, _>>().map_err(sql)
}

fn row_id(row: &Row<'_>) -> rusqlite::Result<SentenceId> {
    let raw: String = row.get(0)?;
    SentenceId::parse(raw).map_err(|error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error)))
}

fn upsert_collection(connection: &Connection, id: &CollectionId, title: &CollectionTitle, language: &LanguageCode) -> Result<(), LibraryFailure> {
    connection.execute(
        "INSERT INTO collections(id,title,language,created_at,updated_at) VALUES(?1,?2,?3,?4,?4)
         ON CONFLICT(id) DO UPDATE SET title=excluded.title, language=excluded.language, updated_at=excluded.updated_at",
        params![id.as_str(), title.as_str(), language.as_str(), now_utc()],
    ).map_err(sql)?;
    Ok(())
}

fn insert_sentence_row(connection: &Connection, sentence: &Sentence) -> Result<(), LibraryFailure> {
    connection.execute(
        "INSERT INTO sentences(id,collection_id,batch_id,title,section,ord,target,romanisation,english,literal,register,authority,breakdown,tokens,tags,status,enrich_run,provenance,created_at,updated_at)
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,NULL,?14,?15,?16,?17,?18,?18)",
        params![
            sentence.id().as_str(),
            sentence.collection().as_str(),
            sentence.batch_id().map(BatchId::as_str),
            sentence.title().map(SectionName::as_str),
            sentence.section().map(SectionName::as_str),
            sentence.order().get(),
            sentence.target().as_str(),
            sentence.romanisation().map(Romanisation::as_str),
            sentence.english().map(Gloss::as_str),
            sentence.literal().map(LiteralGloss::as_str),
            sentence.register().map(|r| r.wire_name()),
            json(sentence.authority())?,
            sentence.breakdown().map(json).transpose()?,
            json(sentence.tags())?,
            sentence.status().wire_name(),
            sentence.enrich_run().map(RunId::as_str),
            json(sentence.provenance())?,
            now_utc(),
        ],
    ).map_err(sql)?;
    Ok(())
}

fn update_sentence_row(connection: &Connection, sentence: &Sentence) -> Result<(), LibraryFailure> {
    connection.execute(
        "UPDATE sentences SET batch_id=?3,title=?4,section=?5,ord=?6,target=?7,romanisation=?8,english=?9,literal=?10,register=?11,authority=?12,breakdown=?13,tags=?14,status=?15,enrich_run=?16,provenance=?17,updated_at=?18 WHERE id=?1 AND collection_id=?2",
        params![
            sentence.id().as_str(),
            sentence.collection().as_str(),
            sentence.batch_id().map(BatchId::as_str),
            sentence.title().map(SectionName::as_str),
            sentence.section().map(SectionName::as_str),
            sentence.order().get(),
            sentence.target().as_str(),
            sentence.romanisation().map(Romanisation::as_str),
            sentence.english().map(Gloss::as_str),
            sentence.literal().map(LiteralGloss::as_str),
            sentence.register().map(|r| r.wire_name()),
            json(sentence.authority())?,
            sentence.breakdown().map(json).transpose()?,
            json(sentence.tags())?,
            sentence.status().wire_name(),
            sentence.enrich_run().map(RunId::as_str),
            json(sentence.provenance())?,
            now_utc(),
        ],
    ).map_err(sql)?;
    Ok(())
}

fn load_batch(connection: &Connection, id: &BatchId) -> Result<Batch, LibraryFailure> {
    let mut statement = connection.prepare(
        "SELECT id,collection_id,name,default_title,default_subtitle,raw_text,created_at,updated_at FROM batches WHERE id=?1",
    ).map_err(sql)?;
    statement.query_row(params![id.as_str()], batch_from_row).optional().map_err(sql)?.ok_or_else(|| LibraryFailure::NotFound(id.to_string()))
}

fn batch_from_row(row: &Row<'_>) -> rusqlite::Result<Batch> {
    let id = parse_sql(0, BatchId::parse(row.get::<_, String>(0)?))?;
    let collection = parse_sql(1, CollectionId::parse(row.get::<_, String>(1)?))?;
    let name = parse_sql(2, BatchName::parse(row.get::<_, String>(2)?))?;
    let default_title = row.get::<_, Option<String>>(3)?.map(SectionName::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
    let default_subtitle = row.get::<_, Option<String>>(4)?.map(SectionName::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
    let raw_text = row.get::<_, String>(5)?;
    let created_at = row.get::<_, String>(6)?;
    let updated_at = row.get::<_, String>(7)?;
    Ok(Batch::new(id, collection, name, default_title, default_subtitle, raw_text, created_at, updated_at))
}

fn load_sentence(connection: &Connection, id: &SentenceId) -> Result<Sentence, LibraryFailure> {
    let mut statement = connection.prepare(
        "SELECT s.id,s.collection_id,s.batch_id,s.title,s.section,s.ord,s.target,s.romanisation,s.english,s.literal,s.register,
                s.authority,s.breakdown,s.tags,s.status,s.enrich_run,s.provenance,
                a.path,a.hash,a.backend,a.format,a.voice,a.model
         FROM sentences s LEFT JOIN sentence_audio a ON a.sentence_id=s.id WHERE s.id=?1",
    ).map_err(sql)?;
    statement.query_row(params![id.as_str()], sentence_from_row).optional().map_err(sql)?.ok_or_else(|| LibraryFailure::NotFound(id.to_string()))
}

fn sentence_from_row(row: &Row<'_>) -> rusqlite::Result<Sentence> {
    let id = parse_sql(0, SentenceId::parse(row.get::<_, String>(0)?))?;
    let collection = parse_sql(1, CollectionId::parse(row.get::<_, String>(1)?))?;
    let batch_id = row.get::<_, Option<String>>(2)?.map(BatchId::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?;
    let title = row.get::<_, Option<String>>(3)?.map(SectionName::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
    let section = row.get::<_, Option<String>>(4)?.map(SectionName::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;
    let order = parse_sql(5, SentenceOrder::new(row.get::<_, i64>(5)?))?;
    let target = parse_sql(6, TargetText::parse(row.get::<_, String>(6)?))?;
    let romanisation = row.get::<_, Option<String>>(7)?.map(Romanisation::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(7, rusqlite::types::Type::Text, Box::new(e)))?;
    let english = row.get::<_, Option<String>>(8)?.map(Gloss::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(8, rusqlite::types::Type::Text, Box::new(e)))?;
    let literal = row.get::<_, Option<String>>(9)?.map(LiteralGloss::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(9, rusqlite::types::Type::Text, Box::new(e)))?;
    let register = row.get::<_, Option<String>>(10)?.as_deref().map(Register::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(10, rusqlite::types::Type::Text, Box::new(e)))?;
    let authority = from_json::<FieldAuthoritySet>(11, &row.get::<_, String>(11)?)?;
    let breakdown = row.get::<_, Option<String>>(12)?.map(|raw| from_json::<TokenBreakdown>(12, &raw)).transpose()?;
    let tags = from_json::<SentenceTags>(13, &row.get::<_, String>(13)?)?;
    let status = parse_sql(14, SentenceStatus::parse(&row.get::<_, String>(14)?))?;
    let enrich_run = row.get::<_, Option<String>>(15)?.map(RunId::parse).transpose().map_err(|e| rusqlite::Error::FromSqlConversionFailure(15, rusqlite::types::Type::Text, Box::new(e)))?;
    let provenance = from_json::<SentenceProvenance>(16, &row.get::<_, String>(16)?)?;
    let audio_path: Option<String> = row.get(17)?;
    let audio = match audio_path {
        Some(path) => Some(SentenceAudio::new(
            parse_sql(17, AudioRelativePath::parse(path))?,
            parse_sql(18, ContentHash::parse(row.get::<_, String>(18)?))?,
            parse_sql(19, AudioBackendId::parse(&row.get::<_, String>(19)?))?,
            AudioFormat::Mp3,
            row.get(21)?,
            row.get(22)?,
        )),
        None => None,
    };
    Ok(Sentence::from_record(SentenceRecordInput { id, collection, batch_id, title, section, order, target, romanisation, english, literal, register, authority, status, enrich_run, provenance, tags, breakdown, audio }))
}

fn ids_for_run(connection: &Connection, run: &RunId) -> Result<Vec<SentenceId>, LibraryFailure> {
    let mut statement = connection.prepare("SELECT id FROM sentences WHERE status='enriching' AND enrich_run=?1 ORDER BY ord").map_err(sql)?;
    statement.query_map(params![run.as_str()], row_id).map_err(sql)?.collect::<Result<Vec<_>, _>>().map_err(sql)
}

fn rebuild_word_projection(connection: &Connection, sentence: &Sentence) -> Result<(), LibraryFailure> {
    connection.execute("DELETE FROM sentence_words WHERE sentence_id=?1", params![sentence.id().as_str()]).map_err(sql)?;
    let Some(breakdown) = sentence.breakdown() else { return Ok(()); };
    for (index, item) in breakdown.items().iter().enumerate() {
        let key = WordKey::from_surface(item.surface());
        let existing: Option<String> = connection.query_row(
            "SELECT id FROM words WHERE collection_id=?1 AND key=?2",
            params![sentence.collection().as_str(), key.as_str()],
            |row| row.get(0),
        ).optional().map_err(sql)?;
        let word_id = match existing {
            Some(id) => WordId::parse(id).map_err(invalid)?,
            None => {
                let id = WordId::generate();
                connection.execute(
                    "INSERT INTO words(id,collection_id,key,form,roman,kind) VALUES(?1,?2,?3,?4,?5,?6)",
                    params![id.as_str(), sentence.collection().as_str(), key.as_str(), item.surface().as_str(), item.roman().map(Romanisation::as_str), item.kind()],
                ).map_err(sql)?;
                id
            }
        };
        let meaning_key = WordKey::from_meaning(item.gloss().as_str());
        let meaning_id = WordMeaningId::generate();
        connection.execute(
            "INSERT OR IGNORE INTO word_meanings(id,word_id,meaning,meaning_key,first_seen_sentence_id) VALUES(?1,?2,?3,?4,?5)",
            params![meaning_id.as_str(), word_id.as_str(), item.gloss().as_str(), meaning_key.as_str(), sentence.id().as_str()],
        ).map_err(sql)?;
        connection.execute(
            "INSERT INTO sentence_words(sentence_id,word_id,position,surface_form,gloss) VALUES(?1,?2,?3,?4,?5)",
            params![sentence.id().as_str(), word_id.as_str(), index as i64, item.surface().as_str(), item.gloss().as_str()],
        ).map_err(sql)?;
    }
    Ok(())
}

fn load_meanings(connection: &Connection, id: &WordId) -> Result<Vec<Gloss>, LibraryFailure> {
    let mut statement = connection.prepare("SELECT meaning FROM word_meanings WHERE word_id=?1 ORDER BY meaning").map_err(sql)?;
    statement.query_map(params![id.as_str()], |row| {
        let raw: String = row.get(0)?;
        Gloss::parse(raw).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
    }).map_err(sql)?.collect::<Result<Vec<_>, _>>().map_err(sql)
}

fn update_text_field(connection: &Connection, sentence: &SentenceId, column: &str, value: Option<&str>) -> Result<(), LibraryFailure> {
    let sql_text = format!("UPDATE sentences SET {column}=?2, updated_at=?3 WHERE id=?1");
    connection.execute(&sql_text, params![sentence.as_str(), value, now_utc()]).map_err(sql)?;
    Ok(())
}

fn json<T: serde::Serialize>(value: &T) -> Result<String, LibraryFailure> {
    serde_json::to_string(value).map_err(|error| LibraryFailure::Invalid(error.to_string()))
}

fn from_json<T: serde::de::DeserializeOwned>(index: usize, raw: &str) -> rusqlite::Result<T> {
    serde_json::from_str(raw).map_err(|error| rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(error)))
}

fn parse_sql<T, E>(index: usize, result: Result<T, E>) -> rusqlite::Result<T>
where E: std::error::Error + Send + Sync + 'static {
    result.map_err(|error| rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, Box::new(error)))
}

fn invalid(error: impl std::fmt::Display) -> LibraryFailure { LibraryFailure::Invalid(error.to_string()) }
fn sql(error: rusqlite::Error) -> LibraryFailure { LibraryFailure::Sql(error.to_string()) }
fn now_utc() -> String {
    let seconds = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default();
    format!("unix:{seconds}")
}

#[cfg(test)]
mod tests {
    use super::SqliteLibraryStore;
    use lingo_application::ports::*;
    use lingo_domain::*;

    #[test]
    fn migrates_and_round_trips_drafts() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteLibraryStore::open(dir.path().join("library.db")).unwrap();
        let mut authority = FieldAuthoritySet::empty();
        authority.mark(SentenceField::English, FieldAuthority::Human);
        let report = store.insert_drafts(InsertDrafts {
            collection: CollectionId::parse("default").unwrap(),
            title: CollectionTitle::parse("Complete Hindi").unwrap(),
            language: LanguageCode::parse("hi").unwrap(),
            batch_id: None,
            sentence_title: None,
            section: Some(SectionName::parse("Chapter 01").unwrap()),
            drafts: vec![DraftSentence {
                target: TargetText::parse("नमस्ते जी").unwrap(),
                romanisation: Some(Romanisation::parse("namaste jī").unwrap()),
                english: Some(Gloss::parse("Hello ji").unwrap()),
                authority,
                tags: SentenceTags::try_from_values(["greeting".to_string()]).unwrap(),
                provenance: SentenceProvenance::Manual,
            }],
        }).unwrap();
        assert_eq!(report.created, 1);
        let sentence = store.get_sentence(&report.ids[0]).unwrap();
        assert_eq!(sentence.english().unwrap().as_str(), "Hello ji");
    }

    #[test]
    fn claim_prevents_double_claim_and_apply_projects_words() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteLibraryStore::open(dir.path().join("library.db")).unwrap();
        let inserted = store.insert_drafts(InsertDrafts {
            collection: CollectionId::parse("default").unwrap(),
            title: CollectionTitle::parse("Complete Hindi").unwrap(),
            language: LanguageCode::parse("hi").unwrap(),
            batch_id: None,
            sentence_title: None,
            section: None,
            drafts: vec![DraftSentence {
                target: TargetText::parse("नमस्ते").unwrap(),
                romanisation: None,
                english: None,
                authority: FieldAuthoritySet::empty(),
                tags: SentenceTags::default(),
                provenance: SentenceProvenance::Manual,
            }],
        }).unwrap();
        let run = RunId::parse("run-a").unwrap();
        let claimed = store.claim_for_enrichment(ClaimEnrichment { run_id: run.clone(), selection: SentenceSelection::All, limit: 10, force: false }).unwrap();
        assert_eq!(claimed.sentences.len(), 1);
        let second = store.claim_for_enrichment(ClaimEnrichment { run_id: RunId::parse("run-b").unwrap(), selection: SentenceSelection::All, limit: 10, force: false }).unwrap();
        assert!(second.sentences.is_empty());
        store.apply_enrichment(ApplyEnrichmentCommit { run_id: run, enrichments: vec![SentenceEnrichment {
            id: inserted.ids[0].clone(),
            target: None,
            romanisation: Some(Romanisation::parse("namaste").unwrap()),
            english: Some(Gloss::parse("Hello").unwrap()),
            literal: Some(LiteralGloss::parse("greeting").unwrap()),
            register: Some(Register::Standard),
            breakdown: Some(TokenBreakdown::try_new(vec![BreakdownItem::new(TargetText::parse("नमस्ते").unwrap(), Some(Romanisation::parse("namaste").unwrap()), Gloss::parse("hello").unwrap(), None)]).unwrap()),
        }] }).unwrap();
        let words = store.list_words(&WordQuery::default()).unwrap();
        assert_eq!(words.words.len(), 1);
        assert_eq!(words.words[0].sentence_count, 1);
    }

    #[test]
    fn batch_round_trips_membership_and_counts() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteLibraryStore::open(dir.path().join("library.db")).unwrap();
        let batch = store.create_batch(CreateBatch {
            collection: CollectionId::parse("default").unwrap(),
            title: CollectionTitle::parse("Complete Hindi").unwrap(),
            language: LanguageCode::parse("hi").unwrap(),
            name: BatchName::parse("Chapter 02 dialogue").unwrap(),
            default_title: Some(SectionName::parse("Complete Hindi").unwrap()),
            default_subtitle: Some(SectionName::parse("Chapter 02").unwrap()),
            raw_text: "raw one".to_string(),
        }).unwrap().batch;
        let inserted = store.insert_drafts(InsertDrafts {
            collection: CollectionId::parse("default").unwrap(),
            title: CollectionTitle::parse("Complete Hindi").unwrap(),
            language: LanguageCode::parse("hi").unwrap(),
            batch_id: Some(batch.id().clone()),
            sentence_title: None,
            section: Some(SectionName::parse("Chapter 02").unwrap()),
            drafts: vec![DraftSentence {
                target: TargetText::parse("नमस्ते").unwrap(),
                romanisation: None,
                english: None,
                authority: FieldAuthoritySet::empty(),
                tags: SentenceTags::default(),
                provenance: SentenceProvenance::Manual,
            }],
        }).unwrap();
        let sentence = store.get_sentence(&inserted.ids[0]).unwrap();
        assert_eq!(sentence.batch_id(), Some(batch.id()));
        let page = store.list_batches(None).unwrap();
        assert_eq!(page.batches.len(), 1);
        assert_eq!(page.batches[0].draft, 1);
        assert_eq!(page.batches[0].total, 1);
        // append raw text concatenates
        let updated = store.append_batch_raw_text(AppendBatchRawText { batch_id: batch.id().clone(), raw_text: "raw two".to_string() }).unwrap().batch;
        assert!(updated.raw_text().contains("raw one") && updated.raw_text().contains("raw two"));
        // selecting by batch finds the member
        let selected = store.list_sentences(&SentenceQuery { selection: SentenceSelection::Batch(batch.id().clone()), sort: SentenceSort::LibraryOrder, page: PageRequest { limit: 10, offset: 0 } }).unwrap();
        assert_eq!(selected.sentences.len(), 1);
    }

    #[test]
    fn explicit_ids_reclaim_stuck_enriching() {
        let dir = tempfile::tempdir().unwrap();
        let store = SqliteLibraryStore::open(dir.path().join("library.db")).unwrap();
        let inserted = store.insert_drafts(InsertDrafts {
            collection: CollectionId::parse("default").unwrap(),
            title: CollectionTitle::parse("Complete Hindi").unwrap(),
            language: LanguageCode::parse("hi").unwrap(),
            batch_id: None,
            sentence_title: None,
            section: None,
            drafts: vec![DraftSentence {
                target: TargetText::parse("नमस्ते").unwrap(),
                romanisation: None,
                english: None,
                authority: FieldAuthoritySet::empty(),
                tags: SentenceTags::default(),
                provenance: SentenceProvenance::Manual,
            }],
        }).unwrap();
        let id = inserted.ids[0].clone();
        // First claim leaves it stuck in 'enriching' (run abandoned).
        store.claim_for_enrichment(ClaimEnrichment { run_id: RunId::parse("run-a").unwrap(), selection: SentenceSelection::All, limit: 10, force: false }).unwrap();
        // A bulk All claim must NOT re-claim the stuck sentence...
        let bulk = store.claim_for_enrichment(ClaimEnrichment { run_id: RunId::parse("run-b").unwrap(), selection: SentenceSelection::All, limit: 10, force: false }).unwrap();
        assert!(bulk.sentences.is_empty(), "All claim stays draft-only");
        // ...but an explicit Ids selection re-claims it under a fresh run.
        let reclaim = store.claim_for_enrichment(ClaimEnrichment { run_id: RunId::parse("run-c").unwrap(), selection: SentenceSelection::Ids(vec![id]), limit: 10, force: false }).unwrap();
        assert_eq!(reclaim.sentences.len(), 1, "explicit Ids re-claims stuck enriching");
    }
}
